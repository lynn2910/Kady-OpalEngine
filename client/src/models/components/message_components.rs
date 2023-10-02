use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde_json::Value;
use crate::manager::cache::UpdateCache;
use crate::models::channel::ChannelKind;
use crate::models::components::Emoji;
use crate::models::interaction::InteractionDataOptionValue;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ComponentType {
    ActionRow = 1,
    Button = 2,
    StringSelect = 3,
    TextInput = 4,
    UserSelect = 5,
    RoleSelect = 6,
    MentionSelect = 7,
    ChannelSelect = 8,
}

impl<'de> Deserialize<'de> for ComponentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match Deserialize::deserialize(deserializer)? {
            1 => Ok(Self::ActionRow),
            2 => Ok(Self::Button),
            3 => Ok(Self::StringSelect),
            4 => Ok(Self::TextInput),
            5 => Ok(Self::UserSelect),
            6 => Ok(Self::RoleSelect),
            7 => Ok(Self::MentionSelect),
            8 => Ok(Self::ChannelSelect),
            _ => Err(serde::de::Error::custom("invalid component type"))
        }
    }
}

impl Serialize for ComponentType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Self::ActionRow => serializer.serialize_u8(1),
            Self::Button => serializer.serialize_u8(2),
            Self::StringSelect => serializer.serialize_u8(3),
            Self::TextInput => serializer.serialize_u8(4),
            Self::UserSelect => serializer.serialize_u8(5),
            Self::RoleSelect => serializer.serialize_u8(6),
            Self::MentionSelect => serializer.serialize_u8(7),
            Self::ChannelSelect => serializer.serialize_u8(8),
        }
    }
}

/// Represents an action row in a message
///
/// Reference:
/// - [Action Row Structure](https://discord.com/developers/docs/interactions/message-components#action-row-object-action-row-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ActionRow {
    #[serde(rename = "type")]
    pub kind: ComponentType,
    #[serde(default)]
    pub components: Vec<Component>,
}

impl UpdateCache for ActionRow {
    fn update(&mut self, from: &Self) {
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }

        if self.components != from.components {
            self.components = from.components.clone();
        }
    }
}

impl Default for ActionRow {
    fn default() -> Self {
        Self {
            kind: ComponentType::ActionRow,
            components: Vec::new(),
        }
    }
}

impl ActionRow {
    /// Creates a new action row
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a component to the action row
    pub fn add_component(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }
}

/// Represents a component in a message
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
    TextInput(TextInput),
}

impl Component {
    pub fn to_json(&self) -> Value {
        match self {
            Self::ActionRow(action_row) => serde_json::to_value(action_row).unwrap_or(Value::Null),
            Self::Button(btn) => serde_json::to_value(btn).unwrap_or(Value::Null),
            Self::SelectMenu(select_menu) => serde_json::to_value(select_menu).unwrap_or(Value::Null),
            Self::TextInput(text_input) => serde_json::to_value(text_input).unwrap_or(Value::Null),
        }
    }
}

impl UpdateCache for Component {
    fn update(&mut self, from: &Self) {
        match self {
            Self::ActionRow(action_row) => {
                if let Self::ActionRow(from_action_row) = from {
                    action_row.update(from_action_row);
                }
            },
            Self::Button(btn) => {
                if let Self::Button(from_btn) = from {
                    btn.update(from_btn);
                }
            },
            Self::SelectMenu(select_menu) => {
                if let Self::SelectMenu(from_select_menu) = from {
                    select_menu.update(from_select_menu);
                }
            },
            Self::TextInput(text_input) => {
                if let Self::TextInput(from_text_input) = from {
                    text_input.update(from_text_input);
                }
            }
        }
    }
}




/// Represents a button in a message
///
/// Reference:
/// - [Button Structure](https://discord.com/developers/docs/interactions/message-components#button-object-button-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Button {
    #[serde(rename = "type")]
    pub kind: ComponentType,
    pub style: ButtonStyle,
    pub label: Option<String>,
    pub emoji: Option<Emoji>,
    pub custom_id: String,
    pub url: Option<String>,
    pub disabled: Option<bool>,
}

impl UpdateCache for Button {
    fn update(&mut self, from: &Self) {
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }

        if self.style != from.style {
            self.style = from.style.clone();
        }

        if self.label != from.label {
            self.label = from.label.clone();
        }

        if self.emoji != from.emoji {
            self.emoji = from.emoji.clone();
        }

        if self.custom_id != from.custom_id {
            self.custom_id = from.custom_id.clone();
        }

        if self.url != from.url {
            self.url = from.url.clone();
        }

        if self.disabled != from.disabled {
            self.disabled = from.disabled;
        }
    }
}

impl Button {
    /// Creates a new button
    pub fn new(custom_id: impl ToString) -> Self {
        Self {
            kind: ComponentType::Button,
            style: ButtonStyle::Primary,
            label: None,
            emoji: None,
            custom_id: custom_id.to_string(),
            url: None,
            disabled: None,
        }
    }

    /// Sets the button style
    pub fn set_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the button label
    pub fn set_label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Sets the button emoji
    pub fn set_emoji(mut self, emoji: Emoji) -> Self {
        self.emoji = Some(emoji);
        self
    }

    /// Sets the button url
    pub fn set_url(mut self, url: impl ToString) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Sets the button disabled
    pub fn set_disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

/// Represents a button style
///
/// Reference:
/// - [Button Styles](https://discord.com/developers/docs/interactions/message-components#button-object-button-styles)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5,
}

impl Serialize for ButtonStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Self::Primary => serializer.serialize_u8(1),
            Self::Secondary => serializer.serialize_u8(2),
            Self::Success => serializer.serialize_u8(3),
            Self::Danger => serializer.serialize_u8(4),
            Self::Link => serializer.serialize_u8(5),
        }
    }
}

impl<'de> Deserialize<'de> for ButtonStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match Deserialize::deserialize(deserializer)? {
            1 => Ok(Self::Primary),
            2 => Ok(Self::Secondary),
            3 => Ok(Self::Success),
            4 => Ok(Self::Danger),
            5 => Ok(Self::Link),
            _ => Err(serde::de::Error::custom("invalid button style"))
        }
    }
}



/// Represents a select menu in a message
///
/// Reference:
/// - [Select Menu Structure](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SelectMenu {
    #[serde(rename = "type")]
    pub kind: ComponentType,
    pub custom_id: String,
    /// Specified choices in a select menu (only required and available for string selects (type 3);
    ///
    /// The maximum number of choices is 25.
    #[serde(default)]
    pub options: Vec<SelectOption>,
    /// List of channel types to include in the channel select component
    pub channel_types: Option<Vec<ChannelKind>>,
    /// Placeholder text if nothing is selected
    ///
    /// Maximum 150 characters
    pub placeholder: Option<String>,
    pub min_values: Option<u64>,
    pub max_values: Option<u64>,
    pub disabled: Option<bool>,
}

impl UpdateCache for SelectMenu {
    fn update(&mut self, from: &Self) {
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }
        if self.custom_id != from.custom_id {
            self.custom_id = from.custom_id.clone();
        }
        if self.options != from.options {
            self.options = from.options.clone();
        }
        if self.channel_types != from.channel_types {
            self.channel_types = from.channel_types.clone();
        }
        if self.placeholder != from.placeholder {
            self.placeholder = from.placeholder.clone();
        }
        if self.min_values != from.min_values {
            self.min_values = from.min_values;
        }
        if self.max_values != from.max_values {
            self.max_values = from.max_values;
        }
        if self.disabled != from.disabled {
            self.disabled = from.disabled;
        }
    }
}

/// Represents a select option in a select menu
///
/// Reference:
/// - [Select Menu Structure](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure)
/// - [Select Option Structure](https://discord.com/developers/docs/interactions/message-components#select-option-object-select-option-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<Emoji>,
    pub default: Option<bool>,
}



/// Represents a text input in a modal
///
/// Reference:
/// - [Text Input Structure](https://discord.com/developers/docs/interactions/message-components#text-input-object-text-input-structure)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextInput {
    #[serde(rename = "type")]
    pub kind: ComponentType,
    pub style: Option<TextInputStyle>,
    pub label: Option<String>,
    pub custom_id: String,
    pub placeholder: Option<String>,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub disabled: Option<bool>,

    /// Only in modals :)
    pub value: Option<InteractionDataOptionValue>
}

impl Eq for TextInput {}

impl UpdateCache for TextInput {
    fn update(&mut self, from: &Self) {
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }
        if self.style != from.style {
            self.style = from.style.clone();
        }
        if self.label != from.label {
            self.label = from.label.clone();
        }
        if self.custom_id != from.custom_id {
            self.custom_id = from.custom_id.clone();
        }
        if self.placeholder != from.placeholder {
            self.placeholder = from.placeholder.clone();
        }
        if self.min_length != from.min_length {
            self.min_length = from.min_length;
        }
        if self.max_length != from.max_length {
            self.max_length = from.max_length;
        }
        if self.disabled != from.disabled {
            self.disabled = from.disabled;
        }
    }
}

/// Represents a text input style
///
/// Reference:
/// - [Text Input Styles](https://discord.com/developers/docs/interactions/message-components#text-inputs-text-input-styles)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum TextInputStyle {
    Short = 1,
    Paragraph = 2,
}