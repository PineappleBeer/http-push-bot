use teloxide::{prelude::*, dispatching::DefaultKey, utils::command::{BotCommands}};
use crate::*;
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug)]
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Auth(String),
    #[command(description = "unkonw command")]
    Other(String)
}

pub struct MyBot {
    pub dispatcher: Dispatcher<Arc<Bot>, anyhow::Error, DefaultKey>,
    pub tg: Arc<Bot>,
}

impl MyBot {
    pub async fn new(token: &String) -> Result<Self> {
        let tg = Arc::new(Bot::new(token));
        tg.set_my_commands(Command::bot_commands()).await?;

        let handler = Update::filter_message().branch(
            dptree::filter(|msg: Message| {
                msg.from()
                    .map(|_| true)
                    .unwrap_or_default()
            })
            .filter_command::<Command>()
            .endpoint(handle_command),
        );

        let dispatcher = Dispatcher::builder(tg.clone(), handler)
            .dependencies(dptree::deps![tg.clone()])
            .default_handler(|upd| async move {
                warn!("unhandled update: {:?}", upd);
            })
            .error_handler(LoggingErrorHandler::with_custom_text(
                "an error has occurred in the dispatcher",
            ))
            .build();

        let my_bot = MyBot {
            dispatcher,
            tg: tg.clone(),
        };
        Ok(my_bot)
    }

    pub fn spawn(
        mut self,
    ) -> (
        tokio::task::JoinHandle<()>,
        teloxide::dispatching::ShutdownToken,
    ) {
        let shutdown_token = self.dispatcher.shutdown_token();
        (
            tokio::spawn(async move { self.dispatcher.dispatch().await }),
            shutdown_token,
        )
    }
}

async fn handle_command( bot:  Arc<Bot>, msg: Message, cmd: Command) -> Result<()> {
    let db = db::Database::open()?;
    let chat_id = msg.chat.id;
    println!("{:?}", cmd);
    match cmd {
        Command::Help => {
            let text = "help command".to_string();
            db.mark_push_message(chat_id.0, text.clone(), "help".to_string()).unwrap();
            bot.send_message(msg.chat.id, text).await?
        },
        Command::Auth(key) => {
            let auth_res = crate::auth::auth(key);
            let text =  if auth_res {
                db.mark_auth_channel(msg.chat.id.0, msg.chat.username().unwrap_or("no_name"), "default_type".to_string()).unwrap();
                "auth success".to_string()
            } else {
                "auth failed".to_string()
            };
            db.mark_push_message(chat_id.0, text.clone(), "auth".to_string()).unwrap();
            bot.send_message(msg.chat.id, text.clone()).await?
        }
        Command::Other(_cmd) => {
            let text = "other command".to_string();
            db.mark_push_message(chat_id.0, text.clone(), "other".to_string()).unwrap();
            println!("{:?}", db.get_auth_channel().unwrap());
            bot.send_message(msg.chat.id, text.clone()).await?
        }
    };
    drop(db);
    Ok(())
}
