use miniai_daemon::DaemonServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_addr =
        std::env::var("MINIAI_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:4140".to_string());
    println!(
        "[*] Starting MiniAI Local Daemon on http://{}...",
        bind_addr
    );

    let server = DaemonServer::bind(&bind_addr, None).await?;
    println!(
        "[+] MiniAI Daemon listening on http://{}",
        server.local_addr()
    );
    println!("[+] Endpoints active: /v1/health, /v1/bridge/dispatch");

    server.run().await?;
    Ok(())
}
