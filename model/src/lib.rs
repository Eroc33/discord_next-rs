use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde_json;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

mod payload;
pub use payload::*;
mod ids;
pub use ids::*;
#[macro_use]
mod gateway;
pub use gateway::*;
mod embed;
pub mod voice;
pub use embed::*;
#[macro_use]
mod enum_number;

mod custom_serialization;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConnectionProperties {
    #[serde(rename = "$os")]
    pub os: String,
    #[serde(rename = "$browser")]
    pub browser: String,
    #[serde(rename = "$device")]
    pub device: String,
}

impl Default for ConnectionProperties {
    fn default() -> Self {
        ConnectionProperties {
            os: ::std::env::consts::OS.into(),
            browser: "discord-next-rust".into(),
            device: "discord-next-rust".into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    #[serde(rename = "online")]
    Online,
    //do not disturb`
    #[serde(rename = "dnd")]
    DoNotDisturb,
    //away from keyboard
    #[serde(rename = "idle")]
    AFK,
    //user is invisible, and shown as offline (probably never received)
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "offline")]
    Offline,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateStatus {
    //unix time (in milliseconds) of when the client went idle, or null if the client is not idle
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    //the user's new activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<Activity>,
    //the user's new status
    pub status: Status,
    //whether or not the client is afk
    pub afk: bool,
}

impl UpdateStatus {
    pub fn new_basic(status: Status, afk: bool) -> Self {
        UpdateStatus {
            since: Default::default(),
            game: Default::default(),
            status,
            afk,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    //the activity's name
    pub name: String,
    //activity type
    #[serde(rename = "type")]
    pub typ: ActivityType,
    //is validated when type is 1
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    //unix timestamps for start and/or end of the game
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Timestamps>,
    //application id for the game
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<ApplicationId>,
    //what the player is currently doing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    //the user's current party status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    //information for the current party of the player
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party: Option<Party>,
    //images for the presence and their hover texts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Assets>,
    ///secrets for Rich Presence joining and spectating
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secrets: Option<ActivitySecrets>,
    ///whether or not the activity is an instanced game session
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<bool>,
    ///activity flags, describes what the payload includes
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

impl<'de> serde::de::Deserialize<'de> for ActivityFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self::from_bits_truncate(u32::deserialize(deserializer)?))
    }
}

impl serde::ser::Serialize for ActivityFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActivitySecrets {
    ///the secret for joining a party
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,
    ///the secret for spectating a game
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spectate: Option<String>,
    ///the secret for a specific instanced match
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "match")]
    pub match_secret: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Timestamps {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Party {
    //the id of the party
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    //tuple of two integers (current_size, max_size) used to show the party's current and maximum size
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<(u64, u64)>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Assets {
    //the id for a large asset of the activity, usually a snowflake
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    //text displayed when hovering over the large image of the activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    //the id for a small asset of the activity, usually a snowflake
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    //text displayed when hovering over the small image of the activity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnavailableGuild {
    pub id: GuildId,
    pub unavailable: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Channel {
    //the id of this channel
    pub id: ChannelId,
    //the type of channel
    #[serde(rename = "type")]
    pub typ: ChannelType,
    //the id of the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    //sorting position of the channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    //explicit permission overwrites for members and roles
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permission_overwrites: Vec<Overwrite>,
    //the name of the channel (2-100 characters)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    //the channel topic (0-1024 characters)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    //if the channel is nsfw
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    //the id of the last message sent in this channel (may not point to an existing or valid message)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<MessageId>,
    //the bitrate (in bits) of the voice channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u64>,
    //the user limit of the voice channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u64>,
    //the recipients of the DM
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipients: Vec<User>,
    //icon hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    //id of the DM creator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<UserId>,
    //application id of the group DM creator if it is bot-created
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<ApplicationId>,
    //id of the parent category for a channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<ChannelId>,
    //when the last pinned message was pinned
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    ///voice region id for the voice channel, automatic when set to null
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rtc_region: Option<String>,
    ///the camera video quality mode of the voice channel, 1 when not present
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_quality_mode: Option<u64>,
    ///an approximate count of messages in a thread, stops counting at 50
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_count: Option<u64>,
    ///an approximate count of users in a thread, stops counting at 50
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u64>,
    ///thread-specific fields not needed by other channels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_metadata: Option<ThreadMetadata>,
    ///thread member object for the current user, if they have joined the thread, only included on certain API endpoints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<ThreadMember>,
    ///default duration for newly created threads, in minutes, to automatically archive the thread after recent activity, can be set to: 60, 1440, 4320, 10080
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_auto_archive_duration: Option<u64>,
}

enum_number!(OverwriteType{
    Role = 0,
    Member = 1,
});

#[derive(Debug, Deserialize, Clone)]
pub struct Overwrite {
    //role or user id
    pub id: Snowflake,
    //either "role" or "member"
    #[serde(rename = "type")]
    pub typ: OverwriteType,
    //permission bit set
    #[serde(deserialize_with = "crate::custom_serialization::u64_from_string")]
    pub allow: u64,
    //permission bit set
    #[serde(deserialize_with = "crate::custom_serialization::u64_from_string")]
    pub deny: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    //the user's id
    pub id: UserId,
    //the user's username, not unique across the platform
    pub username: String,
    //the user's 4-digit discord-tag
    pub discriminator: String,
    //the user's avatar hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    //whether the user belongs to an OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    //whether the user has two factor enabled on their account
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfa_enabled: Option<bool>,
    //whether the email on this account has been verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    //the user's email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PartialUser {
    //the user's id
    pub id: UserId,
    //the user's username, not unique across the platform
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    //the user's 4-digit discord-tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    //the user's avatar hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    //whether the user belongs to an OAuth2 application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    //whether the user has two factor enabled on their account
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfa_enabled: Option<bool>,
    //whether the email on this account has been verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    //the user's email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PartialGuild {
    ///guild id
    pub id: GuildId,
    ///guild name (2-100 characters)
    pub name: String,
    ///icon hash
    pub icon: Option<String>,
    ///whether or not the user is the owner of the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<bool>,
    ///total permissions for the user in the guild (does not include channel overrides)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
}

#[derive(Debug, Deserialize)]
pub struct Guild {
    ///guild id
    pub id: GuildId,
    ///guild name (2-100 characters)
    pub name: String,
    ///icon hash
    pub icon: Option<String>,
    ///splash hash
    pub splash: Option<String>,
    ///whether or not the user is the owner of the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<bool>,
    ///id of owner
    pub owner_id: UserId,
    ///total permissions for the user in the guild (does not include channel overrides)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    ///voice region id for the guild
    pub region: String,
    ///id of afk channel
    pub afk_channel_id: Option<ChannelId>,
    ///afk timeout in seconds
    pub afk_timeout: u64,
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
    pub application_id: Option<ApplicationId>,
    ///whether or not the server widget is enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget_enabled: Option<bool>,
    ///the channel id for the server widget
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget_channel_id: Option<ChannelId>,
    ///the id of the channel to which system messages are sent
    pub system_channel_id: Option<ChannelId>,
    ///when this guild was joined at
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joined_at: Option<DateTime<FixedOffset>>,
    ///whether this is considered a large guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large: Option<bool>,
    ///is this guild unavailable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<bool>,
    ///total number of members in this guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u64>,
    ///(without the guild_id key)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_states: Option<Vec<VoiceState>>,
    ///users in the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<GuildMember>>,
    ///channels in the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<Channel>>,
    ///presences of the users in the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presences: Option<Vec<PresenceUpdate>>,
}

bitflags! {
    pub struct Permissions: u64 {
        ///Allows creation of instant invites
        const CREATE_INSTANT_INVITE = 0x0000_0001;
        ///Allows kicking members
        const KICK_MEMBERS = 0x0000_0002;
        ///Allows banning members
        const BAN_MEMBERS = 0x0000_0004;
        ///Allows all permissions and bypasses channel permission overwrites
        const ADMINISTRATOR = 0x0000_0008;
        ///Allows management and editing of channels
        const MANAGE_CHANNELS = 0x0000_0010;
        ///Allows management and editing of the guild
        const MANAGE_GUILD = 0x0000_0020;
        ///Allows for the addition of reactions to messages
        const ADD_REACTIONS = 0x0000_0040;
        ///Allows for viewing of audit logs
        const VIEW_AUDIT_LOG = 0x0000_0080;
        ///Allows guild members to view a channel, which includes reading messages in text channels
        const VIEW_CHANNEL = 0x0000_0400;
        ///Allows for sending messages in a channel
        const SEND_MESSAGES = 0x0000_0800;
        ///Allows for sending of /tts messages
        const SEND_TTS_MESSAGES = 0x0000_1000;
        ///Allows for deletion of other users messages
        const MANAGE_MESSAGES = 0x0000_2000;
        ///Links sent by users with this permission will be auto-embedded
        const EMBED_LINKS = 0x0000_4000;
        ///Allows for uploading images and files
        const ATTACH_FILES = 0x0000_8000;
        ///Allows for reading of message history
        const READ_MESSAGE_HISTORY = 0x0001_0000;
        ///Allows for using the @everyone tag to notify all users in a channel, and the @here tag to notify all online users in a channel
        const MENTION_EVERYONE = 0x0002_0000;
        ///Allows the usage of custom emojis from other servers
        const USE_EXTERNAL_EMOJIS = 0x0004_0000;
        ///Allows for joining of a voice channel
        const CONNECT = 0x0010_0000;
        ///Allows for speaking in a voice channel
        const SPEAK = 0x0020_0000;
        ///Allows for muting members in a voice channel
        const MUTE_MEMBERS = 0x0040_0000;
        ///Allows for deafening of members in a voice channel
        const DEAFEN_MEMBERS = 0x0080_0000;
        ///Allows for moving of members between voice channels
        const MOVE_MEMBERS = 0x0100_0000;
        ///Allows for using voice-activity-detection in a voice channel
        const USE_VAD = 0x0200_0000;
        ///Allows for using priority speaker in a voice channel
        const PRIORITY_SPEAKER = 0x0000_0100;
        ///Allows for modification of own nickname
        const CHANGE_NICK_NAME = 0x0400_0000;
        ///Allows for modification of other users nicknames
        const MANAGE_NICK_NAMES = 0x0800_0000;
        ///Allows management and editing of roles
        const MANAGE_ROLES = 0x1000_0000;
        ///Allows management and editing of webhooks
        const MANAGE_WEB_HOOKS = 0x2000_0000;
        ///Allows management and editing of emojis
        const MANAGE_EMOJIS = 0x4000_0000;
    }
}

impl<'de> serde::de::Deserialize<'de> for Permissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self::from_bits_truncate(
            custom_serialization::u64_from_string(deserializer)?,
        ))
    }
}

impl serde::ser::Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_u64(self.bits())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Role {
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

#[derive(Debug, Deserialize, Clone)]
pub struct Emoji {
    ///emoji id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<EmojiId>,
    ///emoji name (can be null only in reaction emoji objects)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///roles allowed to use this emoji
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
    ///user that created this emoji
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    ///whether this emoji must be wrapped in colons
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_colons: Option<bool>,
    ///whether this emoji is managed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
    ///whether this emoji is animated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
    ///whether this emoji can be used, may be false due to loss of Server Boosts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GuildMember {
    ///the user this guild member represents
    pub user: User,
    ///this users guild nickname (if one is set)
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Deserialize)]
pub struct VoiceState {
    ///the guild id this voice state is for
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the channel id this user is connected to
    pub channel_id: Option<ChannelId>,
    ///the user id this voice state is for
    pub user_id: UserId,
    ///the guild member this voice state is for
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Deserialize)]
pub struct PresenceUpdate {
    ///the user presence is being updated for
    pub user: PartialUser,
    ///id of the guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the user's status
    pub status: Status,
    ///user's current activities
    pub activities: Vec<Activity>,
    ///user's platform-dependent status
    pub client_status: ClientStatus,
}

#[derive(Debug, Deserialize)]
pub struct ClientStatus {
    ///the user's status set for an active desktop (Windows, Linux, Mac) application session
    #[serde(default, skip_serializing_if = "Option::is_none")]
    desktop: Option<String>,
    ///the user's status set for an active mobile (iOS, Android) application session
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mobile: Option<String>,
    ///the user's status set for an active web (browser, bot account) application session
    #[serde(default, skip_serializing_if = "Option::is_none")]
    web: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PartialGuildMember {
    //TODO: figure out which fields are given
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mention {
    //TODO: figure out which fields are given
//"user object, with an additional partial member field"
}

#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    ///id of the message
    pub id: MessageId,
    ///id of the channel the message was sent in
    pub channel_id: ChannelId,
    ///id of the guild the message was sent in
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the author of this message (not guaranteed to be a valid user if the message was created by a webhook)
    pub author: User,
    ///member properties for this message's author
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<Reaction>>,
    ///used for validating a message was sent
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<Option<Snowflake>>,
    ///whether this message is pinned
    pub pinned: bool,
    ///if the message is generated by a webhook, this is the webhook's id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_id: Option<WebhookId>,
    ///type of message
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    ///sent with Rich Presence-related chat embeds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<MessageActivity>,
    ///sent with Rich Presence-related chat embeds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<Application>, // should be PartialApplication
    ///	if the message is a response to an Interaction, this is the id of the interaction's application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<ApplicationId>,
    ///data showing the source of a crosspost, channel follow add, pin, or reply message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    ///message flags combined as a bitfield
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<MessageFlags>,
    ///the message associated with the message_reference
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referenced_message: Option<Box<Message>>,
    ///sent if the message is a response to an Interaction
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interaction: Option<MessageInteraction>,
    ///the thread that was started from this message, includes thread member object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread: Option<Channel>,
    ///sent if the message contains components like buttons, action rows, or other interactive components
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<MessageComponent>>,
    ///sent if the message contains stickers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker_items: Option<Vec<MessageStickerItem>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Reaction {
    ///times this emoji has been used to react
    pub count: u64,
    ///whether the current user reacted using this emoji
    pub me: bool,
    ///emoji information
    pub emoji: Emoji,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Attachment {
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

#[derive(Debug, Deserialize, Clone)]
pub struct MessageActivity {
    ///type of message activity
    #[serde(rename = "type")]
    pub activity_type: MessageActivityType,
    ///party_id from a Rich Presence event
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
}

enum_number!(MessageActivityType{
    Join = 1,
    Spectate = 2,
    Listen = 3,
    JoinRequest = 5,
});

#[derive(Debug, Deserialize, Clone)]
pub struct MessageApplication {
    ///id of the application
    pub id: ApplicationId,
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
    GuildMemberJoin = 7,
    UserPremiumGuildSubscription = 8,
    UserPremiumGuildSubscriptionTier1 = 9,
    UserPremiumGuildSubscriptionTier2 = 10,
    UserPremiumGuildSubscriptionTier3 = 11,
    ChannelFollowAdd = 12,
    GuildDiscoveryDisqualified = 14,
    GuildDiscoveryRequalified = 15,
    GuildDiscoveryGracePeriodInitialWarning = 16,
    GuildDiscoveryGracePeriodFinalWarning = 17,
    ThreadCreated = 18,
    Reply = 19,
    ApplicationCommand = 20,
    ThreadStarterMessage = 21,
    GuildInviteReminder = 22,
});

bitflags! {
    pub struct IntentFlags: u32 {
        const GUILDS                   = (1 << 0);
        /// This is a privileged intent
        const GUILD_MEMBERS            = (1 << 1);
        const GUILD_BANS               = (1 << 2);
        const GUILD_EMOJIS             = (1 << 3);
        const GUILD_INTEGRATIONS       = (1 << 4);
        const GUILD_WEBHOOKS           = (1 << 5);
        const GUILD_INVITES	           = (1 << 6);
        const GUILD_VOICE_STATES       = (1 << 7);
        /// This is a privileged intent
        const GUILD_PRESENCES          = (1 << 8);
        const GUILD_MESSAGES           = (1 << 9);
        const GUILD_MESSAGE_REACTIONS  = (1 << 10);
        const GUILD_MESSAGE_TYPING     = (1 << 11);
        const DIRECT_MESSAGES          = (1 << 12);
        const DIRECT_MESSAGE_REACTIONS = (1 << 13);
        const DIRECT_MESSAGE_TYPING	   = (1 << 14);

        const KNOWN_PRIVILEGED = Self::GUILD_MEMBERS.bits | Self::GUILD_PRESENCES.bits;
    }
}

impl Default for IntentFlags {
    fn default() -> Self {
        Self::all() - Self::KNOWN_PRIVILEGED
    }
}

impl<'de> serde::de::Deserialize<'de> for IntentFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self::from_bits_truncate(u32::deserialize(deserializer)?))
    }
}

impl serde::ser::Serialize for IntentFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageReference {
    ///id of the originating message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    ///id of the originating message's channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<ChannelId>,
    ///id of the originating message's guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///when sending, whether to error if the referenced message doesn't exist instead of sending as a normal (non-reply) message, default true
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_if_not_exists: Option<bool>,
}

bitflags! {
    pub struct MessageFlags: u32 {
        ///this message has been published to subscribed channels (via Channel Following)
        const CROSSPOSTED             = (1 << 0);
        ///	his message originated from a message in another channel (via Channel Following)
        const IS_CROSSPOST            = (1 << 1);
        ///do not include any embeds when serializing this message
        const SUPPRESS_EMBEDS         = (1 << 2);
        ///the source message for this crosspost has been deleted (via Channel Following)
        const SOURCE_MESSAGE_DELETED  = (1 << 3);
        ///this message came from the urgent message system
        const URGENT                  = (1 << 4);
        ///this message has an associated thread, with the same id as the message
        const HAS_THREAD              = (1 << 5);
        ///this message is only visible to the user who invoked the Interaction
        const EPHEMERAL	              = (1 << 6);
        ///this message is an Interaction Response and the bot is "thinking"
        const LOADING                 = (1 << 7);
    }
}

impl<'de> serde::de::Deserialize<'de> for MessageFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self::from_bits_truncate(u32::deserialize(deserializer)?))
    }
}

impl serde::ser::Serialize for MessageFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageInteraction {
    ///id of the interaction
    pub id: InteractionId,
    ///the type of interaction
    #[serde(rename = "type")]
    pub typ: InteractionType,
    ///the name of the application command
    pub name: String,
    ///the user who invoked the interaction
    pub user: User,
}

enum_number!(InteractionType{
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
});

enum_number!(ChannelType{
    GuildText =	0,
    DirectMessage = 1,
    GuildVoice = 2,
    GroupDm = 3,
    GuildCategory = 4,
    GuildNews = 5,
    GuildStore = 6,
    GuildNewsThread = 10,
    GuildPublicThread = 11,
    GuildPrivateThread = 12,
    GuildStageVoice = 13,
});

#[derive(Debug, Deserialize, Clone)]
pub struct MessageComponent {
    ///component type
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub typ: Option<MessageComponentType>,
    ///one of button styles
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<ButtonStyle>,
    ///text that appears on the button, max 80 characters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    ///	name, id, and animated
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<Emoji>,
    ///a developer-defined identifier for the button, max 100 characters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_id: Option<String>,
    ///a url for link-style buttons
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    ///whether the button is disabled, default false
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    ///a list of child components
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<MessageComponent>>,
}

enum_number!(MessageComponentType{
    ActionRow = 1,
    Button = 2,
    SelectMenu = 3,
});

enum_number!(ButtonStyle{
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5,
});

#[derive(Debug, Deserialize, Clone)]
pub struct MessageStickerItem {
    ///id of the sticker
    pub id: StickerId,
    ///name of the sticker
    pub name: String,
    ///type of sticker format
    pub format_type: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadMetadata {
    ///whether the thread is archived
    pub archived: bool,
    ///duration in minutes to automatically archive the thread after recent activity, can be set to: 60, 1440, 4320, 10080
    pub auto_archive_duration: u64,
    ///timestamp when the thread's archive status was last changed, used for calculating recent activity
    pub archive_timestamp: DateTime<FixedOffset>,
    ///when a thread is locked, only users with MANAGE_THREADS can unarchive it
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadMember {
    ///whether the thread is archived
    pub archived: bool,
    ///duration in minutes to automatically archive the thread after recent activity, can be set to: 60, 1440, 4320, 10080
    pub auto_archive_duration: u64,
    ///timestamp when the thread's archive status was last changed, used for calculating recent activity
    pub archive_timestamp: DateTime<FixedOffset>,
    ///when a thread is locked, only users with MANAGE_THREADS can unarchive it
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Application {
    ///the id of the app
    pub id: ApplicationId,
    ///the name of the app
    pub name: String,
    ///the icon hash of the app
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    ///the description of the app
    pub description: String,
    ///an array of rpc origin urls, if rpc is enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rpc_origins: Option<Vec<String>>,
    ///when false only app owner can join the app's bot to guilds
    pub bot_public: bool,
    ///when true the app's bot will only join upon completion of the full oauth2 code grant flow
    pub bot_require_code_grant: bool,
    ///the url of the app's terms of service
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terms_of_service_url: Option<String>,
    ///the url of the app's privacy policy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
    ///partial user object containing info on the owner of the application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<PartialUser>,
    ///if this application is a game sold on Discord, this field will be the summary field for the store page of its primary sku
    pub summary: String,
    ///the hex encoded key for verification in interactions and the GameSDK's GetTicket
    pub verify_key: String,
    ///if the application belongs to a team, this will be a list of the members of that team
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<Team>,
    ///if this application is a game sold on Discord, this field will be the guild to which it has been linked
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///if this application is a game sold on Discord, this field will be the id of the "Game SKU" that is created, if exists
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_sku_id: Option<Snowflake>,
    ///if this application is a game sold on Discord, this field will be the URL slug that links to the store page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    ///the application's default rich presence invite cover image hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    ///the application's public flags
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<ApplicationFlags>,
}

bitflags! {
    pub struct ApplicationFlags: u32 {
        const GATEWAY_PRESENCE                 = (1 << 12);
        const GATEWAY_PRESENCE_LIMITED         = (1 << 13);
        const GATEWAY_GUILD_MEMBERS            = (1 << 14);
        const GATEWAY_GUILD_MEMBERS_LIMITED    = (1 << 15);
        const VERIFICATION_PENDING_GUILD_LIMIT = (1 << 16);
        const EMBEDDED                         = (1 << 17);
    }
}

impl<'de> serde::de::Deserialize<'de> for ApplicationFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Self::from_bits_truncate(u32::deserialize(deserializer)?))
    }
}

impl serde::ser::Serialize for ApplicationFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Team {
    ///a hash of the image of the team's icon
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    ///the unique id of the team
    pub id: TeamId,
    ///the members of the team
    pub members: Vec<TeamMember>,
    ///the name of the team
    pub name: String,
    ///the user id of the current team owner
    pub owner_user_id: UserId,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TeamMember {
    ///the user's membership state on the team
    pub membership_state: MembershipState,
    ///will always be ["*"]
    pub permissions: Vec<String>,
    ///the id of the parent team of which they are a member
    pub team_id: TeamId,
    ///the avatar, discriminator, id, and username of the user
    pub user: PartialUser,
}

enum_number!(MembershipState{
    Invited = 1,
    Accepted = 2,
});

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationCommand {
    ///unique id of the command
    pub id: ApplicationCommandId,
    ///unique id of the parent application
    pub application_id: ApplicationId,
    ///guild id of the command, if not global
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///1-32 lowercase character name matching ^[\w-]{1,32}$
    pub name: String,
    ///1-100 character description
    pub description: String,
    ///the parameters for the command
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ApplicationCommandOption>>,
    ///whether the command is enabled by default when the app is added to a guild
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_permission: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationCommandOption {
    ///value of application command option type
    #[serde(rename = "type")]
    pub typ: ApplicationCommandOptionType,
    ///1-32 lowercase character name matching ^[\w-]{1,32}$
    pub name: String,
    ///1-100 character description
    pub description: String,
    ///if the parameter is required or optional--default false
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    ///choices for string and int types for the user to pick from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<ApplicationCommandOptionChoice>>,
    ///if the option is a subcommand or subcommand group type, this nested options will be the parameters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ApplicationCommandOption>>,
}

enum_number!(ApplicationCommandOptionType{
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
    Mentionable = 9,
});

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringOrInt {
    String(String),
    Int(i64),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationCommandOptionChoice {
    ///1-100 character choice name
    pub name: String,
    ///value of the choice, up to 100 characters if string
    pub value: StringOrInt,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Interaction {
    ///id of the interaction
    pub id: InteractionId,
    ///id of the application this interaction is for
    pub application_id: ApplicationId,
    ///the type of interaction
    #[serde(rename = "type")]
    pub typ: InteractionType,
    ///the command data payload
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<ApplicationCommandInteractionData>,
    ///the guild it was sent from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    ///the channel it was sent from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<ChannelId>,
    ///guild member data for the invoking user, including permissions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<GuildMember>,
    ///user object for the invoking user, if invoked in a DM
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    ///a continuation token for responding to the interaction
    pub token: String,
    ///read-only property, always 1
    pub version: u64,
    ///for components, the message they were attached to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationCommandInteractionData {
    ///the ID of the invoked command
    pub id: ApplicationCommandId,
    ///the name of the invoked command
    pub name: String,
    ///converted users + roles + channels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved: Option<ApplicationCommandInteractionDataResolved>,
    ///the params + values from the user
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ApplicationCommandInteractionDataOption>>,
    ///for components, the custom_id of the component
    pub custom_id: String,
    ///for components, the type of the component
    pub component_type: MessageComponentType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationCommandInteractionDataResolved {
    ///the ids and User objects
    pub users: HashMap<UserId, User>,
    ///the ids and partial Member objects
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<HashMap<GuildId, GuildMember>>,
    ///the ids and Role objects
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<HashMap<RoleId, Role>>,
    ///the ids and partial Channel objects
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<ChannelId, Channel>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationCommandInteractionDataOption {
    ///the name of the parameter
    pub name: String,
    ///value of application command option type
    #[serde(rename = "type")]
    pub typ: i64,
    ///the value of the pair
    pub value: Option<ApplicationCommandOptionType>,
    ///present if this option is a group or subcommand
    pub options: Option<Vec<ApplicationCommandInteractionDataOption>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StageInstance {
    ///The id of this Stage instance
    pub id: StageInstanceId,
    ///The guild id of the associated Stage channel
    pub guild_id: GuildId,
    ///The id of the associated Stage channel
    pub channel_id: ChannelId,
    ///The topic of the Stage instance (1-120 characters)
    pub topic: String,
    ///The privacy level of the Stage instance
    pub privacy_level: PrivacyLevel,
    ///Whether or not Stage discovery is disabled
    pub discoverable_disabled: bool,
}
enum_number!(PrivacyLevel{
    Public = 1,
    GuildOnly = 2,
});
