use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use std::env;
use std::sync::Arc;
use futures_util::StreamExt; 
use futures_util::SinkExt;

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
    // split()을 통해 송신부(sender)와 수신부(receiver)를 나눕니다.
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // 1. 메시지 수신 (수신부) - spawn으로 비동기 처리
    let tx_clone = state.tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx_clone.send(text);
        }
    });

    // 2. 메시지 전달 (송신부) - 루프에서 직접 처리
    while let Ok(msg) = rx.recv().await {
        // sender를 사용하여 메시지를 전송합니다.
        if sender.send(Message::Text(msg)).await.is_err() {
            break; 
        }
    }
}