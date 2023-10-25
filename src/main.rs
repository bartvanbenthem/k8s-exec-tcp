use std::process;


#[tokio::main]
async fn main() {

    let config: k8stcp::Config = k8stcp::get_args().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = k8stcp::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
