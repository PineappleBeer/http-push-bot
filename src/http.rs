use crate::*;
use actix_web::{web, App,HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::sync::Arc;
use teloxide::{prelude::*};
use anyhow::Result;
use futures::stream::{StreamExt};
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct Params {
    msg: String,
}

pub async fn start_http(token: String) -> std::io::Result<()> {

    async fn push_message(tg: Arc<Bot>, msg: String)-> Result<()>  {
        let db = Arc::new(Mutex::new(db::Database::open()?));
        let db_clone = db.clone();
        let channels ={
            let db_lock = db_clone.lock().await;
            db_lock.get_auth_channel()?
        };
        
        futures::stream::iter(channels)
        .for_each_concurrent(None, |channel| {
            let tg = tg.clone();
            let msg = msg.clone();
            let db_clone = db.clone();
            let channel_id = channel.channel_id;

            async move {
                match tg.send_message(channel_id.clone(), &msg).await {
                    Ok(_) => {
                        println!("send message success， {}", channel_id);
                        let msg = msg.clone();
                        let db_lock = db_clone.lock().await;
                        match channel_id.clone().parse::<i64>() {
                            Ok(n) => db_lock.mark_push_message(n, msg, "msg".to_string()).unwrap(),
                            Err(e) => println!("convert failed {}", e)
                        };
                        ()
                    },
                    Err(e) => eprintln!("Error sending message: {:?}", e),
                }
            }
        })
        .await;
        drop(db);
        Ok(())
    }

    async fn index(info: web::Query<Params>, tg: Arc<Bot>,) -> impl Responder {
        println!("{}", info.msg);
        match push_message(tg, info.msg.clone()).await {
            Ok(()) =>  HttpResponse::Ok().body(format!("send message: {}", info.msg)),
            Err(_) => HttpResponse::InternalServerError().finish()
        }
    }

    println!("httpServer start");
    let bot = bot::MyBot::new(&token).await.expect("Error creating bot");

    HttpServer::new(move || {
        let tg_clone = bot.tg.clone();

        App::new().route("/", web::get().to(move |info| index(info, tg_clone.clone())))
    })
    .bind("127.0.0.1:6789")?
    .run()
    .await
}