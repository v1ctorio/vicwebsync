use chrono::Local;
use dotenv;

use mongodb::{ 
    bson::{Document, doc, oid},
    Client, Collection,
    error::Result,
}; 
use serenity::client::Client as SerenityClient;
use futures::stream::TryStreamExt;use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::channel::Message as DiscordMessage;
use std::convert::Infallible;
use std::net::SocketAddr;
use log::info;
use env_logger::Env;
use teloxide::prelude::*;
use std::sync::Arc;

use serde::{ Deserialize, Serialize };
use serde_json;
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Post {
    id: oid::ObjectId,
    date: String,
    content: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mongo_client = Client::with_uri_str(&std::env::var("MONGO_URI").expect("MONGO_URI must be set"))
    .await
    .expect("Failed to initialize standalone client.");
    let db: Collection<Document> = mongo_client.database("vicweb").collection("posts");
    
    let collection_arc = Arc::new(db);
    info!("Connected to the database, {:?}",collection_arc.clone().find(None,None).await);

    info!("Starting the server");
    let web_server = tokio::spawn(start_web_server(collection_arc.clone()));
    let telegram_bot = tokio::spawn(start_telegram_bot(collection_arc.clone()));

    let _ = tokio::try_join!(web_server, telegram_bot);

}


async fn handle_request(req: Request<Body>,mongo: Arc<Collection<Document>>) -> Result<Response<Body>> {

    info!("Trying to fulfull web request");
    info!("Received a request to the webserver, {:?}", req.uri());
    
    let uri = req.uri().path();
    
    if uri == "/posts" {
        let cursor = mongo.find(None, None).await.expect("Failed to execute find.");
        //let mut posts: Vec<Post> = Vec::new();
        
        
        let posts: Vec<_> = cursor.try_collect().await?;
        

        info!("Data from the database: {:?}", posts);
        let response = serde_json::to_string(&posts).unwrap();        
        return Ok(Response::new(Body::from(response)));
    }
    if uri == "/post" {
        let q = req.uri().query().unwrap();
        
        let post = mongo.find_one(doc! {"_id": oid::ObjectId::parse_str(q).unwrap()}, None).await.expect("Failed to execute find_one.");
        
        let post = post.unwrap();
        let response = serde_json::to_string(&post).unwrap();
        return Ok(Response::new(Body::from(response)));
    }
    Ok(Response::new(Body::from("Hello chat")))
}

async fn start_web_server(mongo: Arc<Collection<Document>>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let make_svc = make_service_fn(|_conn| {
        let mongo = mongo.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, mongo.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn start_telegram_bot(mongo: Arc<Collection<Document>>) {
    let bot = Bot::from_env();
    info!("Bot successfully created");
    teloxide::repl(bot, move |_bot: Bot, msg: Message| {
        let mongo = Arc::clone(&mongo);
        async move {
            let content = msg.text();

            let possible_post = mongo.find_one(doc! {"content": content}, None).await.expect("Failed to execute find_one.");
            if possible_post.is_none() {
                mongo.insert_one(doc! {
                    "_id": oid::ObjectId::new(),
                    "date": Local::now().to_string(),
                    "content": content.unwrap().to_string(),
                }, None).await.expect("Failed to insert document.");
                info!("Inserted a new post into the database");
                return Ok(())
            }

            info!("Ignoring message since its already in the database");
            info!("Received a message from the bot, {:?}", msg.text());
            Ok(())
        }
    })
    .await;
}


struct Handler {
    mongo: Arc<Collection<Document>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: DiscordMessage) {
        if msg.author.id.get() == std::env::var("DISCORD_OWNER_ID").unwrap().parse::<u64>().unwrap() {
            return;
        }
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
            return ;
        }
            let possible_post = self.mongo.find_one(doc! {"content": &msg.content}, None).await.expect("Failed to execute find_one.");
            if possible_post.is_none() {
                self.mongo.insert_one(doc! {
                    "_id": oid::ObjectId::new(),
                    "date": Local::now().to_string(),
                    "content": &msg.content,
                }, None).await.expect("Failed to insert document.");
                info!("Inserted a new post into the database");
                return;
            }

            info!("Ignoring message since its already in the database");
            info!("Received a message from the bot, {:?}", &msg.content);

    }
}

async fn start_discord_bot(mongo: Arc<Collection<Document>>) {
    let token = &std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
        let mut client =
        SerenityClient::builder(&token, intents).event_handler(Handler {mongo}).await.expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
