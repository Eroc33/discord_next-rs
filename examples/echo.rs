#![feature(async_await, await_macro)]
extern crate tokio;
extern crate discord_next;
extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate serde_derive;

use tokio::prelude::*;

#[derive(Deserialize, Debug)]
struct EnvVars{
    #[serde(rename="discord_bot_token")]
    bot_token: String,
}

const ACTIVATOR: &str = "!echo";

fn main(){
    dotenv::dotenv().ok();
    
    let vars = envy::from_env::<EnvVars>().unwrap();

    tokio::run_async(async move {
        let conn = await!(discord_next::Connection::new(vars.bot_token.clone()));
        let conn = match conn{
            Ok(conn) => {
                println!("conn built");
                conn
            },
            Err(e) => {
                println!("snafu: {}",e);
                return;
            }
        };
        let res = await!(conn.run(async move |event,client|{
            eprintln!("event: {:?}:", event);
            match event{
                discord_next::model::ReceivableEvent::MessageCreate(msg) => {
                    if msg.content.starts_with(ACTIVATOR) {
                        let cmd = &msg.content[ACTIVATOR.len()..].trim();
                        await!(client.send_message(msg.channel_id,discord_next::NewMessage{content: (*cmd).into(), ..Default::default()}))?;
                    }
                }
                _other => {}
            }
            Ok(())
        }));
        eprintln!("Bot closed, res: {:?}",res);
    });
}