use dotenv::dotenv;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use log::info;
use std::{collections::HashMap, convert::Infallible, error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::service::{handle_delete, handle_get, handle_not_found, handle_set};
use crate::store::{Data, Store};

pub mod service;
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
    //             sleep(Duration::from_secs(5)).await;

    //             store.cleanup_expired();
    //         }
    //     }
    // });

    let ctrl_c = tokio::signal::ctrl_c();
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(async {
            ctrl_c.await.expect("Failed to handle ctrl + c.");
            info!("Shutting down the server, closing all connections.");
        });

    server.await?;

    info!("Server has been shutdown successfully.");

    Ok(())
}
