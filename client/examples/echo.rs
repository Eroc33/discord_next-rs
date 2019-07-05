#![feature(async_await, await_macro, futures_api)]
extern crate tokio;
extern crate discord_next;
extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate serde_derive;

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
        let res: Result<(),discord_next::Error> = await!(conn.run(async move |event, client|{
            println!("event: {:?}:", event);
            match event{
                discord_next::model::ReceivableEvent::MessageCreate(msg) => {
                    if msg.content.starts_with(ACTIVATOR) {
                        let cmd: String = msg.content[ACTIVATOR.len()..].trim().to_owned();
                        await!(client.send_message(msg.channel_id,discord_next::NewMessage::text(cmd)))?;
                    }
                }
                _other => {}
            }
            Ok(())
        }));
        println!("Bot closed, res: {:?}",res);
    });
}