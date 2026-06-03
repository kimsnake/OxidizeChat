use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use std::env;
use std::sync::Arc;

struct AppState {
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    // 100명까지 메시지를 담을 수 있는 브로드캐스트 채널 생성
    let (tx, _) = broadcast::channel(100);
    let app_state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // 메시지 수신 (한 명이 보낸 걸 다른 모두에게 broadcast)
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.recv().await {
            let _ = state.tx.send(text);
        }
    });

    // 메시지 전달 (구독 중인 모든 사람에게 전송)
    while let Ok(msg) = rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() { break; }
    }
}