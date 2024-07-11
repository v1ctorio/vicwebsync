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
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting the server");
    let web_server = tokio::spawn(start_web_server());
    let telegram_bot = tokio::spawn(start_telegram_bot());

    let _ = tokio::try_join!(web_server, telegram_bot);

}


async fn handle_request(req: Request<Body>) -> Result<Response<Body>> {
    info!("Received a request to the webserver, {:?}", req.uri());
    Ok(Response::new(Body::from("Hello chat")))
}

async fn start_web_server() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let make_svc = make_service_fn(|_conn| {
        async {
            Ok::<_, Infallible>(service_fn(handle_request))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn start_telegram_bot() {
    let bot = Bot::from_env();
    info!("Bot successfully created");
    teloxide::repl(bot, |_bot: Bot, msg: Message| async move {
        info!("Received a message from the bot, {:?}", msg.text());
        Ok(())
    })
    .await;
}