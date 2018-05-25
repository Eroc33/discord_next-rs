use serde_json;
use std::convert::TryFrom;

#[derive(Debug,Deserialize,Serialize)]
pub struct Payload{
    //opcode for the payload
    op: u64,
    //event data
    d: serde_json::Value,
    //sequence number, used for resuming sessions and heartbeats (Only for Opcode 0)
    s: Option<u64>,
    //the event name for this payload (Only for Opcode 0)	
    t: Option<String>,
}

impl Payload{
    pub fn received_event_data(self) -> Result<ReceivablePayload,::Error>{
        Ok(match self.op{
            0  => ReceivableEvent::from_payload(self)?.into(),
            1  => unimplemented!("Op Heartbeat nyi"),
            //TODO: just ignore this rather than panic?
            2  => panic!("Should never recieve identify payloads"),
            3  => unimplemented!("Op Status Update nyi"),
            4  => unimplemented!("Op Voice State Update nyi"),
            5  => unimplemented!("Op Voice Server Ping nyi"),
            6  => unimplemented!("Op Resume nyi"),
            7  => unimplemented!("Op Reconnect nyi"),
            8  => unimplemented!("Op Request Guild Members nyi"),
            9  => unimplemented!("Op Invalid Session nyi"),
            10 => serde_json::from_value::<Hello>(self.d)?.into(),
            11 => unimplemented!("Op Heartbeat ACK nyi"),
            other => unimplemented!("Unknown op {}",other),
        })
    }
}

impl Payload{
    pub fn try_from_sendable<P: Into<SendablePayload>>(payload: P) -> Result<Self,::Error>{
        let payload = payload.into();

        let (op,payload) = match payload{
            SendablePayload::Identify(identify) => (2,serde_json::to_value(identify)?),
        };
        Ok(Payload{
            op,
            d: payload,
            s: None,
            t: None,
        })
    }
}

macro_rules! wrapping_from {
    ($wrapper: tt, $wrapped: tt, $expect_fn: ident) => {
        impl From<$wrapped> for $wrapper{
            fn from(inner: $wrapped) -> Self{
                $wrapper::$wrapped(inner)
            }
        }

        impl $wrapper{
            pub fn $expect_fn(self) -> $wrapped{
                match self{
                    $wrapper::$wrapped(inner) => inner,
                    _ => panic!("$wrapper was not a $wrapped"),
                }
            }
        }
    };
}

#[derive(Debug)]
pub enum SendablePayload{
    Identify(Identify),
}

wrapping_from!(SendablePayload,Identify,expect_identify);

#[derive(Debug)]
pub enum ReceivablePayload{
    Hello(Hello),
    ReceivableEvent(ReceivableEvent),
    //TODO: moar
}

wrapping_from!(ReceivablePayload,Hello,expect_hello);
wrapping_from!(ReceivablePayload,ReceivableEvent,expect_event);

#[derive(Debug,Deserialize)]
pub enum ReceivableEvent{
    Ready(Ready),
}

wrapping_from!(ReceivableEvent,Ready,expect_ready);

impl ReceivableEvent{
    fn name(&self) -> &'static str
    {
        match self{
            ReceivableEvent::Ready(_) => "READY",
        }
    }
    fn from_payload(payload: Payload) -> Result<Self,serde_json::Error>{
        match payload.t {
            Some(s) => match s.as_str(){
                "READY" => Ok(serde_json::from_value::<Ready>(payload.d)?.into()),
                other => panic!("Unknown named event: {:?}",other),
            }
            None => panic!("Event payload should always have a name"),
        }
    }
}

pub type Snowflake = String;

#[derive(Debug,Deserialize,Serialize)]
pub struct Hello{
    pub heartbeat_interval: u64,
    pub _trace: Vec<String>
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Identify{
    pub token: String,
    pub properties: serde_json::Value,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub compress: Option<bool>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub large_threshold: Option<u8>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub shard: Option<[u8;2]>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub presence: Option<UpdateStatus>
}

impl Identify{
    pub fn new(token: String) -> Self{
        Identify{
            token,
            properties: json!({
				"$os": ::std::env::consts::OS,
				"$browser": "Refreshed Discord library for Rust",
				"$device": "discord-next",
				"$referring_domain": "",
				"$referrer": "",
            }),
            compress: Default::default(),
            large_threshold: Default::default(),
            shard: Default::default(),
            presence: Default::default(),
        }
    }
}

#[derive(Debug,Deserialize,Serialize)]
pub enum Status{
    #[serde(rename = "online")]	Online,
    #[serde(rename = "dnd")]	DoNotDisturb,
    #[serde(rename = "idle")]	AFK,
    #[serde(rename = "invisible")]	Invisible,
    #[serde(rename = "offline")]	Offline,
}

#[derive(Debug,Deserialize,Serialize)]
pub struct UpdateStatus{
    //unix time (in milliseconds) of when the client went idle, or null if the client is not idle
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    //the user's new activity
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub game: Option<Activity>,
    //the user's new status
    pub status: Status,
    //whether or not the client is afk
    pub afk:	bool
}

impl UpdateStatus{
    fn new_basic(status: Status, afk: bool) -> Self{
        UpdateStatus{
            since: Default::default(),
            game: Default::default(),
            status,
            afk
        }
    }
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Activity{
    //the activity's name
    pub name: String,
    //activity type
    #[serde(rename = "type")]
    pub typ: u64,
    //is validated when type is 1
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    //unix timestamps for start and/or end of the game
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Timestamps>,
    //application id for the game
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub application_id: Option<Snowflake>,
    //what the player is currently doing
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    //the user's current party status
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    //information for the current party of the player
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub party:	Option<Party>,
    //images for the presence and their hover texts
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub assets:	Option<Assets>,
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Timestamps{
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Party{
    //the id of the party
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    //tuple of two integers (current_size, max_size) used to show the party's current and maximum size
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub size: Option<(u64,u64)>,
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Assets{
    //the id for a large asset of the activity, usually a snowflake
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    //text displayed when hovering over the large image of the activity
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    //the id for a small asset of the activity, usually a snowflake
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    //text displayed when hovering over the small image of the activity
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct Ready{
    //gateway protocol version
    pub v: u64,
    //user object	information about the user including email
    pub user: User,	
    //the direct message channels the user is in
    pub private_channels: Vec<Channel>,
    //the guilds the user is in
    pub guilds: Vec<UnavailableGuild>,
    //used for resuming connections
    pub session_id:	String,
    //used for debugging
    pub _trace: Vec<String>,
}

#[derive(Debug,Deserialize)]
pub struct UnavailableGuild{
    pub id: Snowflake,
    pub unavailable: bool
}

#[derive(Debug,Deserialize)]
pub struct Channel{
    //the id of this channel
    pub id: Snowflake,
    //the type of channel
    #[serde(rename = "type")]
    pub typ: u64,
    //the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    //sorting position of the channel
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    //explicit permission overwrites for members and roles
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    pub permission_overwrites: Vec<Overwrite>,
    //the name of the channel (2-100 characters)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    //the channel topic (0-1024 characters)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    //if the channel is nsfw
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    //the id of the last message sent in this channel (may not point to an existing or valid message)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<Snowflake>,
    //the bitrate (in bits) of the voice channel
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u64>,
    //the user limit of the voice channel
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u64>,
    //the recipients of the DM
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    pub recipients: Vec<User>,
    //icon hash
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    //id of the DM creator
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Snowflake>,
    //application id of the group DM creator if it is bot-created
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub application_id: Option<Snowflake>,
    //id of the parent category for a channel
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Snowflake>,
    //when the last pinned message was pinned
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub last_pin_timestamp: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct Overwrite{
    //role or user id
    pub id: Snowflake,
    //either "role" or "member"
    #[serde(rename = "type")]
    pub typ: String,
    //permission bit set
    pub allow: u64,
    //permission bit set
    pub deny: u64,
}

#[derive(Debug,Deserialize)]
pub struct User{
    //the user's id
    pub id: Snowflake,
    //the user's username, not unique across the platform
    pub username: String,
    //the user's 4-digit discord-tag
    pub discriminator: String,
    //the user's avatar hash
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    //whether the user belongs to an OAuth2 application
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    //whether the user has two factor enabled on their account
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub mfa_enabled: Option<bool>,
    //whether the email on this account has been verified
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    //the user's email
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}