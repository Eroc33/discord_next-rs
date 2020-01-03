use super::*;
use crate::Payload;

use serde::{Deserialize,Serialize};
use log::warn;
use std::convert::TryFrom;

pub mod opcode{
    pub const DISPATCH: u64 = 0;
    pub const HEARTBEAT: u64 = 1;
    pub const IDENTIFY: u64 = 2;
    pub const STATUS_UPDATE: u64 = 3;
    pub const VOICE_STATE_UPDATE: u64 = 4;
    //5 is not documented
    pub const RESUME: u64 = 6;
    pub const RECONNECT: u64 = 7;
    pub const REQUEST_GUILD_MEMBERS: u64 = 8;
    pub const INVALID_SESSION: u64 = 9;
    pub const HELLO: u64 = 10;
    pub const HEARTBEAT_ACK: u64 = 11;
}

pub fn opcode_name(opcode: u64) -> &'static str{
    match opcode{
        opcode::DISPATCH => "DISPATCH",
        opcode::HEARTBEAT => "HEARTBEAT",
        opcode::IDENTIFY => "IDENTIFY",
        opcode::STATUS_UPDATE => "STATUS_UPDATE",
        opcode::VOICE_STATE_UPDATE => "VOICE_STATE_UPDATE",
        opcode::RESUME => "RESUME",
        opcode::RECONNECT => "RECONNECT",
        opcode::REQUEST_GUILD_MEMBERS => "REQUEST_GUILD_MEMBERS",
        opcode::INVALID_SESSION => "INVALID_SESSION",
        opcode::HELLO => "HELLO",
        opcode::HEARTBEAT_ACK => "HEARTBEAT_ACK",
        _other=> "unknown",
    }
}

pub fn known_opcode(opcode: u64) -> bool{
    match opcode{
        opcode::DISPATCH |
        opcode::HEARTBEAT |
        opcode::IDENTIFY |
        opcode::STATUS_UPDATE |
        opcode::VOICE_STATE_UPDATE |
        opcode::RESUME |
        opcode::RECONNECT |
        opcode::REQUEST_GUILD_MEMBERS |
        opcode::INVALID_SESSION |
        opcode::HELLO |
        opcode::HEARTBEAT_ACK => true,
        _other=> false,
    }
}

impl Payload{
    pub fn try_from_command<P: Into<GatewayCommand>>(payload: P) -> Result<Self,serde_json::Error>{
        use self::GatewayCommand::*;
        let payload = payload.into();

        let (op,payload) = match payload{
            Heartbeat(heartbeat) => (opcode::HEARTBEAT,serde_json::to_value(heartbeat.last_seq)?),
            Identify(identify) => (opcode::IDENTIFY,serde_json::to_value(identify)?),
            StatusUpdate(status_update) => (opcode::STATUS_UPDATE,serde_json::to_value(status_update)?),
            #[cfg(feature="voice")]
            VoiceStateUpdate(voice_status_update) => (opcode::VOICE_STATE_UPDATE,serde_json::to_value(voice_status_update)?),
            Resume(resume) => (opcode::RESUME,serde_json::to_value(resume)?),
            RequestGuildMembers(request_guild_members) => (opcode::REQUEST_GUILD_MEMBERS,serde_json::to_value(request_guild_members)?),
        };
        Ok(Payload{
            op,
            d: payload,
            s: None,
            t: None,
        })
    }
}

#[macro_export]
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
                    other => panic!("{:?} was not a {}",other,stringify!($wrapped)),
                }
            }
        }
    };
}

#[derive(Debug)]
pub enum GatewayCommand{
    Heartbeat(Heartbeat),
    Identify(Identify),
    StatusUpdate(StatusUpdate),
    #[cfg(feature="voice")]
    VoiceStateUpdate(VoiceStateUpdate),
    Resume(Resume),
    RequestGuildMembers(RequestGuildMembers),
}

wrapping_from!(GatewayCommand,Heartbeat,expect_heartbeat);
wrapping_from!(GatewayCommand,Identify,expect_identify);
wrapping_from!(GatewayCommand,StatusUpdate,expect_status_update);
#[cfg(feature="voice")]
wrapping_from!(GatewayCommand,VoiceStateUpdate,expect_voice_state_update);
wrapping_from!(GatewayCommand,Resume,expect_resume);
wrapping_from!(GatewayCommand,RequestGuildMembers,expect_request_guild_members);

#[cfg(feature="voice")]
#[derive(Debug,Serialize)]
pub struct VoiceStateUpdate{
    //id of the guild
    pub guild_id: GuildId,
    //id of the voice channel client wants to join (null if disconnecting)
    pub channel_id: Option<ChannelId>,
    //is the client muted
    pub self_mute: bool,
    //is the client deafened
    pub self_deaf: bool,
}

#[derive(Debug,Serialize)]
pub struct StatusUpdate{
    //unix time (in milliseconds) of when the client went idle, or null if the client is not idle
    since: Option<u64>,
    // The user's new activity
    game: Option<Activity>,
    //the user's new status
    status:	String,
    //whether or not the client is afk
    afk: bool,
}

#[derive(Debug,Serialize)]
pub struct RequestGuildMembers{
    //id of the guild(s) to get members for
    guild_id: Vec<GuildId>,
    //string that username starts with, or an empty string to return all members
    query: String,
    //maximum number of members to send or 0 to request all members matched
    limit: u64,
}

#[derive(Debug,Serialize)]
pub struct Resume{
    //session token
    token: String,
    session_id: String,
    //last sequence number received
    seq: u64,
}

#[derive(Debug)]
pub enum GatewayEvent{
    Hello(Hello),
    ReceivableEvent(ReceivableEvent),
    HeartbeatAck,
    HeartbeatRequest,
    //indicates the client should reconnect
    Reconnect,
    InvalidSession(InvalidSession),
}

impl TryFrom<Payload> for GatewayEvent{
    type Error = crate::payload::FromPayloadError;
    fn try_from(payload: Payload) -> Result<Self, Self::Error>
    {
        Ok(match payload.op{
            opcode::DISPATCH  => ReceivableEvent::from_payload(payload)?.into(),
            opcode::HEARTBEAT  => GatewayEvent::HeartbeatRequest,
            opcode::RECONNECT  => GatewayEvent::Reconnect,
            opcode::INVALID_SESSION  => GatewayEvent::InvalidSession(InvalidSession{resumable: serde_json::from_value(payload.d)?}),
            opcode::HELLO => serde_json::from_value::<Hello>(payload.d)?.into(),
            opcode::HEARTBEAT_ACK => GatewayEvent::HeartbeatAck,
            //TODO: just ignore send only ops rather than panic?
            other if known_opcode(other) => panic!("should never recieve `{}` payload", opcode_name(other)),
            other => Err(crate::payload::FromPayloadError::UnknownOpcode(other))?,
        })
    }
}

wrapping_from!(GatewayEvent,Hello,expect_hello);
wrapping_from!(GatewayEvent,ReceivableEvent,expect_event);
wrapping_from!(GatewayEvent,InvalidSession,expect_invalid_session);

#[derive(Debug)]
pub struct InvalidSession{
    resumable: bool,
}

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
    //Holds unkown events for limited forwards compatibility
    Unknown{name: String, value: serde_json::Value}
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


//TODO: replace this mess with better macros or codegen if possible
impl ReceivableEvent{
    fn from_payload(payload: Payload) -> Result<Self,serde_json::Error>{
        macro_rules! impl_recv_event_from_payload {
            ($payload_expr:expr => {
                $($name:expr => $variant:tt,)*
            }) => {{
                //ensure it's only evaluated once
                let payload = $payload_expr;
                match payload.t {
                    Some(s) => match s.as_str(){
                        $(
                            $name => Ok(ReceivableEvent::$variant(serde_json::from_value(payload.d)?)),
                        )*
                        name => {
                            warn!("Unknown dispatch event: {}",name);
                            Ok(ReceivableEvent::Unknown{name: name.into(), value: payload.d})
                        }
                    }
                    None => panic!("Event payload should always have a name"),
                }
            }};
        }
        impl_recv_event_from_payload!{
            payload => {
                "READY" => Ready,
                "RESUMED" => Resumed,
                "CHANNEL_CREATE" => ChannelCreate,
                "CHANNEL_UPDATE" => ChannelUpdate,
                "CHANNEL_DELETE" => ChannelDelete,
                "CHANNEL_PINS_UPDATE" => ChannelPinsUpdate,
                "GUILD_CREATE" => GuildCreate,
                "GUILD_UPDATE" => GuildUpdate,
                "GUILD_DELETE" => GuildDelete,
                "GUILD_BAN_ADD" => GuildBanAdd,
                "GUILD_BAN_REMOVE" => GuildBanRemove,
                "GUILD_EMOJIS_UPDATE" => GuildEmojisUpdate,
                "GUILD_INTEGRATIONS_UPDATE" => GuildIntegrationsUpdate,
                "GUILD_MEMBER_ADD" => GuildMemberAdd,
                "GUILD_MEMBER_REMOVE" => GuildMemberRemove,
                "GUILD_MEMBER_UPDATE" => GuildMemberUpdate,
                "GUILD_MEMBERS_CHUNK" => GuildMembersChunk,
                "GUILD_ROLE_CREATE" => GuildRoleCreate,
                "GUILD_ROLE_UPDATE" => GuildRoleUpdate,
                "GUILD_ROLE_DELETE" => GuildRoleDelete,
                "MESSAGE_CREATE" => MessageCreate,
                "MESSAGE_UPDATE" => MessageUpdate,
                "MESSAGE_DELETE" => MessageDelete,
                "MESSAGE_DELETE_BULK" => MessageDeleteBulk,
                "MESSAGE_REACTION_ADD" => MessageReactionAdd,
                "MESSAGE_REACTION_REMOVE" => MessageReactionRemove,
                "MESSAGE_REACTION_REMOVE_ALL" => MessageReactionRemoveAll,
                "PRESENCE_UPDATE" => PresenceUpdate,
                "TYPING_START" => TypingStart,
                "USER_UPDATE" => UserUpdate,
                "VOICE_STATE_UPDATE" => VoiceStateUpdate,
                "VOICE_SERVER_UPDATE" => VoiceServerUpdate,
                "WEBHOOKS_UPDATE" => WebhooksUpdate,
            }
        }
    }
}

#[derive(Debug)]
pub struct Heartbeat{
    pub last_seq: Option<u64>,
}

#[derive(Debug,Deserialize,Serialize)]
pub struct Hello{
    pub heartbeat_interval: u64,
    pub _trace: Vec<String>
}


#[derive(Debug,Deserialize,Serialize)]
pub struct Identify{
    //authentication token
    pub token: String,
    pub properties: ConnectionProperties,
    //whether this connection supports compression of packets
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub compress: Option<bool>,
    //value between 50 and 250, total number of members where the gateway will stop sending offline members in the guild member list
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub large_threshold: Option<u8>,
    //used for Guild Sharding
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub shard: Option<[u8;2]>,
    //initial presence information
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

#[derive(Debug)]
pub enum CloseCode{
    ///We're not sure what went wrong. Try reconnecting?
    UnknownError,
    ///You sent an invalid Gateway opcode or an invalid payload for an opcode. Don't do that!
	UnknownOpcode,
    ///You sent an invalid payload to us. Don't do that!
	DecodeError,
    ///You sent us a payload prior to identifying.
	NotAuthenticated,
    ///The account token sent with your identify payload is incorrect.
	AuthenticationFailed,
    ///You sent more than one identify payload. Don't do that!
	AlreadyAuthenticated,
    ///The sequence sent when resuming the session was invalid. Reconnect and start a new session.
	InvalidSeq,
    ///Woah nelly! You're sending payloads to us too quickly. Slow it down!
	RateLimited,
    ///Your session timed out. Reconnect and start a new one.
	SessionTimeout,
    ///You sent us an invalid shard when identifying.
	InvalidShard,
    ///The session would have handled too many guilds - you are required to shard your connection in order to connect.
	ShardingRequired,
}

impl TryFrom<u16> for CloseCode{
    type Error = ();
    fn try_from(close_code: u16) -> Result<Self, Self::Error>
    {
        Ok(match close_code{
            4000 => CloseCode::UnknownError,
            4001 => CloseCode::UnknownOpcode,
            4002 => CloseCode::DecodeError,
            4003 => CloseCode::NotAuthenticated,
            4004 => CloseCode::AuthenticationFailed,
            4005 => CloseCode::AlreadyAuthenticated,
            //4006 is not documented
            4007 => CloseCode::InvalidSeq,
            4008 => CloseCode::RateLimited,
            4009 => CloseCode::SessionTimeout,
            4010 => CloseCode::InvalidShard,
            4011 => CloseCode::ShardingRequired,
            _else => return Err(()),
        })
    }
}