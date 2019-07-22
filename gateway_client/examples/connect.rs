#![feature(async_await, await_macro)]
extern crate tokio;
extern crate discord_next;
extern crate dotenv;

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok();

    let conn = discord_next::Connection::connect(std::env::var("DISCORD_BOT_TOKEN").expect("$DISCORD_BOT_TOKEN must be set to run the example")).await;
    match conn{
        Ok(_) => println!("conn built"),
        Err(e) => println!("snafu: {}",e),
    }
}