use dotenv;
use frankenstein::Api;
use frankenstein::GetUpdatesParams;

use frankenstein::TelegramApi;
use frankenstein::UpdateContent;

use mongodb::{ 
    bson::{Document, doc, oid},
    sync::{Client, Collection},
    error::Result,
    error::Error
}; 
use rouille::Request;
use rouille::Response;

use std::thread;
use chrono;

fn main() -> Result<()>{
    println!("Hello, chat program started!");
    thread::spawn(|| {
        rouille::start_server("0.0.0.0:5432", move |req| {
            handle_req(&req)
        });
    });
    dotenv::dotenv().ok();
    
    let tgtoken = dotenv::var("TG_TOKEN").unwrap();
    let tgchannelid = dotenv::var("TG_CHAT_ID").unwrap();
    let tgapi = Api::new(&tgtoken);
    
    println!("Hello, world! TGAPITOKEN is {:?}", tgtoken);
    let upb = GetUpdatesParams::builder();
    let mut update_params = upb.clone().build();

    //let mongouri = "mongodb://idk";
    let mongouri = dotenv::var("MONGOURL").unwrap();
    // let mongooptions =
    // ClientOptions::parse(&mongouri, ResolverConfig::cloudflare())
    //    .await?;
    let mongoclient = Client::with_uri_str(mongouri)?;
    let db = mongoclient.database("vicweb");
    let posts_collection: Collection<Document> = db.collection("posts");
    Ok::<(), Error>(()) ;

    loop {
        let result = tgapi.get_updates(&update_params);
        match result {
            Ok(updates) => {
                for update in updates.result {
                    println!("Recived an update: {:?}", update.content);
                    if let UpdateContent::Message(message) = update.content {
                        let chatid = message.chat.id;
                        let text = message.text.unwrap();
                        println!("Chat id: {:?}, Text: {:?}", chatid, &text);
                        
                        //(if chat id is tgchannel id)
                        if chatid == tgchannelid.parse::<i64>().unwrap() {
                            let existing_post = posts_collection.find_one(doc!{"text": &text}, None)?;
                            if existing_post.is_none() {
                                let post = doc! {
                                    "text": &text,
                                    "created_at": chrono::Utc::now().timestamp(),
                                    "_id": oid::ObjectId::new(),
                                };
                                match posts_collection.insert_one(post.clone(),None) {
                                    Ok(resulting_post) => {
                                        println!("New post added to db: {:?}", resulting_post);
                                    }
                                    Err(e) => {
                                        println!("Error adding post to db: {:?}", e);
                                    } 
                                };
                                println!("New post added to db: {:?}", text);
                            } else {
                                println!("Ignoring post sin it already exists in db: {:?}", text);
                            }
                        }

                    }
                    update_params = upb
                        .clone()
                        .offset(update.update_id + 1)
                        .build();

                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

fn handle_req(req: &Request) -> Response {
    println!("Request: {:?}", req);
    Response::text("hello cHAT")
} 