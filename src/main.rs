use dotenv::dotenv;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::{
    collections::HashMap, convert::Infallible, error::Error, net::SocketAddr, sync::Arc, thread,
    time::Duration,
};
use tokio::{sync::Mutex, task, time::sleep};

use crate::store::store::{Data, RedisStore, Store};

pub mod store;

async fn router(
    req: Request<Body>,
    store: Arc<Mutex<Store>>,
) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::POST, "/get") => handle_get(req, store).await,
        (&hyper::Method::POST, "/set") => handle_set(req, store).await,
        (&hyper::Method::POST, "/delete") => handle_delete(req, store).await,
        _ => handle_not_found().await,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let router = Arc::new(router);
    let data: HashMap<String, Data> = HashMap::new();
    let store = Store { data };

    let store = Arc::new(Mutex::new(store));

    let make_svc = make_service_fn(|_conn| {
        let router = router.clone();
        let store = store.clone();
        let service = service_fn(move |req| router(req, store.clone()));
        async { Ok::<_, Infallible>(service) }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("Server running at port: {}", addr.port());

    // tokio::spawn({
    //     let store = Arc::clone(&store);
    //     async move {
    //         loop {
    //             let mut store = store.lock().await;
    //             info!("Running cleanups....");
    //             // Sleep for 5 seconds
    //             sleep(Duration::from_secs(1)).await;

    //             store.cleanup_expired();
    //         }
    //     }
    // });

    let server = Server::bind(&addr).serve(make_svc);
    server.await?;

    Ok(())
}

async fn handle_not_found() -> Result<Response<Body>, Infallible> {
    let contents = "Not Found.";
    let response = Response::builder().body(Body::from(contents)).unwrap();

    Ok(response)
}

async fn handle_get(
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

async fn handle_set(
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

async fn handle_delete(
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

#[derive(Debug, Serialize, Deserialize)]
struct GetRequest {
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetRequest {
    key: String,
    data: String,
    expires_in: u64,
}
