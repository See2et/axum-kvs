use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::get,
    Router,
};
use tokio::sync::mpsc;

enum Message {
    Key(String),
    Value(String),
    Sender(mpsc::Sender<Message>),
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
    let (tx, mut rx) = mpsc::channel(32);
    sender.send(Message::Key(param)).await;
    sender.send(Message::Sender(tx)).await;
    if let Some(msg) = rx.recv().await {
        match msg {
            Message::Value(v) => v.to_string(),
            _ => "".to_string(),
        }
    } else {
        "".to_string()
    }
}

async fn store_keys(mut reciever: mpsc::Receiver<Message>, db: HashMap<String, String>) {
    let mut value = "".to_string();
    while let Some(msg) = reciever.recv().await {
        match msg {
            Message::Key(k) => {
                value = db.get(&k).unwrap().to_string();
            }
            Message::Sender(tx) => {
                tx.send(Message::Value(value.clone())).await;
            }
            _ => (),
        }
    }
}
