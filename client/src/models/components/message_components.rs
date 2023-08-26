use serde::{ Serialize, Deserialize };
use serde_json::{json, Value};
use error::{Result, Error, ModelError};
use crate::manager::cache::UpdateCache;
use crate::manager::http::HttpRessource;
use crate::models::channel::ChannelKind;
use crate::models::components::Emoji;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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

impl HttpRessource for ComponentType {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::ActionRow),
            Some(2) => Ok(Self::Button),
            Some(3) => Ok(Self::StringSelect),
            Some(4) => Ok(Self::TextInput),
            Some(5) => Ok(Self::UserSelect),
            Some(6) => Ok(Self::RoleSelect),
            Some(7) => Ok(Self::MentionSelect),
            Some(8) => Ok(Self::ChannelSelect),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse component type".into())))
        }
    }
}

impl ComponentType {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::ActionRow => 1,
            Self::Button => 2,
            Self::StringSelect => 3,
            Self::TextInput => 4,
            Self::UserSelect => 5,
            Self::RoleSelect => 6,
            Self::MentionSelect => 7,
            Self::ChannelSelect => 8
        }.into()
    }
}

/// Represents an action row in a message
///
/// Reference:
/// - [Action Row Structure](https://discord.com/developers/docs/interactions/message-components#action-row-object-action-row-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ActionRow {
    pub kind: ComponentType,
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

impl HttpRessource for ActionRow {
     fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let kind = ComponentType::ActionRow;
        let components = raw["components"]
            .as_array()
            .ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse action row components".into())))?
            .iter()
            .map(|c| Component::from_raw(c.clone(), shard))
            .collect::<Result<Vec<Component>>>()?;

        Ok(Self {
            kind,
            components,
        })
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

    pub(crate) fn to_json(&self) -> Value {
        let mut components = Vec::new();
        for comp in &self.components {
            components.push(comp.to_json())
        }

        json!({
            "type": self.kind.to_json(),
            "components": Value::Array(components)
        })
    }
}

/// Represents a component in a message
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Component {
    ActionRow(ActionRow),
    Button(Button),
    SelectMenu(SelectMenu),
    TextInput(TextInput),
}

impl Component {
    pub fn to_json(&self) -> Value {
        match self {
            Self::ActionRow(action_row) => action_row.to_json(),
            Self::Button(btn) => btn.to_json(),
            Self::SelectMenu(select_menu) => select_menu.to_json(),
            Self::TextInput(text_input) => text_input.to_json()
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

impl HttpRessource for Component {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let kind = ComponentType::from_raw(raw["type"].clone(), shard)?;

        match kind {
            ComponentType::ActionRow => Ok(Self::ActionRow(ActionRow::from_raw(raw, shard)?)),
            ComponentType::Button => Ok(Self::Button(Button::from_raw(raw, shard)?)),
            ComponentType::StringSelect | ComponentType::UserSelect
                | ComponentType::RoleSelect | ComponentType::MentionSelect | ComponentType::ChannelSelect => Ok(Self::SelectMenu(SelectMenu::from_raw(raw, shard)?)),
            ComponentType::TextInput => Ok(Self::TextInput(TextInput::from_raw(raw, shard)?)),
        }
    }
}





/// Represents a button in a message
///
/// Reference:
/// - [Button Structure](https://discord.com/developers/docs/interactions/message-components#button-object-button-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Button {
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

impl HttpRessource for Button {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let kind = ComponentType::Button;
        let style = ButtonStyle::from_raw(raw["style"].clone(), shard)?;
        let label = raw["label"].as_str().map(|s| s.to_string());
        let emoji = if let Some(emoji) = raw.get("emoji") { Some(Emoji::from_raw(emoji.clone(), shard)?) } else {  None  };
        let custom_id = raw["custom_id"].as_str().map(|s| s.to_string()).ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse button custom id".into())))?;
        let url = raw["url"].as_str().map(|s| s.to_string());
        let disabled = raw["disabled"].as_bool();

        Ok(Self {
            kind,
            style,
            label,
            emoji,
            custom_id,
            url,
            disabled,
        })
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

    pub(crate) fn to_json(&self) -> Value {
        let payload = json!({
            "type": self.kind.to_json(),
            "style": self.style.to_json(),
            "label": self.label,
            "emoji": self.emoji.clone().map(|e| e.to_json()),
            "custom_id": self.custom_id,
            "url": self.url,
            "disabled": self.disabled
        });
        payload
    }
}

/// Represents a button style
///
/// Reference:
/// - [Button Styles](https://discord.com/developers/docs/interactions/message-components#button-object-button-styles)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5,
}

impl HttpRessource for ButtonStyle {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::Primary),
            Some(2) => Ok(Self::Secondary),
            Some(3) => Ok(Self::Success),
            Some(4) => Ok(Self::Danger),
            Some(5) => Ok(Self::Link),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse button style".into()))),
        }
    }
}

impl ButtonStyle {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Primary => 1,
            Self::Secondary => 2,
            Self::Success => 3,
            Self::Danger => 4,
            Self::Link => 5,
        }.into()
    }
}





/// Represents a select menu in a message
///
/// Reference:
/// - [Select Menu Structure](https://discord.com/developers/docs/interactions/message-components#select-menu-object-select-menu-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SelectMenu {
    pub kind: ComponentType,
    pub custom_id: String,
    /// Specified choices in a select menu (only required and available for string selects (type 3);
    ///
    /// The maximum number of choices is 25.
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

impl HttpRessource for SelectMenu {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let kind = ComponentType::from_raw(raw["type"].clone(), shard)?;
        let custom_id = raw["custom_id"].as_str().ok_or(Error::Model(ModelError::InvalidPayload("Failed to parse select menu options".into())))?.to_string();
        let options = raw["options"].as_array().ok_or(Error::Model(ModelError::InvalidPayload("Failed to parse select menu options".into())))?.iter().map(|o| SelectOption::from_raw(o.clone(), shard)).collect::<Result<Vec<SelectOption>>>()?;
        let channel_types = raw["channel_types"].as_array().map(|a| a.iter().map(|v| ChannelKind::from_raw(v.clone(), shard)).collect::<Result<Vec<ChannelKind>>>()).transpose()?;
        let placeholder = raw["placeholder"].as_str().map(|s| s.to_string());
        let min_values = raw["min_values"].as_u64();
        let max_values = raw["max_values"].as_u64();
        let disabled = raw["disabled"].as_bool();

        Ok(Self {
            kind,
            custom_id,
            options,
            channel_types,
            placeholder,
            min_values,
            max_values,
            disabled,
        })
    }
}

impl SelectMenu {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "type": self.kind.to_json(),
            "custom_id": self.custom_id,
            "options": self.options.clone()
                .into_iter()
                .map(|option| option.to_json())
                .collect::<Vec<Value>>(),
            "channel_types": self.channel_types.clone().map(|channels| channels
                .into_iter()
                .map(|kind| kind.to_json())
                .collect::<Vec<Value>>()),
            "placeholder": self.placeholder,
            "min_values": self.min_values,
            "max_values": self.max_values,
            "disabled": self.disabled
        })
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

impl SelectOption {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value,
            "description": self.description,
            "emoji": self.emoji.clone().map(|e| e.to_json()),
            "default": self.default
        })
    }
}

impl HttpRessource for SelectOption {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let label = raw["label"].as_str().ok_or(Error::Model(ModelError::InvalidPayload("Failed to parse select option label".into())))?.to_string();
        let value = raw["value"].as_str().ok_or(Error::Model(ModelError::InvalidPayload("Failed to parse select option value".into())))?.to_string();
        let description = raw["description"].as_str().map(|s| s.to_string());
        let emoji = raw.get("emoji").map(|e| Emoji::from_raw(e.clone(), shard)).transpose()?;
        let default = raw["default"].as_bool();

        Ok(Self { label, value, description, emoji, default })
    }
}




/// Represents a text input in a modal
///
/// Reference:
/// - [Text Input Structure](https://discord.com/developers/docs/interactions/message-components#text-input-object-text-input-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TextInput {
    pub kind: ComponentType,
    pub style: TextInputStyle,
    pub label: String,
    pub custom_id: String,
    pub placeholder: Option<String>,
    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub disabled: Option<bool>,
}

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

impl HttpRessource for TextInput {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let kind = ComponentType::TextInput;
        let style = TextInputStyle::from_raw(raw["style"].clone(), shard)?;
        let label = raw["label"].as_str().ok_or(Error::Model(ModelError::InvalidPayload("Failed to parse text input label".into())))?.to_string();
        let custom_id = raw["custom_id"].as_str().ok_or(Error::Model(ModelError::InvalidPayload("Failed to parse text input custom id".into())))?.to_string();
        let placeholder = raw["placeholder"].as_str().map(|s| s.to_string());
        let min_length = raw["min_length"].as_u64();
        let max_length = raw["max_length"].as_u64();
        let disabled = raw["disabled"].as_bool();

        Ok(Self {
            kind,
            style,
            label,
            custom_id,
            placeholder,
            min_length,
            max_length,
            disabled,
        })
    }
}

impl TextInput {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "type": self.kind.to_json(),
            "style": self.style.to_json(),
            "label": self.label,
            "custom_id": self.custom_id,
            "placeholder": self.placeholder,
            "min_length": self.min_length,
            "max_length": self.max_length,
            "disabled": self.disabled
        })
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

impl HttpRessource for TextInputStyle {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::Short),
            Some(2) => Ok(Self::Paragraph),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse text input style".into())))
        }
    }
}

impl TextInputStyle {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Short => 1,
            Self::Paragraph => 2
        }.into()
    }
}