use chrono::Utc;
use serde::{Serialize, Deserialize };
use crate::manager::cache::UpdateCache;
use crate::models::components::Color;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct Embed {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub kind: Option<EmbedType>,

    #[serde(default)]
    pub url: Option<String>,
    #[serde(with = "crate::models::components::timestamp_serde")]
    #[serde(default)]
    pub timestamp: Option<chrono::DateTime<Utc>>,

    #[serde(default)]
    pub color: Option<Color>,
    #[serde(default)]
    pub footer: Option<Footer>,
    #[serde(default)]
    pub image: Option<EmbedImage>,
    #[serde(default)]
    pub thumbnail: Option<Thumbnail>,
    #[serde(default)]
    pub fields: Option<Vec<Field>>,

    #[serde(default)]
    pub provider: Option<Provider>,
    #[serde(default)]
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

/// Represent the type of an embed
///
/// Reference:
/// - [Embed Type](https://discord.com/developers/docs/resources/channel#embed-object-embed-types)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EmbedType {
    #[default]
    Rich,
    Image,
    Video,
    Gifv,
    Article,
    Link
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