extern crate tokio;
extern crate discord_next;
extern crate dotenv;

use tokio::prelude::*;

fn main(){
    dotenv::dotenv().ok();
    tokio::runtime::run(discord_next::connect_to_gateway(std::env::var("DISCORD_BOT_TOKEN").expect("$DISCORD_BOT_TOKEN must be set to run the example").into()).map(|_conn| println!("conn built")).map_err(|e| eprintln!("snafu: {}",e)));
}