use chrono::{TimeZone, Utc};
use serde::{Serialize, Deserialize };
use serde_json::Value;
use error::{Error, ModelError, Result};
use crate::manager::cache::UpdateCache;
use crate::manager::http::HttpRessource;
use crate::models::components::Color;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Embed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub kind: Option<EmbedType>,

    pub url: Option<String>,
    pub timestamp: Option<chrono::DateTime<Utc>>,

    pub color: Option<Color>,
    pub footer: Option<Footer>,
    pub image: Option<EmbedImage>,
    pub thumbnail: Option<Thumbnail>,
    pub fields: Option<Vec<Field>>,

    pub provider: Option<Provider>,
    pub author: Option<Author>,
}

impl Embed {
    /// Create a new embed without any fields
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title of the embed
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description of the embed
    pub fn set_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the type of the embed
    pub fn set_kind(mut self, kind: EmbedType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Set the url of the embed
    pub fn set_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the timestamp of the embed
    pub fn set_timestamp(mut self, timestamp: chrono::DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the color of the embed
    pub fn set_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the footer of the embed
    pub fn set_footer(mut self, footer: Footer) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Set the image of the embed
    pub fn set_image(mut self, image: EmbedImage) -> Self {
        self.image = Some(image);
        self
    }

    /// Set the thumbnail of the embed
    pub fn set_thumbnail(mut self, thumbnail: Thumbnail) -> Self {
        self.thumbnail = Some(thumbnail);
        self
    }

    /// Add a field to the embed
    pub fn add_field(mut self, field: Field) -> Self {
        match self.fields {
            Some(ref mut fields) => fields.push(field),
            None => self.fields = Some(vec![field])
        }
        self
    }

    /// Add multiple fields to the embed
    pub fn add_fields(mut self, fields: Vec<Field>) -> Self {
        match self.fields {
            Some(ref mut embed_fields) => {
                for field in fields {
                    embed_fields.push(field.to_owned());
                }
            },
            None => self.fields = Some(fields)
        }
        self
    }

    /// Set the fields of the embed
    pub fn set_fields(mut self, fields: Vec<Field>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Set the provider of the embed
    pub fn set_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the author of the embed
    pub fn set_author(mut self, author: Author) -> Self {
        self.author = Some(author);
        self
    }
}

impl UpdateCache for Embed {
    fn update(&mut self, from: &Self) {
        if self.title != from.title {
            self.title = from.title.clone();
        }
        if self.description != from.description {
            self.description = from.description.clone();
        }
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }
        if self.url != from.url {
            self.url = from.url.clone();
        }
        if self.timestamp != from.timestamp {
            self.timestamp = from.timestamp;
        }
        if self.color != from.color {
            self.color = from.color.clone();
        }
        if self.footer != from.footer {
            self.footer = from.footer.clone();
        }
        if self.image != from.image {
            self.image = from.image.clone();
        }
        if self.thumbnail != from.thumbnail {
            self.thumbnail = from.thumbnail.clone();
        }
        if self.fields != from.fields {
            self.fields = from.fields.clone();
        }
        if self.provider != from.provider {
            self.provider = from.provider.clone();
        }
        if self.author != from.author {
            self.author = from.author.clone();
        }
    }
}

impl HttpRessource for Embed {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let title = match raw.get("title") {
            Some(title) => match title.as_str() {
                Some(title) => Some(title.into()),
                None => return Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed title".into())))
            },
            None => None
        };
        let description = match raw.get("description") {
            Some(description) => match description.as_str() {
                Some(description) => Some(description.into()),
                None => return Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed description".into())))
            },
            None => None
        };
        let kind = match raw.get("type") {
            Some(kind) => Some(EmbedType::from_raw(kind.clone(), shard)?),
            None => None
        };
        let url = match raw.get("url") {
            Some(url) => match url.as_str() {
                Some(url) => Some(url.into()),
                None => return Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed url".into())))
            },
            None => None
        };
        let timestamp = match raw.get("timestamp") {
            Some(timestamp) => match timestamp.as_i64() {
                Some(timestamp) => match Utc.timestamp_millis_opt(timestamp) {
                    chrono::LocalResult::Single(timestamp) => Some(timestamp),
                    _ => return Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed timestamp".into())))
                },
                None => return Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed timestamp".into())))
            },
            None => None
        };
        let color = match raw.get("color") {
            Some(color) => Some(Color::from_raw(color.clone(), shard)?),
            None => None
        };
        let footer = match raw.get("footer") {
            Some(footer) => Some(Footer::from_raw(footer.clone(), shard)?),
            None => None
        };
        let image = match raw.get("image") {
            Some(image) => Some(EmbedImage::from_raw(image.clone(), shard)?),
            None => None
        };
        let thumbnail = match raw.get("thumbnail") {
            Some(thumbnail) => Some(Thumbnail::from_raw(thumbnail.clone(), shard)?),
            None => None
        };
        let fields = match raw.get("fields") {
            Some(fields) => match fields.as_array() {
                Some(fields) => Some(fields.iter().map(|field| Field::from_raw(field.clone(), shard)).collect::<Result<Vec<Field>>>()?),
                None => return Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed fields".into())))
            },
            None => None
        };
        let provider = match raw.get("provider") {
            Some(provider) => Some(Provider::from_raw(provider.clone(), shard)?),
            None => None
        };
        let author = match raw.get("author") {
            Some(author) => Some(Author::from_raw(author.clone(), shard)?),
            None => None
        };

        Ok(Self {
            title,
            description,
            kind,
            url,
            timestamp,
            color,
            footer,
            image,
            thumbnail,
            fields,
            provider,
            author
        })
    }
}

/// Represent the type of an embed
///
/// Reference:
/// - [Embed Type](https://discord.com/developers/docs/resources/channel#embed-object-embed-types)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum EmbedType {
    Rich,
    Image,
    Video,
    Gifv,
    Article,
    Link
}

impl HttpRessource for EmbedType {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_str() {
            Some("rich") => Ok(Self::Rich),
            Some("image") => Ok(Self::Image),
            Some("video") => Ok(Self::Video),
            Some("gifv") => Ok(Self::Gifv),
            Some("article") => Ok(Self::Article),
            Some("link") => Ok(Self::Link),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse embed type".into())))
        }
    }
}

/// Represents a footer in an embed
///
/// Reference:
/// - [Embed Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-structure)
/// - [Embed Footer Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-footer-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Footer {
    pub text: String,
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>
}

impl HttpRessource for Footer {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let text = if let Some(text) = raw.get("text") {
            if let Some(t) = text.as_str() { t.to_string() }
            else { return Err(Error::Model(ModelError::InvalidPayload("Failed to parse footer text: field 'text' isn't a string".into())))}
        } else { return Err(Error::Model(ModelError::InvalidPayload("Failed to parse footer text: no 'text' field".into()))) };
        let icon_url = if let Some(icon_url) = raw.get("icon_url") { icon_url.as_str().map(|t| t.to_string()) } else { None };
        let proxy_icon_url = if let Some(proxy_icon_url) = raw.get("proxy_icon_url") { proxy_icon_url.as_str().map(|t| t.to_string()) } else { None };

        Ok(Self { text, icon_url, proxy_icon_url })
    }
}

impl Footer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the icon url of the footer
    pub fn set_icon_url(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// Set the text of the footer
    pub fn set_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }
}

/// Represents an image in an embed
///
/// Reference:
/// - [Embed Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-structure)
/// - [Embed Image Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-image-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EmbedImage {
    pub url: Option<String>,
    pub proxy_url: Option<String>,
    pub height: Option<u64>,
    pub width: Option<u64>
}

impl UpdateCache for EmbedImage {
    fn update(&mut self, from: &Self) {
        if self.url != from.url {
            self.url = from.url.clone();
        }
        if self.proxy_url != from.proxy_url {
            self.proxy_url = from.proxy_url.clone();
        }
        if self.height != from.height {
            self.height = from.height;
        }
        if self.width != from.width {
            self.width = from.width;
        }
    }
}

impl HttpRessource for EmbedImage {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let url = if let Some(url) = raw.get("url") { url.as_str().map(|t| t.to_string()) } else { None };
        let proxy_url = if let Some(proxy_url) = raw.get("proxy_url") { proxy_url.as_str().map(|t| t.to_string()) } else { None };
        let height = if let Some(height) = raw.get("height") { height.as_u64() } else { None };
        let width = if let Some(width) = raw.get("width") { width.as_u64() } else { None };

        Ok(EmbedImage {
            url,
            proxy_url,
            height,
            width
        })
    }
}

impl EmbedImage {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            proxy_url: None,
            height: None,
            width: None
        }
    }
}

/// Represents a thumbnail in an embed
///
/// Reference:
/// - [Embed Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-structure)
/// - [Embed Thumbnail Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-thumbnail-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Thumbnail {
    pub url: Option<String>,
    pub proxy_url: Option<String>,
    pub height: Option<u64>,
    pub width: Option<u64>
}

impl UpdateCache for Thumbnail {
    fn update(&mut self, from: &Self) {
        if self.url != from.url {
            self.url = from.url.clone();
        }
        if self.proxy_url != from.proxy_url {
            self.proxy_url = from.proxy_url.clone();
        }
        if self.height != from.height {
            self.height = from.height;
        }
        if self.width != from.width {
            self.width = from.width;
        }
    }
}

impl HttpRessource for Thumbnail {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let url = if let Some(url) = raw.get("url") { url.as_str().map(|t| t.to_string()) } else { None };
        let proxy_url = if let Some(proxy_url) = raw.get("proxy_url") { proxy_url.as_str().map(|t| t.to_string()) } else { None };
        let height = if let Some(height) = raw.get("height") { height.as_u64() } else { None };
        let width = if let Some(width) = raw.get("width") { width.as_u64() } else { None };

        Ok(Thumbnail {
            url,
            proxy_url,
            height,
            width
        })
    }
}

impl Thumbnail {
    /// Create a new thumbnail with the given url
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            proxy_url: None,
            height: None,
            width: None
        }
    }
}

/// Represents a provider in an embed
///
/// Reference:
/// - [Embed Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-structure)
/// - [Embed Provider Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-provider-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Provider {
    pub name: Option<String>,
    pub url: Option<String>
}

impl UpdateCache for Provider {
    fn update(&mut self, from: &Self) {
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.url != from.url {
            self.url = from.url.clone();
        }
    }
}

impl HttpRessource for Provider {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let name = if let Some(name) = raw.get("name") { name.as_str().map(|t| t.to_string()) } else { None };
        let url = if let Some(url) = raw.get("url") { url.as_str().map(|t| t.to_string()) } else { None };

        Ok(Provider {
            name,
            url
        })
    }
}

impl Provider {
    /// Create a new empty provider
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the url of the provider
    pub fn set_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the name of the provider
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Represents an author in an embed
///
/// Reference:
/// - [Embed Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-structure)
/// - [Embed Author Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-author-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Author {
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
    pub proxy_icon_url: Option<String>
}

impl UpdateCache for Author {
    fn update(&mut self, from: &Self) {
        if self.name != from.name {
            self.name = from.name.clone();
        }

        if self.url != from.url {
            self.url = from.url.clone();
        }

        if self.icon_url != from.icon_url {
            self.icon_url = from.icon_url.clone();
        }

        if self.proxy_icon_url != from.proxy_icon_url {
            self.proxy_icon_url = from.proxy_icon_url.clone();
        }
    }
}

impl HttpRessource for Author {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let name = if let Some(name) = raw.get("name") {
            if let Some(name) = name.as_str() {
                name.to_string()
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Field 'name' for embed author is not a string".into())));
            }
        } else { return Err(Error::Model(ModelError::MissingField("Field 'name' for embed author is missing".into()))); };
        let url = if let Some(url) = raw.get("url") { url.as_str().map(|t| t.to_string()) } else { None };
        let icon_url = if let Some(icon_url) = raw.get("icon_url") { icon_url.as_str().map(|t| t.to_string()) } else { None };
        let proxy_icon_url = if let Some(proxy_icon_url) = raw.get("proxy_icon_url") { proxy_icon_url.as_str().map(|t| t.to_string()) } else { None };

        Ok(Author {
            name,
            url,
            icon_url,
            proxy_icon_url
        })
    }
}

impl Author {
    /// Create a new empty author
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name of the author
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the url of the author
    pub fn set_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the icon url of the author
    pub fn set_icon_url(mut self, icon_url: Option<impl ToString>) -> Self {
        self.icon_url = icon_url.map(|u| u.to_string());
        self
    }
}

/// Represents a field in an embed
///
/// Reference:
/// - [Embed Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-structure)
/// - [Embed Field Structure](https://discord.com/developers/docs/resources/channel#embed-object-embed-field-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub inline: Option<bool>
}

impl UpdateCache for Field {
    fn update(&mut self, from: &Self) {
        if self.name != from.name {
            self.name = from.name.clone();
        }

        if self.value != from.value {
            self.value = from.value.clone();
        }

        if self.inline != from.inline {
            self.inline = from.inline;
        }
    }
}

impl HttpRessource for Field {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let name = if let Some(name) = raw.get("name") {
            if let Some(name) = name.as_str() {
                name.to_string()
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Field 'name' for embed field is not a string".into())));
            }
        } else { return Err(Error::Model(ModelError::MissingField("Field 'name' for embed field is missing".into()))); };

        let value = if let Some(value) = raw.get("value") {
            if let Some(value) = value.as_str() {
                value.to_string()
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Field 'value' for embed field is not a string".into())));
            }
        } else { return Err(Error::Model(ModelError::MissingField("Field 'value' for embed field is missing".into()))); };
        let inline = if let Some(inline) = raw.get("inline") { inline.as_bool() } else { None };

        Ok(Field {
            name,
            value,
            inline
        })
    }
}

impl Field {
    /// Create a new empty field
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name of the field
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the value of the field
    pub fn set_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set the inline of the field
    pub fn set_inline(mut self, inline: bool) -> Self {
        self.inline = Some(inline);
        self
    }
}