use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::env;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new().route("/ws", get(ws_handler));

    println!("OxidizeChat 웹소켓 서버가 {}에서 실행 중입니다...", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // 접속 시 환영 메시지
    if let Err(_) = socket.send(Message::Text("OxidizeChat WebSocket에 연결되었습니다!".into())).await {
        return;
    }

    // 메시지 수신 및 에코(Echo)
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let reply = format!("받은 메시지: {}", text);
            if socket.send(Message::Text(reply)).await.is_err() {
                break;
            }
        }
    }
}