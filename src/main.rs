use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 8080 포트로 서버를 엽니다.
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    // 2. 메시지를 브로드캐스트할 채널을 생성합니다 (최대 10개 메시지 유지).
    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // 3. 각 클라이언트마다 별도의 비동기 태스크를 생성합니다.
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    // 클라이언트로부터 메시지를 받을 때
                    result = reader.read_line(&mut line) => {
                        if result.unwrap_or(0) == 0 { break; }
                        let msg = format!("{}: {}", addr, line);
                        let _ = tx.send(msg);
                        line.clear();
                    }
                    // 다른 사람이 보낸 메시지를 받을 때
                    result = rx.recv() => {
                        let msg = result.unwrap();
                        let _ = writer.write_all(msg.as_bytes()).await;
                    }
                }
            }
        });
    }
}