use chrono::{DateTime,FixedOffset};

#[derive(Debug,Deserialize,Serialize,Default,Clone)]
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

impl Embed{
    pub fn set_color(&mut self,color: u32) -> &mut Self{
        self.color = Some(color);
        self
    }
    pub fn set_title<S: Into<String>>(&mut self,title: S) -> &mut Self{
        self.title = Some(title.into());
        self
    }
    pub fn set_description<S: Into<String>>(&mut self,description: S) -> &mut Self{
        self.description = Some(description.into());
        self
    }
    pub fn with_fields<I>(&mut self, fields: I)
        where I: IntoIterator<Item=EmbedField>
    {
        self.fields = Some(fields.into_iter().collect())
    }
    pub fn push_field(&mut self, field: EmbedField)
    {
        self.fields = self.fields.take().or_else(Default::default);
        self.fields.as_mut().expect("just set fields to be Some").push(field)
    }
    pub fn data_size(&self) -> usize{
        self.title.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.description.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.url.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.footer.as_ref().map(|s| s.data_size()).unwrap_or(0) + 
        self.image.as_ref().map(|s| s.data_size()).unwrap_or(0) + 
        self.thumbnail.as_ref().map(|s| s.data_size()).unwrap_or(0) + 
        self.video.as_ref().map(|s| s.data_size()).unwrap_or(0) + 
        self.provider.as_ref().map(|s| s.data_size()).unwrap_or(0) + 
        self.author.as_ref().map(|s| s.data_size()).unwrap_or(0) + 
        self.fields.as_ref().map(|fs| fs.iter().map(|f| f.data_size()).sum()).unwrap_or(0)
    }
    pub fn enforce_embed_limits(&self) -> Result<(),EmbedTooBigError>
    {
        let title_len = self.title.as_ref().map(|s| s.len()).unwrap_or(0);
        if title_len > 256 {
            return Err(EmbedTooBigError::FieldTooBig{field: "title", length: title_len, max: 256});
        }
        let description_len = self.description.as_ref().map(|s| s.len()).unwrap_or(0);
        if description_len > 256 {
            return Err(EmbedTooBigError::FieldTooBig{field: "description", length: description_len, max: 2048});
        }
        if let Some(fields) = self.fields.as_ref(){
            if fields.len() > 25 {
                return Err(EmbedTooBigError::FieldTooBig{field: "fields", length: fields.len(), max: 25});
            }
            for field in fields{
                field.enforce_embed_limits()?;
            }
        }
        if self.data_size() > 6000 {
            return Err(EmbedTooBigError::WholeTooBig(self.data_size()))
        }
        Ok(())
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
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

impl EmbedThumbnail{
    //TODO: clarify: do urls count towards the size limit?
    pub fn data_size(&self) -> usize{
        self.url.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.proxy_url.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
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

impl EmbedVideo{
    //TODO: clarify: do urls count towards the size limit?
    pub fn data_size(&self) -> usize{
        self.url.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
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

impl EmbedImage{
    //TODO: clarify: do urls count towards the size limit?
    pub fn data_size(&self) -> usize{
        self.url.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.proxy_url.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct EmbedProvider{
    ///name of provider
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    ///url of provider
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl EmbedProvider{
    //TODO: clarify: do urls count towards the size limit?
    pub fn data_size(&self) -> usize{
        self.name.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.url.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
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

impl EmbedAuthor{
    //TODO: clarify: do urls count towards the size limit?
    pub fn data_size(&self) -> usize{
        self.name.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.url.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.icon_url.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.proxy_icon_url.as_ref().map(|s| s.len()).unwrap_or(0)
    }
    pub fn enforce_embed_limits(&self) -> Result<(),EmbedTooBigError>{
        let name_len = self.name.as_ref().map(|s| s.len()).unwrap_or(0);
        if name_len > 256 {
            return Err(EmbedTooBigError::FieldTooBig{field: "footer.name",length: name_len, max: 256});
        }
        Ok(())
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
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

impl EmbedFooter{
    //TODO: clarify: do urls count towards the size limit?
    pub fn data_size(&self) -> usize{
        self.text.len() + 
        self.icon_url.as_ref().map(|s| s.len()).unwrap_or(0) + 
        self.proxy_icon_url.as_ref().map(|s| s.len()).unwrap_or(0)
    }
    pub fn enforce_embed_limits(&self) -> Result<(),EmbedTooBigError>{
        if self.text.len() > 2048 {
            return Err(EmbedTooBigError::FieldTooBig{field: "footer.text",length: self.text.len(), max: 2048});
        }
        Ok(())
    }
}

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct EmbedField{
    ///name of the field
    pub name: String,
    ///value of the field
    pub value: String,
    ///whether or not this field should display inline
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}

impl EmbedField{
    pub fn inline(name: String, value: String) -> Self{
        Self{
            name,
            value,
            inline: Some(true)
        }
    }
    pub fn new(name: String, value: String) -> Self{
        Self{
            name,
            value,
            inline: None
        }
    }
}

impl EmbedField{
    pub fn data_size(&self) -> usize{
        self.name.len() + 
        self.value.len()
    }
    pub fn enforce_embed_limits(&self) -> Result<(),EmbedTooBigError>{
        if self.name.len() > 256 {
            return Err(EmbedTooBigError::FieldTooBig{field: "field.name",length: self.name.len(), max: 256});
        }
        if self.value.len() > 1024 {
            return Err(EmbedTooBigError::FieldTooBig{field: "field.value",length: self.value.len(), max: 1024});
        }
        Ok(())
    }
}

#[derive(Debug,Fail)]
pub enum EmbedTooBigError{
    #[fail(display="Tried to send an embed which was too big. {} was {} bytes when it should have been at most {} bytes",field,length,max)]
    FieldTooBig{
        field: &'static str,
        length: usize,
        max: usize,
    },
    #[fail(display="Tried to send an embed which was too big. maximum size is 6000 bytes, but the mebed was {} bytes",_0)]
    WholeTooBig(usize)
}