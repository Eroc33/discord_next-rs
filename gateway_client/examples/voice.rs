#![feature(async_closure)]
extern crate tokio;
extern crate discord_next;
extern crate dotenv;
extern crate envy;
#[macro_use]
extern crate serde_derive;

use tracing_subscriber;

use std::sync::Arc;

#[derive(Deserialize, Debug)]
struct EnvVars{
    #[serde(rename="discord_bot_token")]
    bot_token: String,
    voice_guild_id: discord_next::model::GuildId,
    voice_channel_id: discord_next::model::ChannelId,
}

struct TestAudioStream;

impl Default for TestAudioStream{
    fn default() -> Self{
        TestAudioStream
    }
}

impl discord_next::voice::AudioStream for TestAudioStream{
    fn read_frame(&mut self, buffer: &mut [i16]) -> Result<usize,std::io::Error>{
        for i in 0..buffer.len(){
            buffer[i] = ((i16::max_value() as f32) * 0.75 * (((i as f32)/((buffer.len() - 1) as f32))*2.0*std::f32::consts::PI).sin()) as i16;
        }
        Ok(buffer.len())
    }
    fn is_stereo(&self) -> bool
    {
        false
    }
}

const ACTIVATOR: &str = "!clip";

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok();
    
    let vars = Arc::new(envy::from_env::<EnvVars>().unwrap());

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

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

    let res: Result<(),discord_next::Error> = conn.run(move |conn, event, _client|{
        let voice_connector = discord_next::voice::VoiceConnector::from(conn);
        let vars = vars.clone();
        async move{
            println!("event: {:?}:", event);
            match event{
                discord_next::model::ReceivableEvent::MessageCreate(msg) => {
                    if msg.content.starts_with(ACTIVATOR) {
                        let voice_conn_fut = voice_connector.connect(vars.voice_guild_id, Some(vars.voice_channel_id));

                        tokio::spawn(async move{
                            let voice_conn = voice_conn_fut.await.unwrap();
                            println!("Starting clip");
                            voice_conn.run(discord_next::voice::ffmpeg::FfmpegStream::open("test.ogg",None,false).unwrap()).await;
                            println!("Voice conn completed");
                        });
                    }
                }
                _other => {}
            }
            Result::<(),discord_next::Error>::Ok(())
        }
    }).await;
    println!("Bot closed, res: {:?}",res);
}