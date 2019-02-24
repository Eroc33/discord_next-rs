#![feature(async_await, await_macro, futures_api)]
extern crate tokio;
extern crate discord_next;
extern crate dotenv;

fn main(){
    dotenv::dotenv().ok();

    tokio::run_async(async {
        let conn = await!(discord_next::Connection::new(std::env::var("DISCORD_BOT_TOKEN").expect("$DISCORD_BOT_TOKEN must be set to run the example")));
        match conn{
            Ok(_) => println!("conn built"),
            Err(e) => println!("snafu: {}",e),
        }
    });
}