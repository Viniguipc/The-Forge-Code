mod jade;
mod sim;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use sim::{start_simulation_engine, types::SimCommand, SimulationController};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let controller = start_simulation_engine();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/health", get(|| async { "OK" }))
        .fallback_service(ServeDir::new("web/dist"))
        .layer(CorsLayer::permissive())
        .with_state(controller);

    let addr = "127.0.0.1:3030";
    info!("CartelGAS JADE-Rust server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(controller): State<Arc<SimulationController>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, controller))
}

async fn handle_socket(mut socket: WebSocket, controller: Arc<SimulationController>) {
    // Send latest snapshot immediately on connect
    if let Some(latest) = controller.latest_snapshot.read().await.clone() {
        if let Ok(json) = serde_json::to_string(&latest) {
            if socket.send(Message::Text(json)).await.is_err() {
                return;
            }
        }
    }

    let mut rx = controller.snapshot_tx.subscribe();

    loop {
        tokio::select! {
            Ok(snap) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&snap) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Text(text) => {
                        if let Ok(cmd) = serde_json::from_str::<SimCommand>(&text) {
                            let _ = controller.cmd_tx.send(cmd).await;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            else => break,
        }
    }
}
