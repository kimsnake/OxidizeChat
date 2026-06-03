use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Render는 환경 변수로 PORT를 전달합니다. 없다면 기본값 8080을 사용합니다.
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    println!("OxidizeChat 서버가 {}에서 실행 중입니다...", addr);

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // 클라이언트에게 환영 메시지 전송
            let welcome = "환영합니다! OxidizeChat 서버에 접속하셨습니다.\n";
            if let Err(e) = socket.write_all(welcome.as_bytes()).await {
                eprintln!("메시지 전송 실패: {}", e);
                return;
            }

            // 에코(Echo) 기능: 입력받은 내용을 그대로 돌려줌
            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("읽기 오류: {}", e);
                        return;
                    }
                };

                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("쓰기 오류: {}", e);
                    return;
                }
            }
        });
    }
}