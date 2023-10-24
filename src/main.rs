use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use tracing::*;
use chrono::Utc;

use kube::{
    api::{
        Api, AttachParams, AttachedProcess, DeleteParams, PostParams, ResourceExt, WatchEvent,
        WatchParams,
    },
    Client,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    let name = format!("tcp-check-{}", Utc::now().timestamp());
    let p: Pod = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": { "name": &name },
        "spec": {
            "restartPolicy": "Never",
            "containers": [{
                "name": &name,
                "image": "busybox",
                // Do nothing
                "command": ["tail", "-f", "/dev/null"],
            }],
        }
    }))?;

    let pods: Api<Pod> = Api::default_namespaced(client);

    // Stop on error including a pod already exists or is still being deleted.
    pods.create(&PostParams::default(), &p).await?;

    // Wait until the pod is running, otherwise we get 500 error.
    let wp = WatchParams::default()
        .fields(format!("metadata.name={}", name).as_str())
        .timeout(10);
    let mut stream = pods.watch(&wp, "0").await?.boxed();
    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(o) => {
                info!("Added {}", o.name_any());
            }
            WatchEvent::Modified(o) => {
                let s = o.status.as_ref().expect("status exists on pod");
                if s.phase.clone().unwrap_or_default() == "Running" {
                    info!("Ready to attach to {}", o.name_any());
                    break;
                }
            }
            _ => {}
        }
    }
    
    let host = "172.22.128.32";
    let port = 22;
    let command = format!(
        "if nc -zv {} {} 2>/dev/null; 
                            then echo -n 'tcpcheck-successful'; 
                            else echo -n 'tcpcheck-failed'; fi",
        host, port
    );

    {
        let attached = pods
            .exec(
                &name,
                vec!["sh", "-c", &command],
                &AttachParams::default().stderr(false).stderr(false),
            )
            .await?;
        let output = get_output(attached).await;
        println!("{output} on host: {} port: {}", host, port);
    }

    // Delete it
    pods.delete(&name, &DeleteParams::default())
        .await?
        .map_left(|pdel| {
            assert_eq!(pdel.name_any(), name);
        });

    Ok(())
}

async fn get_output(mut attached: AttachedProcess) -> String {
    let stdout = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let out = stdout
        .filter_map(|r| async { r.ok().and_then(|v| String::from_utf8(v.to_vec()).ok()) })
        .collect::<Vec<_>>()
        .await
        .join("");
    attached.join().await.unwrap();
    out
}
