use chrono::{DateTime,FixedOffset};

#[derive(Debug,Deserialize)]
pub struct Embed{
    ///title of embed
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    ///type of embed (always "rich" for webhook embeds)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    #[serde(rename="type")]
    pub embed_type: Option<String>,
    ///description of embed
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    ///url of embed
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    ///timestamp of embed content
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<FixedOffset>>,
    ///color code of the embed
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    ///footer information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
    ///image information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    ///thumbnail information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<EmbedThumbnail>,
    ///video information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub video: Option<EmbedVideo>,
    ///provider information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub provider: Option<EmbedProvider>,
    ///author information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub author: Option<EmbedAuthor>,
    ///fields information
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedThumbnail{
    ///source url of thumbnail (only supports http(s) and attachments)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    ///a proxied url of the thumbnail
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    ///height of thumbnail
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    ///width of thumbnail
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedVideo{
    ///source url of video
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    ///height of video
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    ///width of video
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedImage{
    ///source url of image (only supports http(s) and attachments)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    ///a proxied url of the image
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    ///height of image
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub height: Option<u64>,
    ///width of image
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub width: Option<u64>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedProvider{
    ///name of provider
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///url of provider
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedAuthor{
    ///name of author
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///url of author
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    ///url of author icon (only supports http(s) and attachments)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    ///a proxied url of author icon
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedFooter{
    ///footer text
    pub text: String,
    ///url of footer icon (only supports http(s) and attachments)
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    ///a proxied url of footer icon
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub proxy_icon_url: Option<String>,
}

#[derive(Debug,Deserialize)]
pub struct EmbedField{
    ///name of the field
    pub name: String,
    ///value of the field
    pub value: String,
    ///whether or not this field should display inline
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}