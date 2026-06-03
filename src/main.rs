use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Render가 할당해주는 포트를 사용 (설정 안 되어 있으면 10000번)
    let port = env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    println!("OxidizeChat 서버가 {}에서 실행 중입니다...", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            // 클라이언트가 보낸 데이터의 첫 부분을 읽음
            let n = socket.read(&mut buf).await.unwrap_or(0);
            
            // 1. Render의 HTTP 스캔(GET)을 감지하면 HTTP 응답을 보내 스캔을 통과시킴
            if n > 0 && buf.starts_with(b"GET") {
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello Server!";
                let _ = socket.write_all(response.as_bytes()).await;
            } else {
                // 2. 일반 TCP 채팅 접속이라면 환영 메시지 전송
                let _ = socket.write_all(b"OxidizeChat Connected!\n").await;
                
                // 채팅 루프: 입력받은 내용을 그대로 돌려줌 (Echo)
                loop {
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    if n == 0 { break; }
                    let _ = socket.write_all(&buf[0..n]).await;
                }
            }
        });
    }
}