#![feature(async_await, await_macro)]
extern crate tokio;
extern crate discord_next;
extern crate dotenv;

use tokio::prelude::*;

fn main(){
    dotenv::dotenv().ok();

    tokio::run_async(async {
        let conn = await!(discord_next::connect_to_gateway(std::env::var("DISCORD_BOT_TOKEN").expect("$DISCORD_BOT_TOKEN must be set to run the example").into()));
        match conn{
            Ok(_) => println!("conn built"),
            Err(e) => println!("snafu: {}",e),
        }
    });
}