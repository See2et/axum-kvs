use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use tokio::sync::{mpsc, oneshot};

enum Message {
    GetValue(String, oneshot::Sender<String>),
}

#[tokio::main]
async fn main() {
    let mut db: HashMap<String, String> = HashMap::new();
    db.insert("cat".to_string(), "neko".to_string());
    db.insert("dog".to_string(), "inu".to_string());
    db.insert("bird".to_string(), "tori".to_string());

    let (tx, rx) = mpsc::channel(32);
    tokio::spawn(async move { store_keys(rx, db).await });

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/get-key/:key", get(get_key).with_state(tx));

    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listner, app).await.unwrap();
}

async fn get_key(Path(param): Path<String>, State(sender): State<mpsc::Sender<Message>>) -> String {
    let (tx, rx) = oneshot::channel();
    sender.send(Message::GetValue(param, tx)).await;
    match rx.await {
        Ok(v) => v.to_string(),
        Err(_) => panic!(),
    }
}

async fn store_keys(mut reciever: mpsc::Receiver<Message>, db: HashMap<String, String>) {
    let mut value = "".to_string();
    while let Some(msg) = reciever.recv().await {
        match msg {
            Message::GetValue(key, tx) => {
                value = db.get(&key).unwrap().to_string();
                tx.send(value.clone());
            }
            _ => panic!(),
        }
    }
}
