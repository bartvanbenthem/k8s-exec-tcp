use std::process;

use k8stcp::Config;

#[tokio::main]
async fn main() {
    let config: Config = Config::build().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = k8stcp::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
