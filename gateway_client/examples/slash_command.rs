use discord_next_model::{
    AllowedMentions, ApplicationCommandOption, ApplicationCommandOptionType,
    ApplicationCommandValue, ApplicationId, InteractionApplicationCommandCallbackData,
    InteractionCallbackType, InteractionResponse, NewApplicationCommand, Snowflake,
};

extern crate discord_next;
extern crate dotenv;
extern crate envy;
extern crate tokio;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Debug)]
struct EnvVars {
    #[serde(rename = "discord_bot_token")]
    bot_token: String,
    application_id: ApplicationId,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let vars = envy::from_env::<EnvVars>().unwrap();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let conn = discord_next::Connection::connect(vars.bot_token.clone()).await;
    let conn = match conn {
        Ok(conn) => {
            println!("conn built");
            conn
        }
        Err(e) => {
            println!("snafu: {}", e);
            return;
        }
    };
    let client = discord_next::rest_client::Client::new(vars.bot_token);
    let existing_commands = client
        .get_application_commands(vars.application_id)
        .await
        .unwrap();
    for cmd in existing_commands {
        client
            .delete_application_command(vars.application_id, cmd.id)
            .await
            .unwrap();
    }
    client
        .create_application_command(
            vars.application_id,
            NewApplicationCommand {
                application_id: vars.application_id.clone(),
                guild_id: None,
                name: "echo".to_string(),
                description: "Echo some text".to_string(),
                options: Some(vec![ApplicationCommandOption {
                    typ: ApplicationCommandOptionType::String,
                    name: "text".to_string(),
                    description: "The text to echo".to_string(),
                    required: Some(true),
                    choices: None,
                    options: None,
                }]),
                default_permission: Some(true),
            },
        )
        .await
        .unwrap();

    let res: Result<(), discord_next::Error> = conn
        .run(
            move |_conn, event, client: discord_next::rest_client::Client| async move {
                println!("event: {:?}:", event);
                match event {
                    discord_next::model::ReceivableEvent::InteractionCreate(interaction) => {
                        if let Some(interaction_data) = &interaction.data {
                            if interaction_data.name == "echo" {
                                let value = interaction_data
                                    .options
                                    .iter()
                                    .flatten()
                                    .find(|opt| opt.name == "text")
                                    .and_then(|opt| opt.value.clone());
                                let response = if let Some(ApplicationCommandValue::String(value)) =
                                    value
                                {
                                    InteractionResponse {
                                        typ: InteractionCallbackType::ChannelMessageWithSource,
                                        data: Some(InteractionApplicationCommandCallbackData {
                                            tts: None,
                                            content: Some(value),
                                            embeds: None,
                                            allowed_mentions: Some(AllowedMentions::none()),
                                            flags: None,
                                            components: None,
                                        }),
                                    }
                                } else {
                                    InteractionResponse {
                                        typ: InteractionCallbackType::ChannelMessageWithSource,
                                        data: Some(InteractionApplicationCommandCallbackData {
                                            tts: None,
                                            content: Some(
                                                "Expected a string argument \"text\"".to_string(),
                                            ),
                                            embeds: None,
                                            allowed_mentions: None,
                                            flags: None,
                                            components: None,
                                        }),
                                    }
                                };
                                client
                                    .create_interaction_response(&interaction, response)
                                    .await?;
                            }
                        }
                    }
                    _other => {}
                }
                Result::<(), discord_next::Error>::Ok(())
            },
        )
        .await;
    println!("Bot closed, res: {:?}", res);
}
