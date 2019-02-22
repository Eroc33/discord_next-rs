use chrono::{DateTime,FixedOffset};
#[macro_use]
use serde_json;

mod ids;
pub use ids::*;
mod gateway_events;
pub use gateway_events::*;
mod embed;
pub use embed::*;
#[macro_use]
mod enum_number;

#[derive(Debug,Deserialize,Serialize)]
pub struct ConnectionProperties{
    #[serde(rename="$os")]
    pub os: String,
    #[serde(rename="$browser")]
    pub browser: String,
    #[serde(rename="$device")]
    pub device: String,
}

impl Default for ConnectionProperties{
    fn default() -> Self{
        ConnectionProperties{
            os: ::std::env::consts::OS.into(),
            browser: "discord-next-rust".into(),
            device: "discord-next-rust".into(),
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
    pub typ: ActivityType,
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
    ///secrets for Rich Presence joining and spectating
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub secrets: Option<ActivitySecrets>,
    ///whether or not the activity is an instanced game session
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub instance: Option<bool>,
    ///activity flags, describes what the payload includes
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub flags: Option<ActivityFlags>,
}

//TODO: is this actually an enum, or is it bitflags?
enum_number!(ActivityType{
    Game = 0,
    Streaming = 1,
    Listening = 2,
});

bitflags! {
    pub struct ActivityFlags: u32 {
        const INSTANCE	   = 1;
        const JOIN         = 2;
        const SPECTATE	   = 4;
        const JOIN_REQUEST = 8;
        const SYNC	       = 16;
        const PLAY	       = 32;
    }
}

impl<'de> serde::de::Deserialize<'de> for ActivityFlags{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>
    {
        Ok(ActivityFlags::from_bits_truncate(u32::deserialize(deserializer)?))
    }
}

impl serde::ser::Serialize for ActivityFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Debug,Deserialize,Serialize)]
pub struct ActivitySecrets{
    ///the secret for joining a party
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,
    ///the secret for spectating a game
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub spectate: Option<String>,
    ///the secret for a specific instanced match
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename="match")]
    pub match_secret: Option<String>,
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
pub struct UnavailableGuild{
    pub id: GuildId,
    pub unavailable: bool
}

#[derive(Debug,Deserialize)]
pub struct Channel{
    //the id of this channel
    pub id: ChannelId,
    //the type of channel
    #[serde(rename = "type")]
    pub typ: u64,
    //the id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
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
    pub last_message_id: Option<MessageId>,
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
    pub owner_id: Option<UserId>,
    //application id of the group DM creator if it is bot-created
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub application_id: Option<Snowflake>,
    //id of the parent category for a channel
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ChannelId>,
    //when the last pinned message was pinned
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
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

#[derive(Debug,Deserialize,Clone)]
pub struct User{
    //the user's id
    pub id: UserId,
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

#[derive(Debug,Deserialize)]
pub struct PartialUser{
    //the user's id
    pub id: UserId,
    //the user's username, not unique across the platform
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    //the user's 4-digit discord-tag
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
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

#[derive(Debug,Deserialize)]
pub struct Guild{
    ///guild id
    pub id: GuildId,
    ///guild name (2-100 characters)
    pub name: String,
    ///icon hash
    pub icon: Option<String>,
    ///splash hash
    pub splash: Option<String>,
    ///whether or not the user is the owner of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub owner: Option<bool>,
    ///id of owner
    pub owner_id: UserId,
    ///total permissions for the user in the guild (does not include channel overrides)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    ///voice region id for the guild
    pub region: String,
    ///id of afk channel
    pub afk_channel_id: Option<ChannelId>,
    ///afk timeout in seconds
    pub afk_timeout: u64,
    ///is this guild embeddable (e.g. widget)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub embed_enabled: Option<bool>,
    ///if not null, the channel id that the widget will generate an invite to
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub embed_channel_id: Option<ChannelId>,
    ///verification level required for the guild
    pub verification_level: u64,
    ///default message notifications level
    pub default_message_notifications: u64,
    ///explicit content filter level
    pub explicit_content_filter: u64,
    ///roles in the guild
    pub roles: Vec<Role>,
    ///custom guild emojis
    pub emojis: Vec<Emoji>,
    ///enabled guild features
    pub features: Vec<String>,
    ///required MFA level for the guild
    pub mfa_level: u64,
    ///application id of the guild creator if it is bot-created
    pub application_id: Option<Snowflake>,
    ///whether or not the server widget is enabled
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub widget_enabled: Option<bool>,
    ///the channel id for the server widget
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub widget_channel_id: Option<ChannelId>,
    ///the id of the channel to which system messages are sent
    pub system_channel_id: Option<ChannelId>,
    ///when this guild was joined at
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<DateTime<FixedOffset>>,
    ///whether this is considered a large guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub large: Option<bool>,
    ///is this guild unavailable
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<bool>,
    ///total number of members in this guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u64>,
    ///(without the guild_id key)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub voice_states: Option<Vec<VoiceState>>,
    ///users in the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<GuildMember>>,
    ///channels in the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<Channel>>,
    ///presences of the users in the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub presences: Option<Vec<PresenceUpdate>>,
}

bitflags! {
    pub struct Permissions: u64 {
        ///Allows creation of instant invites
        const CREATE_INSTANT_INVITE = 0x00000001;
        ///Allows kicking members
        const KICK_MEMBERS = 0x00000002;
        ///Allows banning members
        const BAN_MEMBERS = 0x00000004;
        ///Allows all permissions and bypasses channel permission overwrites
        const ADMINISTRATOR = 0x00000008;
        ///Allows management and editing of channels
        const MANAGE_CHANNELS = 0x00000010;
        ///Allows management and editing of the guild
        const MANAGE_GUILD = 0x00000020;
        ///Allows for the addition of reactions to messages
        const ADD_REACTIONS = 0x00000040;
        ///Allows for viewing of audit logs
        const VIEW_AUDIT_LOG = 0x00000080;
        ///Allows guild members to view a channel, which includes reading messages in text channels
        const VIEW_CHANNEL = 0x00000400;
        ///Allows for sending messages in a channel
        const SEND_MESSAGES = 0x00000800;
        ///Allows for sending of /tts messages
        const SEND_TTS_MESSAGES = 0x00001000;
        ///Allows for deletion of other users messages
        const MANAGE_MESSAGES = 0x00002000;
        ///Links sent by users with this permission will be auto-embedded
        const EMBED_LINKS = 0x00004000;
        ///Allows for uploading images and files
        const ATTACH_FILES = 0x00008000;
        ///Allows for reading of message history
        const READ_MESSAGE_HISTORY = 0x00010000;
        ///Allows for using the @everyone tag to notify all users in a channel, and the @here tag to notify all online users in a channel
        const MENTION_EVERYONE = 0x00020000;
        ///Allows the usage of custom emojis from other servers
        const USE_EXTERNAL_EMOJIS = 0x00040000;
        ///Allows for joining of a voice channel
        const CONNECT = 0x00100000;
        ///Allows for speaking in a voice channel
        const SPEAK = 0x00200000;
        ///Allows for muting members in a voice channel
        const MUTE_MEMBERS = 0x00400000;
        ///Allows for deafening of members in a voice channel
        const DEAFEN_MEMBERS = 0x00800000;
        ///Allows for moving of members between voice channels
        const MOVE_MEMBERS = 0x01000000;
        ///Allows for using voice-activity-detection in a voice channel
        const USE_VAD = 0x02000000;
        ///Allows for using priority speaker in a voice channel
        const PRIORITY_SPEAKER = 0x00000100;
        ///Allows for modification of own nickname
        const CHANGE_NICK_NAME = 0x04000000;
        ///Allows for modification of other users nicknames
        const MANAGE_NICK_NAMES = 0x08000000;
        ///Allows management and editing of roles
        const MANAGE_ROLES = 0x10000000;
        ///Allows management and editing of webhooks
        const MANAGE_WEB_HOOKS = 0x20000000;
        ///Allows management and editing of emojis
        const MANAGE_EMOJIS = 0x40000000;
    }
}

impl<'de> serde::de::Deserialize<'de> for Permissions{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>
    {
        Ok(Permissions::from_bits_truncate(u64::deserialize(deserializer)?))
    }
}

impl serde::ser::Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer
    {
        serializer.serialize_u64(self.bits())
    }
}

#[derive(Debug,Deserialize)]
pub struct Role{
    ///role id
    pub id: RoleId,
    ///role name
    pub name: String,
    ///integer representation of hexadecimal color code
    pub color: u32,
    ///if this role is pinned in the user listing
    pub hoist: bool,
    ///position of this role
    pub position: u64,
    ///permission bit set
    pub permissions: Permissions,
    ///whether this role is managed by an integration
    pub managed: bool,
    ///whether this role is mentionable
    pub mentionable: bool,
}

#[derive(Debug,Deserialize,Clone)]
pub struct Emoji{
    ///emoji id
    pub id: Option<Snowflake>,
    ///emoji name
    pub name: String,
    ///roles this emoji is whitelisted to
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
    ///user that created this emoji
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    ///whether this emoji must be wrapped in colons
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub require_colons: Option<bool>,
    ///whether this emoji is managed
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
    ///whether this emoji is animated
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
}

#[derive(Debug,Deserialize)]
pub struct GuildMember{
    ///the user this guild member represents
    pub user: User,
    ///this users guild nickname (if one is set)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    ///array of role object ids
    pub roles: Vec<RoleId>,
    ///when the user joined the guild
    pub joined_at: DateTime<FixedOffset>,
    ///whether the user is deafened in voice channels
    pub deaf: bool,
    ///whether the user is muted in voice channels
    pub mute: bool,
}

#[derive(Debug,Deserialize)]
pub struct VoiceState{
    ///the guild id this voice state is for
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the channel id this user is connected to
    pub channel_id: Option<ChannelId>,
    ///the user id this voice state is for
    pub user_id: UserId,
    ///the guild member this voice state is for
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub member: Option<GuildMember>,
    ///the session id for this voice state
    pub session_id: String,
    ///whether this user is deafened by the server
    pub deaf: bool,
    ///whether this user is muted by the server
    pub mute: bool,
    ///whether this user is locally deafened
    pub self_deaf: bool,
    ///whether this user is locally muted
    pub self_mute: bool,
    ///whether this user is muted by the current user
    pub suppress: bool,
}

#[derive(Debug,Deserialize)]
pub struct PresenceUpdate{
    ///the user presence is being updated for
    pub user: PartialUser,
    ///roles this user is in
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
    ///null, or the user's current activity
    pub game: Option<Activity>,
    ///id of the guild
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the user's status
    pub status: Status,
    ///user's current activities
    pub activities: Vec<Activity>,
}

#[derive(Debug,Deserialize,Clone)]
pub struct PartialGuildMember{
    //TODO: figure out which fields are given
}

#[derive(Debug,Deserialize,Clone)]
pub struct Mention{
    //TODO: figure out which fields are given
    //"user object, with an additional partial member field"
}

#[derive(Debug,Deserialize,Clone)]
pub struct Message{
    ///id of the message
    pub id: MessageId,
    ///id of the channel the message was sent in
    pub channel_id: ChannelId,
    ///id of the guild the message was sent in
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the author of this message (not guaranteed to be a valid user if the message was created by a webhook)
    pub author: User,
    ///member properties for this message's author
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub member: Option<PartialGuildMember>,
    ///contents of the message
    pub content: String,
    ///when this message was sent
    pub timestamp: DateTime<FixedOffset>,
    ///when this message was edited (or null if never)
    pub edited_timestamp: Option<DateTime<FixedOffset>>,
    ///whether this was a TTS message
    pub tts: bool,
    ///whether this message mentions everyone
    pub mention_everyone: bool,
    ///users specifically mentioned in the message
    pub mentions: Vec<Mention>,
    ///roles specifically mentioned in this message
    pub mention_roles: Vec<RoleId>,
    ///any attached files
    pub attachments: Vec<Attachment>,
    ///any embedded content
    pub embeds: Vec<Embed>,
    ///reactions to the message
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<Reaction>>,
    ///used for validating a message was sent
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Option<Snowflake>>,
    ///whether this message is pinned
    pub pinned: bool,
    ///if the message is generated by a webhook, this is the webhook's id
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub webhook_id: Option<Snowflake>,
    ///type of message
    #[serde(rename="type")]
    pub msg_type: MessageType,
    ///sent with Rich Presence-related chat embeds
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub activity: Option<MessageActivity>,
    ///sent with Rich Presence-related chat embeds
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub application: Option<MessageApplication>,
}

#[derive(Debug,Deserialize,Clone)]
pub struct Reaction{
    ///times this emoji has been used to react
    pub count: u64,
    ///whether the current user reacted using this emoji
    pub me: bool,
    ///emoji information
    pub emoji: Emoji,
}

#[derive(Debug,Deserialize,Clone)]
pub struct Attachment{
    ///attachment id
    pub id: Snowflake,
    ///name of file attached
    pub filename: String,
    ///size of file in bytes
    pub size: u64,
    ///source url of file
    pub url: String,
    ///a proxied url of file
    pub proxy_url: String,
    ///height of file (if image)
    pub height: Option<u64>,
    ///width of file (if image)
    pub width: Option<u64>,
}

#[derive(Debug,Deserialize,Clone)]
pub struct MessageActivity{
    ///type of message activity
    #[serde(rename="type")]
    pub activity_type: MessageActivityType,
    ///party_id from a Rich Presence event
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
}

enum_number!(MessageActivityType{
    Join = 1,
    Spectate = 2,
    Listen = 3,
    JoinRequest = 5,
});

#[derive(Debug,Deserialize,Clone)]
pub struct MessageApplication{
    ///id of the application
    pub id: Snowflake,
    ///id of the embed's image asset
    pub cover_image: String,
    ///application's description
    pub description: String,
    ///id of the application's icon
    pub icon: String,
    ///name of the application
    pub name: String,
}

enum_number!(MessageType{
    Default = 0,
    RecipientAdd = 1,
    RecipientRemove = 2,
    Call = 3,
    ChannelnameChange = 4,
    ChannelIconChange = 5,
    ChannePinnedMessage = 6,
    GuildMmeberJoin = 7,
});