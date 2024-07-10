use dotenv;

use mongodb::{ 
    bson::{Document, doc, oid},
    sync::{Client, Collection},
    error::Result,
    error::Error
}; 

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use log::{info, LevelFilter};
use env_logger::Env;
use teloxide::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Hello, chat program started!");
    dotenv::dotenv().ok();
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let tgtoken = dotenv::var("TG_TOKEN").unwrap();
    let tgchannelid = dotenv::var("TG_CHAT_ID").unwrap();
    let tgbot = Bot::from_env();
    let tgbot = Arc::new(Mutex::new(tgbot));
    
    let tgbot_clone = Arc::clone(&tgbot);

    let make_svc = make_service_fn(move |_conn| {
        let tgbot_clone = Arc::clone(&tgbot_clone);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, Arc::clone(&tgbot_clone))
            }))
        }
    });

    let webserver = Server::bind(&addr).serve(make_svc);
    info!("Listening on http://{}", addr);

    teloxide::repl(tgbot.clone(), |bot: Bot, msg: Message| async move {
        info!("Received a message with content: {:?}", msg.text());
        Ok(())
    })
    .await;

    if let Err(e) = webserver.await {
        info!("server error: {}", e);
    }
    
}


async fn handle_request(req: Request<Body>, _bot: Arc<Mutex<Bot>>) -> Result<Response<Body>> {
    info!("Received a request to the webserver, {:?}", req.uri());
    Ok(Response::new(Body::from("Hello chat")))
}