use super::*;

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
    pub fn received_event_data(self) -> Result<ReceivablePayload,crate::Error>{
        Ok(match self.op{
            0  => ReceivableEvent::from_payload(self)?.into(),
            1  => ReceivablePayload::HeartbeatRequest,
            //TODO: just ignore send only ops rather than panic?
            2  => panic!("Should never recieve identify payloads"),
            3  => panic!("Should never recieve status update payloads"),
            4  => panic!("Should never recieve voice update payloads"),
            5  => panic!("Should never recieve voice ping payloads"),
            6  => panic!("Should never recieve ping payloads"),
            7  => unimplemented!("Op Reconnect nyi"),
            8  => panic!("Should never recieve request guild members payloads"),
            9  => unimplemented!("Op Invalid Session nyi"),
            10 => serde_json::from_value::<Hello>(self.d)?.into(),
            11 => ReceivablePayload::HeartbeatAck,
            other => unimplemented!("Unknown op {}",other),
        })
    }
}

impl Payload{
    pub fn try_from_sendable<P: Into<SendablePayload>>(payload: P) -> Result<Self,crate::Error>{
        let payload = payload.into();

        let (op,payload) = match payload{
            SendablePayload::Identify(identify) => (2,serde_json::to_value(identify)?),
            SendablePayload::Heartbeat(_) => (1,serde_json::Value::Null),
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
    Heartbeat(Heartbeat),
}

wrapping_from!(SendablePayload,Identify,expect_identify);
wrapping_from!(SendablePayload,Heartbeat,expect_heartbeat);

#[derive(Debug)]
pub enum ReceivablePayload{
    Hello(Hello),
    ReceivableEvent(ReceivableEvent),
    HeartbeatAck,
    HeartbeatRequest,
    //TODO: moar
}

wrapping_from!(ReceivablePayload,Hello,expect_hello);
wrapping_from!(ReceivablePayload,ReceivableEvent,expect_event);

#[derive(Debug,Deserialize)]
pub enum ReceivableEvent{
    //contains the initial state information
    Ready(Ready),
    //response to Resume
    Resumed(Resumed),
    //new channel created
    ChannelCreate(Channel),
    //channel was updated
    ChannelUpdate(Channel),
    //channel was deleted
    ChannelDelete(Channel),
    //message was pinned or unpinned
    ChannelPinsUpdate(ChannelPinsUpdate),
    //lazy-load for unavailable guild, guild became available, or user joined a new guild
    GuildCreate(Guild),
    //guild was updated
    GuildUpdate(Guild),
    //guild became unavailable, or user left/was removed from a guild
    GuildDelete(UnavailableGuild),
    //user was banned from a guild
    GuildBanAdd(GuildBanAdd),
    //user was unbanned from a guild
    GuildBanRemove(GuildBanRemove),
    //guild emojis were updated
    GuildEmojisUpdate(GuildEmojisUpdate),
    //guild integration was updated
    GuildIntegrationsUpdate(GuildIntegrationsUpdate),
    //new user joined a guild
    GuildMemberAdd(GuildMemberAdd),
    //user was removed from a guild
    GuildMemberRemove(GuildMemberRemove),
    //guild member was updated
    GuildMemberUpdate(GuildMemberUpdate),
    //response to Request Guild Members
    GuildMembersChunk(GuildMembersChunk),
    //guild role was created
    GuildRoleCreate(GuildRoleCreate),
    //guild role was updated
    GuildRoleUpdate(GuildRoleUpdate),
    //guild role was deleted
    GuildRoleDelete(GuildRoleDelete),
    //message was created
    MessageCreate(Message),
    //message was edited
    MessageUpdate(MessageUpdate),
    //message was deleted
    MessageDelete(MessageDelete),
    //multiple messages were deleted at once
    MessageDeleteBulk(MessageDeleteBulk),
    //user reacted to a message
    MessageReactionAdd(MessageReactionAdd),
    //user removed a reaction from a message
    MessageReactionRemove(MessageReactionRemove),
    //all reactions were explicitly removed from a message
    MessageReactionRemoveAll(MessageReactionRemoveAll),
    //user was updated
    PresenceUpdate(PresenceUpdate),
    //user started typing in a channel
    TypingStart(TypingStart),
    //properties about the user changed
    UserUpdate(User),
    //someone joined, left, or moved a voice channel
    VoiceStateUpdate(VoiceState),
    //guild's voice server was updated
    VoiceServerUpdate(VoiceServerUpdate),
    //guild channel webhook was created, update, or deleted
    WebhooksUpdate(WebhooksUpdate),
}

wrapping_from!(ReceivableEvent,Ready,expect_ready);
wrapping_from!(ReceivableEvent,Resumed,expect_resumed);
wrapping_from!(ReceivableEvent,ChannelPinsUpdate,expect_channel_pins_update);
wrapping_from!(ReceivableEvent,GuildBanAdd,expect_guild_ban_add);
wrapping_from!(ReceivableEvent,GuildBanRemove,expect_guild_ban_remove);
wrapping_from!(ReceivableEvent,GuildEmojisUpdate,expect_guild_emojis_update);
wrapping_from!(ReceivableEvent,GuildIntegrationsUpdate,expect_guild_integrations_update);
wrapping_from!(ReceivableEvent,GuildMemberAdd,expect_guild_member_add);
wrapping_from!(ReceivableEvent,GuildMemberRemove,expect_guild_member_remove);
wrapping_from!(ReceivableEvent,GuildMemberUpdate,expect_guild_member_update);
wrapping_from!(ReceivableEvent,GuildMembersChunk,expect_guild_members_chunk);
wrapping_from!(ReceivableEvent,GuildRoleCreate,expect_guild_role_create);
wrapping_from!(ReceivableEvent,GuildRoleUpdate,expect_guild_role_update);
wrapping_from!(ReceivableEvent,GuildRoleDelete,expect_guild_role_delete);
wrapping_from!(ReceivableEvent,MessageUpdate,expect_message_update);
wrapping_from!(ReceivableEvent,MessageDelete,expect_message_delete);
wrapping_from!(ReceivableEvent,MessageDeleteBulk,expect_message_delete_bulk);
wrapping_from!(ReceivableEvent,MessageReactionAdd,expect_message_reaction_add);
wrapping_from!(ReceivableEvent,MessageReactionRemove,expect_message_reaction_remove);
wrapping_from!(ReceivableEvent,MessageReactionRemoveAll,expect_message_reaction_remove_all);
wrapping_from!(ReceivableEvent,PresenceUpdate,expect_presence_update);
wrapping_from!(ReceivableEvent,TypingStart,expect_typing_start);
wrapping_from!(ReceivableEvent,VoiceServerUpdate,expect_voice_server_update);
wrapping_from!(ReceivableEvent,WebhooksUpdate,expect_webhooks_update);

//TODO: replace this cyber-chef copypaste mess with serde if possible
impl ReceivableEvent{
    fn from_payload(payload: Payload) -> Result<Self,serde_json::Error>{
        match payload.t {
            Some(s) => match s.as_str(){
                "READY" => Ok(ReceivableEvent::Ready(serde_json::from_value(payload.d)?)),
                "RESUMED" => Ok(ReceivableEvent::Resumed(serde_json::from_value(payload.d)?)),
                "CHANNEL_CREATE" => Ok(ReceivableEvent::ChannelCreate(serde_json::from_value(payload.d)?)),
                "CHANNEL_UPDATE" => Ok(ReceivableEvent::ChannelUpdate(serde_json::from_value(payload.d)?)),
                "CHANNEL_DELETE" => Ok(ReceivableEvent::ChannelDelete(serde_json::from_value(payload.d)?)),
                "CHANNEL_PINS_UPDATE" => Ok(ReceivableEvent::ChannelPinsUpdate(serde_json::from_value(payload.d)?)),
                "GUILD_CREATE" => Ok(ReceivableEvent::GuildCreate(serde_json::from_value(payload.d)?)),
                "GUILD_UPDATE" => Ok(ReceivableEvent::GuildUpdate(serde_json::from_value(payload.d)?)),
                "GUILD_DELETE" => Ok(ReceivableEvent::GuildDelete(serde_json::from_value(payload.d)?)),
                "GUILD_BAN_ADD" => Ok(ReceivableEvent::GuildBanAdd(serde_json::from_value(payload.d)?)),
                "GUILD_BAN_REMOVE" => Ok(ReceivableEvent::GuildBanRemove(serde_json::from_value(payload.d)?)),
                "GUILD_EMOJIS_UPDATE" => Ok(ReceivableEvent::GuildEmojisUpdate(serde_json::from_value(payload.d)?)),
                "GUILD_INTEGRATIONS_UPDATE" => Ok(ReceivableEvent::GuildIntegrationsUpdate(serde_json::from_value(payload.d)?)),
                "GUILD_MEMBER_ADD" => Ok(ReceivableEvent::GuildMemberAdd(serde_json::from_value(payload.d)?)),
                "GUILD_MEMBER_REMOVE" => Ok(ReceivableEvent::GuildMemberRemove(serde_json::from_value(payload.d)?)),
                "GUILD_MEMBER_UPDATE" => Ok(ReceivableEvent::GuildMemberUpdate(serde_json::from_value(payload.d)?)),
                "GUILD_MEMBERS_CHUNK" => Ok(ReceivableEvent::GuildMembersChunk(serde_json::from_value(payload.d)?)),
                "GUILD_ROLE_CREATE" => Ok(ReceivableEvent::GuildRoleCreate(serde_json::from_value(payload.d)?)),
                "GUILD_ROLE_UPDATE" => Ok(ReceivableEvent::GuildRoleUpdate(serde_json::from_value(payload.d)?)),
                "GUILD_ROLE_DELETE" => Ok(ReceivableEvent::GuildRoleDelete(serde_json::from_value(payload.d)?)),
                "MESSAGE_CREATE" => Ok(ReceivableEvent::MessageCreate(serde_json::from_value(payload.d)?)),
                "MESSAGE_UPDATE" => Ok(ReceivableEvent::MessageUpdate(serde_json::from_value(payload.d)?)),
                "MESSAGE_DELETE" => Ok(ReceivableEvent::MessageDelete(serde_json::from_value(payload.d)?)),
                "MESSAGE_DELETE_BULK" => Ok(ReceivableEvent::MessageDeleteBulk(serde_json::from_value(payload.d)?)),
                "MESSAGE_REACTION_ADD" => Ok(ReceivableEvent::MessageReactionAdd(serde_json::from_value(payload.d)?)),
                "MESSAGE_REACTION_REMOVE" => Ok(ReceivableEvent::MessageReactionRemove(serde_json::from_value(payload.d)?)),
                "MESSAGE_REACTION_REMOVE_ALL" => Ok(ReceivableEvent::MessageReactionRemoveAll(serde_json::from_value(payload.d)?)),
                "PRESENCE_UPDATE" => Ok(ReceivableEvent::PresenceUpdate(serde_json::from_value(payload.d)?)),
                "TYPING_START" => Ok(ReceivableEvent::TypingStart(serde_json::from_value(payload.d)?)),
                "USER_UPDATE" => Ok(ReceivableEvent::UserUpdate(serde_json::from_value(payload.d)?)),
                "VOICE_STATE_UPDATE" => Ok(ReceivableEvent::VoiceStateUpdate(serde_json::from_value(payload.d)?)),
                "VOICE_SERVER_UPDATE" => Ok(ReceivableEvent::VoiceServerUpdate(serde_json::from_value(payload.d)?)),
                "WEBHOOKS_UPDATE" => Ok(ReceivableEvent::WebhooksUpdate(serde_json::from_value(payload.d)?)),
                other => panic!("Unknown named event: {:?}",other),
            }
            None => panic!("Event payload should always have a name"),
        }
    }
}

#[derive(Debug)]
pub struct Heartbeat;

#[derive(Debug,Deserialize,Serialize)]
pub struct Hello{
    pub heartbeat_interval: u64,
    pub _trace: Vec<String>
}


#[derive(Debug,Deserialize,Serialize)]
pub struct Identify{
    pub token: String,
    pub properties: ConnectionProperties,
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
            properties: Default::default(),
            compress: Default::default(),
            large_threshold: Default::default(),
            shard: Default::default(),
            presence: Default::default(),
        }
    }
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

///response to Resume
#[derive(Debug,Deserialize)]
pub struct Resumed
{
    //used for debugging
    pub _trace: Vec<String>,
}
///message was pinned or unpinned
#[derive(Debug,Deserialize)]
pub struct ChannelPinsUpdate
{
    channel_id: ChannelId,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    last_pin_timestamp: Option<DateTime<FixedOffset>>,
}
///user was banned from a guild
#[derive(Debug,Deserialize)]
pub struct GuildBanAdd
{
    guild_id: GuildId,
    user: User,
}
///user was unbanned from a guild
#[derive(Debug,Deserialize)]
pub struct GuildBanRemove
{
    guild_id: GuildId,
    user: User,
}
///guild emojis were updated
#[derive(Debug,Deserialize)]
pub struct GuildEmojisUpdate
{
    ///id of the guild
    guild_id: GuildId,
    ///array of emojis
    emojis: Vec<Emoji>,
}
///guild integration was updated
#[derive(Debug,Deserialize)]
pub struct GuildIntegrationsUpdate
{
    ///id of the guild whose integrations were updated
    guild_id: GuildId,
}
///new user joined a guild
#[derive(Debug,Deserialize)]
pub struct GuildMemberAdd
{
    ///id of the guild
    guild_id: GuildId,
    #[serde(flatten)]
    member: GuildMember,
}
///user was removed from a guild
#[derive(Debug,Deserialize)]
pub struct GuildMemberRemove
{
    ///the id of the guild
    guild_id: GuildId,
    ///the user who was removed
    user: User
}
///guild member was updated
#[derive(Debug,Deserialize)]
pub struct GuildMemberUpdate
{
    ///the id of the guild
    guild_id: GuildId,
    ///user role ids
    roles: Vec<RoleId>,
    ///the user
    user: User,
    ///nickname of the user in the guild
    nick: String,
}
///response to Request Guild Members
#[derive(Debug,Deserialize)]
pub struct GuildMembersChunk
{
    ///the id of the guild
    guild_id: GuildId,
    ///set of guild members
    members: Vec<GuildMember>,
}
///guild role was created
#[derive(Debug,Deserialize)]
pub struct GuildRoleCreate
{
    ///the id of the guild
    guild_id: GuildId,
    ///the role created
    role: Role,
}
///guild role was updated
#[derive(Debug,Deserialize)]
pub struct GuildRoleUpdate
{
    ///the id of the guild
    guild_id: GuildId,
    ///the role updated
    role: Role,
}
///guild role was deleted
#[derive(Debug,Deserialize)]
pub struct GuildRoleDelete
{
    ///id of the guild
    guild_id: GuildId,
    ///id of the role
    role_id: RoleId,
}
///message was edited
#[derive(Debug,Deserialize)]
pub struct MessageUpdate
{
    //TODO: partial Message
}
///message was deleted
#[derive(Debug,Deserialize)]
pub struct MessageDelete
{
    ///the id of the message
    pub id: MessageId,
    ///the id of the channel
    pub channel_id: ChannelId,
    ///the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
}
///multiple messages were deleted at once
#[derive(Debug,Deserialize)]
pub struct MessageDeleteBulk
{
    ///the ids of the messages
    pub ids: Vec<MessageId>,
    ///the id of the channel
    pub channel_id: ChannelId,
    ///the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
}
///user reacted to a message
#[derive(Debug,Deserialize)]
pub struct MessageReactionAdd
{
    ///the id of the user
    pub user_id: UserId,
    ///the id of the channel
    pub channel_id: ChannelId,
    ///the id of the message
    pub message_id: MessageId,
    ///the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the emoji used to react
    pub emoji: Emoji,
}
///user removed a reaction from a message
#[derive(Debug,Deserialize)]
pub struct MessageReactionRemove
{
    ///the id of the user
    pub user_id: UserId,
    ///the id of the channel
    pub channel_id: ChannelId,
    ///the id of the message
    pub message_id: MessageId,
    ///the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the emoji used to react
    pub emoji: Emoji,
}
///all reactions were explicitly removed from a message
#[derive(Debug,Deserialize)]
pub struct MessageReactionRemoveAll
{
    ///the id of the channel
    pub channel_id: ChannelId,
    ///the id of the message
    pub message_id: MessageId,
    ///the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
}
///user started typing in a channel
#[derive(Debug,Deserialize)]
pub struct TypingStart
{
    ///id of the channel
    pub channel_id: ChannelId,
    ///id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///id of the user
    pub user_id: UserId,
    ///unix time (in seconds) of when the user started typing
    pub timestamp: u64,
}
///guild's voice server was updated
#[derive(Debug,Deserialize)]
pub struct VoiceServerUpdate
{
    ///voice connection token
    pub token: String,
    ///the guild this voice server update is for
    pub guild_id: GuildId,
    ///the voice server host
    pub endpoint: String,
}
///guild channel webhook was created, update, or deleted
#[derive(Debug,Deserialize)]
pub struct WebhooksUpdate
{
    ///id of the guild
    pub guild_id: GuildId,
    ///id of the channel
    pub channel_id: ChannelId,
}