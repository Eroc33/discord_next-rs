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
    channel_id: discord_next::model::ChannelId,
}

fn main(){
    dotenv::dotenv().ok();
    
    let vars = envy::from_env::<EnvVars>().unwrap();

    tokio::run_async(async {
        let conn = await!(discord_next::Connection::new(vars.bot_token.clone()));
        match conn{
            Ok(_) => println!("conn built"),
            Err(e) => {
                println!("snafu: {}",e);
                return;
            }
        };
        let client = discord_next::Client::new(vars.bot_token);
        let res = await!(client.send_message(vars.channel_id,discord_next::NewMessage{content:"Message test".into(),..Default::default()}));
        match res{
            Ok(_) => println!("message sent"),
            Err(e) => {
                println!("snafu: {}",e);
                return;
            }
        };
    });
}