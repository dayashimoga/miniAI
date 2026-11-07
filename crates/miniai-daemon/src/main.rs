use miniai_daemon::DaemonServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "doctor") {
        let engine = miniai_bridge::BridgeEngine::new_in_memory()?;
        let resp = engine.dispatch(miniai_bridge::BridgeRequest::RunDoctor);
        if let Some(data) = resp.data {
            println!("{}", serde_json::to_string_pretty(&data)?);
        } else {
            eprintln!("Doctor diagnostic failed: {:?}", resp.error);
        }
        return Ok(());
    }

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
