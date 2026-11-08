use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use miniai_daemon::DaemonServer;

#[tokio::test]
async fn test_daemon_server_health_and_dispatch() {
    // Bind to dynamic loopback port
    let server = DaemonServer::bind("127.0.0.1:0", None)
        .await
        .expect("bind daemon");
    let (addr, _handle) = server.spawn();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // 1. Test GET /v1/health
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let req = format!(
            "GET /v1/health HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            addr
        );
        stream
            .write_all(req.as_bytes())
            .await
            .expect("write request");

        let mut resp = String::new();
        stream
            .read_to_string(&mut resp)
            .await
            .expect("read response");
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("miniai-daemon"));
    }

    // 2. Test OPTIONS CORS Preflight
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let req = format!(
            "OPTIONS /v1/bridge/dispatch HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            addr
        );
        stream
            .write_all(req.as_bytes())
            .await
            .expect("write request");

        let mut resp = String::new();
        stream
            .read_to_string(&mut resp)
            .await
            .expect("read response");
        assert!(resp.contains("204 No Content"));
        assert!(resp.contains("Access-Control-Allow-Origin: *"));
    }

    // 3. Test POST /v1/bridge/dispatch with ProbeHardware
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let body = r#"{"action":"probe_hardware"}"#;
        let req = format!(
            "POST /v1/bridge/dispatch HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            addr,
            body.len(),
            body
        );
        stream
            .write_all(req.as_bytes())
            .await
            .expect("write request");

        let mut resp = String::new();
        stream
            .read_to_string(&mut resp)
            .await
            .expect("read response");
        assert!(resp.contains("200 OK"));
        assert!(resp.contains(r#""success":true"#));
        assert!(resp.contains("logical_cpu_cores"));
    }

    // 4. Test POST /v1/bridge/dispatch with ExecuteChat
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let body =
            r#"{"action":"execute_chat","payload":{"session_id":"s-01","prompt":"hello world"}}"#;
        let req = format!(
            "POST /v1/bridge/dispatch HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            addr,
            body.len(),
            body
        );
        stream
            .write_all(req.as_bytes())
            .await
            .expect("write request");

        let mut resp = String::new();
        stream
            .read_to_string(&mut resp)
            .await
            .expect("read response");
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("CPU (Universal Baseline)"));
    }
}

#[tokio::test]
async fn test_daemon_server_bearer_auth() {
    let server = DaemonServer::bind("127.0.0.1:0", None)
        .await
        .expect("bind daemon")
        .with_auth_token("ephemeral_secret_token_abc123");
    let (addr, _handle) = server.spawn();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // 1. Request without auth token must receive 401 Unauthorized
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let req = format!(
            "GET /v1/health HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            addr
        );
        stream.write_all(req.as_bytes()).await.unwrap();

        let mut resp = String::new();
        stream.read_to_string(&mut resp).await.unwrap();
        assert!(resp.contains("401 Unauthorized"));
        assert!(resp.contains("UNAUTHORIZED"));
    }

    // 2. Request with invalid auth token must receive 401 Unauthorized
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let req = format!(
            "GET /v1/health HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer wrong_token\r\nConnection: close\r\n\r\n",
            addr
        );
        stream.write_all(req.as_bytes()).await.unwrap();

        let mut resp = String::new();
        stream.read_to_string(&mut resp).await.unwrap();
        assert!(resp.contains("401 Unauthorized"));
    }

    // 3. Request with valid auth token must receive 200 OK
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let req = format!(
            "GET /v1/health HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer ephemeral_secret_token_abc123\r\nConnection: close\r\n\r\n",
            addr
        );
        stream.write_all(req.as_bytes()).await.unwrap();

        let mut resp = String::new();
        stream.read_to_string(&mut resp).await.unwrap();
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("miniai-daemon"));
    }

    // 4. Request with valid X-MiniAI-Session-Nonce must receive 200 OK
    {
        let mut stream = TcpStream::connect(addr).await.expect("connect to daemon");
        let req = format!(
            "GET /v1/health HTTP/1.1\r\nHost: {}\r\nX-MiniAI-Session-Nonce: ephemeral_secret_token_abc123\r\nConnection: close\r\n\r\n",
            addr
        );
        stream.write_all(req.as_bytes()).await.unwrap();

        let mut resp = String::new();
        stream.read_to_string(&mut resp).await.unwrap();
        assert!(resp.contains("200 OK"));
        assert!(resp.contains("miniai-daemon"));
    }
}
