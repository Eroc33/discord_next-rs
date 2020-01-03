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
    channel_id: discord_next::model::ChannelId,
}

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok();
    
    let vars = envy::from_env::<EnvVars>().unwrap();

    let conn = discord_next::Connection::connect(vars.bot_token.clone()).await;
    match conn{
        Ok(_) => println!("conn built"),
        Err(e) => {
            println!("snafu: {}",e);
            return;
        }
    };
    let client = discord_next::rest_client::Client::new(vars.bot_token);
    let res = client.send_message(vars.channel_id,discord_next::rest_client::NewMessage{content:"Message test".into(),..Default::default()}).await;
    match res{
        Ok(_) => println!("message sent"),
        Err(e) => {
            println!("snafu: {}",e);
            return;
        }
    };
}