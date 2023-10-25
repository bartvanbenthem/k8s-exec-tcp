use chrono::Utc;
use clap::{App, Arg};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use std::error::Error;
use std::sync::Arc;
use tracing::*;

use kube::{
    api::{
        Api, AttachParams, AttachedProcess, DeleteParams, PostParams, ResourceExt, WatchEvent,
        WatchParams,
    },
    Client,
};

#[derive(Debug)]
pub struct Config {
    namespace: String,
    image: String,
    hosts: Vec<String>,
    ports: Vec<u32>,
    max_connections: usize,
}

impl Config {
    pub fn build() -> Result<Config, Box<dyn Error>> {
        // Define and parse command-line arguments using clap
        let matches = App::new("k8s-exec-tcp")
            .arg(
                Arg::with_name("namespace")
                    .short("n")
                    .long("namespace")
                    .required(false)
                    .takes_value(true)
                    .help("Kubernetes Namespace"),
            )
            .arg(
                Arg::with_name("image")
                    .short("i")
                    .long("image")
                    .required(false)
                    .takes_value(true)
                    .help("Override alpine container image"),
            )
            .arg(
                Arg::with_name("hosts")
                    .short("h")
                    .long("hosts")
                    .required(false)
                    .takes_value(true)
                    .multiple(true)
                    .help("Space separated list of hosts"),
            )
            .arg(
                Arg::with_name("ports")
                    .short("p")
                    .long("ports")
                    .required(true)
                    .takes_value(true)
                    .multiple(true)
                    .help("Port that remote host listens on"),
            )
            .arg(
                Arg::with_name("connections")
                    .short("c")
                    .long("max-connections")
                    .required(false)
                    .takes_value(true)
                    .help("Port that remote host listens on"),
            )
            .get_matches();

        let mut param_hosts: Vec<String> = vec![];
        if let Some(values) = matches.values_of("hosts") {
            for value in values {
                param_hosts.push(value.to_string());
            }
        }

        let mut param_ports: Vec<u32> = vec![];
        if let Some(values) = matches.values_of("ports") {
            for value in values {
                param_ports.push(value.parse::<u32>().unwrap_or(8080));
            }
        }

        Ok(Config {
            hosts: param_hosts,
            ports: param_ports,
            namespace: matches
                .value_of("namespace")
                .unwrap_or("default")
                .to_string(),
            image: matches.value_of("image").unwrap_or("alpine").to_string(),
            max_connections: matches
                .value_of("connections")
                .unwrap_or("10")
                .parse::<usize>()
                .unwrap_or(10),
        })
    }
}

pub async fn get_output(mut attached: AttachedProcess) -> String {
    let stdout = tokio_util::io::ReaderStream::new(attached.stdout().unwrap());
    let out = stdout
        .filter_map(|r| async { r.ok().and_then(|v| String::from_utf8(v.to_vec()).ok()) })
        .collect::<Vec<_>>()
        .await
        .join("");
    attached.join().await.unwrap();
    out
}

pub async fn check_remote_host(
    host: &str,
    port: &u32,
    name: &str,
    pods: Api<Pod>,
) -> Result<(), Box<dyn Error>> {
    let command = format!(
        "timeout 5 nc -zv -w 2 {} {} && echo -n 'tcpcheck-successful' || echo -n 'tcpcheck-failed'",
        host, port
    );

    let attached = pods
        .exec(
            &name,
            vec!["sh", "-c", &command],
            &AttachParams::default().stderr(false).stderr(false),
        )
        .await?;
    let output = get_output(attached).await;
    println!("{output} on host: {} port: {}", host, port);

    Ok(())
}

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
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
                "image": &config.image,
                // Do nothing
                "command": ["tail", "-f", "/dev/null"],
            }],
        }
    }))?;

    let pods: Api<Pod> = Api::namespaced(client, &config.namespace);

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

    // Initialize and configure a semaphore with the value of --max-connections
    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_connections));
    // Collect JoinHandles in a vector
    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for port in config.ports {
        for host in config.hosts.clone() {
            let name = name.to_string();
            let pods = pods.clone();
            let semaphore = semaphore.clone();

            let handle = tokio::spawn(async move {
                let permit = semaphore.acquire().await.unwrap();
                if let Err(err) = check_remote_host(&host, &port, &name, pods).await {
                    eprintln!("Error for host {}: {:?}", host, err);
                }
                drop(permit);
            });

            handles.push(handle);
        }
    }

    // Wait for all the spawned tasks to complete
    for handle in handles {
        handle.await?;
    }

    // Delete it
    pods.delete(&name, &DeleteParams::default())
        .await?
        .map_left(|pdel| {
            assert_eq!(pdel.name_any(), name);
        });

    Ok(())
}
