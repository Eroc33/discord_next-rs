#![feature(async_await, await_macro, async_closure)]
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

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok();
    
    let vars = envy::from_env::<EnvVars>().unwrap();

    let conn = discord_next::Connection::connect(vars.bot_token.clone()).await;
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
    let res: Result<(),discord_next::Error> = conn.run(async move |_conn, event, client: discord_next::rest_client::Client|{
        println!("event: {:?}:", event);
        match event{
            discord_next::model::ReceivableEvent::MessageCreate(msg) => {
                if msg.content.starts_with(ACTIVATOR) {
                    let cmd: String = msg.content[ACTIVATOR.len()..].trim().to_owned();
                    client.send_message(msg.channel_id,discord_next::rest_client::NewMessage::text(cmd)).await?;
                }
            }
            _other => {}
        }
        Result::<(),discord_next::Error>::Ok(())
    }).await;
    println!("Bot closed, res: {:?}",res);
}