use hyper::{Body, Request, Response};
use log::info;
use serde::{Deserialize, Serialize};

use std::{convert::Infallible, sync::Arc};
use tokio::sync::Mutex;

use crate::store::{RedisStore, Store};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRequest {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetRequest {
    key: String,
    data: String,
    expires_in: u64,
}

pub async fn handle_not_found() -> Result<Response<Body>, Infallible> {
    let contents = "Not Found.";
    let response = Response::builder().body(Body::from(contents)).unwrap();

    Ok(response)
}

pub async fn handle_get(
    req: Request<Body>,
    store: Arc<Mutex<Store>>,
) -> Result<Response<Body>, Infallible> {
    let store = store.lock().await;

    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    let data: GetRequest = serde_json::from_str(&body_str).unwrap();

    info!("Getting key: ");
    info!("{:?}", data);

    let contents = store.get(data.key);

    let response = Response::builder().body(Body::from(contents)).unwrap();

    Ok(response)
}

pub async fn handle_set(
    req: Request<Body>,
    store: Arc<Mutex<Store>>,
) -> Result<Response<Body>, Infallible> {
    let mut store = store.lock().await;

    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    let data: SetRequest = serde_json::from_str(&body_str).unwrap();

    info!("Setting key: ");
    info!("{:?}", data);

    store.set(data.key, data.data, data.expires_in);

    let response = Response::builder()
        .body(Body::from("Completed..."))
        .unwrap();

    Ok(response)
}

pub async fn handle_delete(
    req: Request<Body>,
    store: Arc<Mutex<Store>>,
) -> Result<Response<Body>, Infallible> {
    let mut store = store.lock().await;

    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    let data: GetRequest = serde_json::from_str(&body_str).unwrap();

    info!("Deleting key: ");
    info!("{:?}", data);

    store.delete(data.key);

    let response = Response::builder()
        .body(Body::from("Completed..."))
        .unwrap();

    Ok(response)
}
