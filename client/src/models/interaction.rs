use std::collections::HashMap;
use std::fmt::Display;
use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde_json::{json, Number, Value};
use error::Result;
use crate::manager::cache::UpdateCache;
use crate::manager::http::{ApiResult, Http};
use crate::models::channel::{Channel, ChannelId, ChannelKind};
use crate::models::components::message_components::{Component, ComponentType};
use crate::models::guild::{GuildId, GuildMember, Role};
use crate::models::message::{Attachment, AttachmentBuilder, Message, MessageBuilder, MessageFlags};
use crate::models::Snowflake;
use crate::models::user::{User, UserId};

/// The type of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-types)
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutocomplete = 4,
    ModalSubmit = 5,
}

impl<'de> Deserialize<'de> for InteractionType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let value: i64 = Deserialize::deserialize(deserializer)?;

        match value {
            1 => Ok(Self::Ping),
            2 => Ok(Self::ApplicationCommand),
            3 => Ok(Self::MessageComponent),
            4 => Ok(Self::ApplicationCommandAutocomplete),
            5 => Ok(Self::ModalSubmit),
            _ => Err(serde::de::Error::custom(format!("Unknown interaction type: {}", value)))
        }
    }
}

/// Represents an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Snowflake,
    pub application_id: Snowflake,
    #[serde(rename = "type")]
    pub interaction_type: InteractionType,
    pub data: Option<InteractionData>,
    pub guild_id: Option<GuildId>,
    pub channel: Option<Channel>,
    pub channel_id: Option<ChannelId>,
    /// The member that invoked this interaction
    ///
    /// Will be present only if the interaction is invoked in a guild
    pub member: Option<GuildMember>,
    /// The user that invoked this interaction
    ///
    /// Will be present only if the interaction is invoked in a DM
    pub user: Option<User>,
    /// Continuation token for responding to the interaction
    pub token: String,
    /// Bitwise set of permissions the app or bot has within the channel the interaction was sent from
    pub app_permissions: Option<String>,
    /// Selected language of the invoking user
    ///
    /// Available for all interaction types except `Ping`
    pub locale: Option<String>,
    /// Guild's preferred locale, if invoked in a guild
    pub guild_locale: Option<String>,
    /// When the interaction is a component, the message attached
    pub message: Option<Message>
}

impl Interaction {
    /// Respond to an interaction
    ///
    /// Reference:
    /// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type)
    pub async fn reply(&self, http: &Http, content: MessageBuilder) -> Result<ApiResult<()>> {
        let json = serde_json::to_value(content)?;

        http.reply_interaction(
            &self.id,
            &self.token,
            InteractionCallbackType::ChannelMessageWithSource,
            json,
            None
        ).await
    }

    pub async fn reply_with_modal(&self, http: &Http, content: MessageBuilder) -> Result<ApiResult<()>> {
        let json = content.to_json();

        http.reply_interaction(
            &self.id,
            &self.token,
            InteractionCallbackType::Modal,
            json,
            None
        ).await
    }

    pub async fn reply_with_files(&self, http: &Http, content: MessageBuilder, files: Vec<AttachmentBuilder>) -> Result<ApiResult<()>> {
        let json = content.to_json();

        http.reply_interaction(
            &self.id,
            &self.token,
            InteractionCallbackType::ChannelMessageWithSource,
            json,
            Some(files)
        ).await
    }

    /// Acknowledge an interaction without sending a message
    ///
    /// Reference:
    /// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type)
    pub async fn defer(&self, http: &Http, flags: Option<MessageFlags>) -> Result<ApiResult<()>> {
        http.reply_interaction(
            &self.id,
            &self.token,
            InteractionCallbackType::DeferredChannelMessageWithSource,
            json!({ "flags": flags.map(|f| f.to_json()) }),
            None
        ).await
    }

    /// Edit an interaction response
    ///
    /// Reference:
    /// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response)
    pub async fn update(
        &self,
        http: &Http,
        content: MessageBuilder
    ) -> Result<ApiResult<Message>> {
        let json = content.to_json();

        http.edit_interaction(
            &self.application_id,
            &self.token,
            json,
            None
        ).await
    }

    pub async fn update_with_files(
        &self,
        http: &Http,
        content: MessageBuilder,
        files: Vec<AttachmentBuilder>
    ) -> Result<ApiResult<Message>> {
        let json = content.to_json();

        http.edit_interaction(
            &self.application_id,
            &self.token,
            json,
            Some(files)
        ).await
    }

    /// Edits the original message.
    ///
    /// # Arguments
    ///
    /// * `http` - A reference to the Http structure.
    /// * `content` - A MessageBuilder instance for building the content of your message
    pub async fn edit_original_message(&self, http: &Http, content: MessageBuilder) -> Option<Result<ApiResult<()>>> {
        if self.channel_id.is_none() || self.message.is_none() {
            return None;
        }

        Some(
            http.reply_interaction(
                &self.id,
                &self.token,
                InteractionCallbackType::UpdateMessage,
                content.to_json(),
                None
            ).await
        )
        //
        // Some(
        //     http.edit_message(
        //         self.channel_id.as_ref().unwrap(),
        //         &self.message.as_ref().unwrap().id,
        //         content,
        //         None
        //     ).await
        // )
    }
}

/// Represents the type of an interaction
///
/// Documentation: https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InteractionCallbackType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
    ApplicationCommandAutocompleteResult = 8,
    Modal = 9
}

impl Serialize for InteractionCallbackType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            Self::Pong => 1,
            Self::ChannelMessageWithSource => 4,
            Self::DeferredChannelMessageWithSource => 5,
            Self::DeferredUpdateMessage => 6,
            Self::UpdateMessage => 7,
            Self::ApplicationCommandAutocompleteResult => 8,
            Self::Modal => 9
        };

        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for InteractionCallbackType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let value: i64 = Deserialize::deserialize(deserializer)?;

        match value {
            1 => Ok(Self::Pong),
            4 => Ok(Self::ChannelMessageWithSource),
            5 => Ok(Self::DeferredChannelMessageWithSource),
            6 => Ok(Self::DeferredUpdateMessage),
            7 => Ok(Self::UpdateMessage),
            8 => Ok(Self::ApplicationCommandAutocompleteResult),
            9 => Ok(Self::Modal),
            _ => Err(serde::de::Error::custom(format!("Unknown interaction callback type: {}", value)))
        }
    }
}

impl InteractionCallbackType {
    pub(crate) fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// Represents the data of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionData {
    /// The ID of the invoked command
    pub id: Option<Snowflake>,
    /// The name of the invoked command
    pub name: Option<String>,
    /// The type of the invoked command
    #[serde(rename = "type")]
    pub command_type: Option<ApplicationCommandType>,
    /// ID of the user or message targeted by a user or message command
    pub target_id: Option<Snowflake>,
    /// Converted users + roles + channels + attachments
    pub resolved: Option<InteractionDataResolved>,
    /// The params + values from the user
    pub options: Option<Vec<InteractionDataOption>>,

    /// The values returned from a select menu interaction
    pub values: Option<Vec<String>>,

    /// The type of the component
    pub component_type: Option<ComponentType>,
    pub components: Option<Vec<Component>>,
    /// The custom ID of the component
    pub custom_id: Option<String>
}

/// Represents the resolved data of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-resolved-structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionDataResolved {
    pub users: Option<HashMap<UserId, User>>,
    ///  Partial Member objects are missing user, deaf and mute fields
    pub members: Option<HashMap<Snowflake, GuildMember>>,
    pub roles: Option<HashMap<Snowflake, Role>>,
    ///  Partial Channel objects only have `id`, `name`, `type` and `permissions` fields. Threads will also have `thread_metadata` and `parent_id` fields.
    pub channels: Option<HashMap<ChannelId, Channel>>,
    pub messages: Option<HashMap<Snowflake, Message>>,
    pub attachments: Option<HashMap<Snowflake, Attachment>>,
}

/// Represents the options of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-application-command-interaction-data-option-structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionDataOption {
    pub name: String,
    #[serde(rename = "type")]
    pub option_type: ApplicationCommandOptionType,
    pub value: Option<InteractionDataOptionValue>,
    pub options: Option<Vec<InteractionDataOption>>,
    pub focused: Option<bool>,
}

/// Value of the option resulting from the user input
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InteractionDataOptionValue {
    String(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
    None
}

impl Default for InteractionDataOptionValue {
    fn default() -> Self {
        Self::None
    }
}

impl Display for InteractionDataOptionValue {
    fn fmt(&self, f1: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::String(s) => s.to_string(),
            Self::Integer(i) => i.to_string(),
            Self::Double(f) => f.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::None => String::new()
        };
        write!(f1, "{}", str)
    }
}

impl From<&str> for InteractionDataOptionValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for InteractionDataOptionValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl Eq for InteractionDataOptionValue {}














/// The type of an application command
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-types)
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub enum ApplicationCommandType {
    /// Slash commands; a text-based command that shows up when a user types /
    ChatInput = 1,
    /// Slash commands; a text-based command that shows up when a user types /
    User = 2,
    /// A UI-based command that shows up when you right click or tap on a message
    Message = 3,
}

impl<'de> Deserialize<'de> for ApplicationCommandType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        match Deserialize::deserialize(deserializer)? {
            1 => Ok(Self::ChatInput),
            2 => Ok(Self::User),
            3 => Ok(Self::Message),
            _ => Err(serde::de::Error::custom("Unknown application command type"))
        }
    }
}

impl ApplicationCommandType {
    pub(crate) fn to_number(&self) -> u64 {
        match self {
            Self::ChatInput => 1,
            Self::User => 2,
            Self::Message => 3,
        }
    }
}


/// Represents a Command
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ApplicationCommand {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub command_type: ApplicationCommandType,
    pub application_id: Snowflake,
    #[serde(default)]
    pub guild_id: Option<GuildId>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_localizations: Option<HashMap<String, String>>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_localizations: Option<HashMap<String, String>>,
    /// Parameters for the command, max of 25
    ///
    /// Available only for ChatInput commands
    #[serde(default)]
    pub options: Option<Vec<ApplicationCommandOption>>,
    /// Set of permissions represented as a bit set
    pub default_member_permissions: Option<String>,
    #[serde(default)]
    pub dm_permission: bool,
    /// Indicates whether the command is age-restricted, defaults to false
    #[serde(default)]
    pub nsfw: bool,
    /// Auto-incrementing version identifier updated during substantial record changes
    pub version: Snowflake,
}

impl UpdateCache for ApplicationCommand {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.command_type != from.command_type {
            self.command_type = from.command_type.clone();
        }
        if self.application_id != from.application_id {
            self.application_id = from.application_id.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.name_localizations != from.name_localizations {
            self.name_localizations = from.name_localizations.clone();
        }
        if self.description != from.description {
            self.description = from.description.clone();
        }
        if self.description_localizations != from.description_localizations {
            self.description_localizations = from.description_localizations.clone();
        }
        if self.options != from.options {
            self.options = from.options.clone();
        }
        if self.default_member_permissions != from.default_member_permissions {
            self.default_member_permissions = from.default_member_permissions.clone();
        }
        if self.dm_permission != from.dm_permission {
            self.dm_permission = from.dm_permission;
        }
        if self.nsfw != from.nsfw {
            self.nsfw = from.nsfw;
        }
        if self.version != from.version {
            self.version = from.version.clone();
        }
    }
}

impl ApplicationCommand {
    /// Creates a new global command
    pub fn new_global(name: impl ToString, description: impl ToString, command_type: ApplicationCommandType) -> Self {
        Self {
            id: Snowflake(String::new()),
            command_type,
            application_id: Snowflake(String::new()),
            guild_id: None,
            name: name.to_string(),
            name_localizations: None,
            description: description.to_string(),
            description_localizations: None,
            options: None,
            default_member_permissions: None,
            dm_permission: false,
            nsfw: false,
            version: Snowflake(String::new()),
        }
    }

    pub fn new_local(name: impl ToString, description: impl ToString, command_type: ApplicationCommandType, guild_id: GuildId) -> Self {
        Self {
            id: Snowflake(String::new()),
            command_type,
            application_id: Snowflake(String::new()),
            guild_id: Some(guild_id),
            name: name.to_string(),
            name_localizations: None,
            description: description.to_string(),
            description_localizations: None,
            options: None,
            default_member_permissions: None,
            dm_permission: false,
            nsfw: false,
            version: Snowflake(String::new()),
        }
    }

    /// Add a localization to the command
    pub fn add_localization(mut self, lang: impl ToString, name: impl ToString, description: impl ToString) -> Self {
        if self.name_localizations.is_none() {
            self.name_localizations = Some(HashMap::new());
        }
        if self.description_localizations.is_none() {
            self.description_localizations = Some(HashMap::new());
        }
        self.name_localizations.as_mut().unwrap().insert(lang.to_string(), name.to_string());
        self.description_localizations.as_mut().unwrap().insert(lang.to_string(), description.to_string());
        self
    }

    /// Set the command to be executable in DMs or not
    pub fn set_dm_permission(mut self, dm_permission: bool) -> Self {
        self.dm_permission = dm_permission;
        self
    }

    /// Set the command to be nsfw or not
    pub fn set_nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = nsfw;
        self
    }

    /// Add an option to the command
    pub fn add_option(mut self, option: ApplicationCommandOption) -> Self {
        if self.options.is_none() {
            self.options = Some(Vec::new());
        }
        self.options.as_mut().unwrap().push(option);
        self
    }

    /// Turn the command into a JSON value to be send as a POST request to Discord
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "options": self.options.as_ref().map(|options| {
                options.iter().map(|option| option.to_json()).collect::<Vec<Value>>()
            }),
            "default_permission": self.default_member_permissions,
            "type": self.command_type.to_number(),
            "version": self.version,
        })
    }
}

/// Represents a command option
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ApplicationCommandOption {
    #[serde(rename = "type")]
    pub option_type: ApplicationCommandOptionType,
    pub name: String,
    #[serde(default)]
    pub name_localizations: Option<HashMap<String, String>>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_localizations: Option<HashMap<String, String>>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Option<Vec<ApplicationCommandOptionChoice>>,
    #[serde(default)]
    pub options: Option<Vec<ApplicationCommandOption>>,
    #[serde(default)]
    pub channel_types: Option<Vec<ChannelKind>>,

    pub min_value: Option<OptionValue>,
    pub max_value: Option<OptionValue>,

    pub min_length: Option<u64>,
    pub max_length: Option<u64>,
    pub autocomplete: Option<bool>,
}

impl ApplicationCommandOption {
    pub fn new(
        option_type: ApplicationCommandOptionType,
        name: impl ToString,
        description: impl ToString,
        required: bool
    ) -> Self {
        Self {
            option_type,
            required,
            name: name.to_string(),
            name_localizations: None,
            description: description.to_string(),
            description_localizations: None,
            choices: None,
            options: None,
            channel_types: None,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
            autocomplete: None
        }
    }

    pub fn add_name_localization(mut self, lang: impl ToString, name: impl ToString) -> Self {
        if self.name_localizations.is_none() {
            self.name_localizations = Some(HashMap::new())
        }

        self.name_localizations.as_mut().unwrap().insert(lang.to_string(), name.to_string());

        self
    }

    pub fn add_description_localization(mut self, lang: impl ToString, description: impl ToString) -> Self {
        if self.description_localizations.is_none() {
            self.description_localizations = Some(HashMap::new())
        }

        self.description_localizations.as_mut().unwrap().insert(lang.to_string(), description.to_string());

        self
    }

    pub fn add_option(mut self, option: Self) -> Self {
        if self.options.is_none() {
            self.options = Some(Vec::new())
        }

        self.options.as_mut().unwrap().push(option);

        self
    }

    pub fn add_choice(mut self, option: Self) -> Self {
        if self.options.is_none() {
            self.options = Some(Vec::new())
        }

        self.options.as_mut().unwrap().push(option);

        self
    }

    pub(crate) fn to_json(&self) -> Value {
        json!({
            "type": self.option_type.to_json(),
            "name": self.name,
            "description": self.description,
            "required": self.required,
            "choices": self.choices.as_ref().map(|choices| {
                choices.iter().map(|choice| choice.to_json()).collect::<Vec<Value>>()
            }),
            "options": self.options.as_ref().map(|options| {
                options.iter().map(|option| option.to_json()).collect::<Vec<Value>>()
            }),
            "channel_types": self.channel_types.as_ref().map(|channel_types| {
                channel_types.iter().map(|channel_type| channel_type.to_json()).collect::<Vec<Value>>()
            }),
            "min_value": self.min_value.as_ref().map(|min_value| min_value.to_json()),
            "max_value": self.max_value.as_ref().map(|max_value| max_value.to_json()),
            "min_length": self.min_length,
            "max_length": self.max_length,
            "autocomplete": self.autocomplete,
        })
    }
}

/// Used for the min_value and max_value fields of ApplicationCommandOption
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionValue {
    Integer(i64),
    Double(f64),
}

impl Eq for OptionValue {

}

impl OptionValue {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Integer(int) => Value::Number(Number::from(*int)),
            Self::Double(double) => Value::Number(Number::from_f64(*double).unwrap_or(Number::from_f64(f64::MIN).expect("Failed to parse f64 min"))),
        }
    }
}

/// Represents a command option type
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-type)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ApplicationCommandOptionType {
    SubCommand = 1,
    SubCommandGroup = 2,
    String = 3,
    Integer = 4,
    Boolean = 5,
    User = 6,
    Channel = 7,
    Role = 8,
    Mentionable = 9,
    Number = 10,
    Attachment = 11,
}

impl<'de> Deserialize<'de> for ApplicationCommandOptionType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        match Deserialize::deserialize(deserializer)? {
            1 => Ok(Self::SubCommand),
            2 => Ok(Self::SubCommandGroup),
            3 => Ok(Self::String),
            4 => Ok(Self::Integer),
            5 => Ok(Self::Boolean),
            6 => Ok(Self::User),
            7 => Ok(Self::Channel),
            8 => Ok(Self::Role),
            9 => Ok(Self::Mentionable),
            10 => Ok(Self::Number),
            11 => Ok(Self::Attachment),
            _ => Err(serde::de::Error::custom("Unknown application command option type"))
        }
    }
}

impl Serialize for ApplicationCommandOptionType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            Self::SubCommand => 1,
            Self::SubCommandGroup => 2,
            Self::String => 3,
            Self::Integer => 4,
            Self::Boolean => 5,
            Self::User => 6,
            Self::Channel => 7,
            Self::Role => 8,
            Self::Mentionable => 9,
            Self::Number => 10,
            Self::Attachment => 11,
        };

        value.serialize(serializer)
    }
}

impl ApplicationCommandOptionType {
    pub(crate) fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// Represents a command choice
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ApplicationCommandOptionChoice {
    pub name: String,
    pub name_localizations: Option<HashMap<String, String>>,
    pub value: CommandChoiceValue,
}

impl ApplicationCommandOptionChoice {
    pub fn new_string(name: String, value: String) -> Self {
        Self {
            name,
            name_localizations: None,
            value: CommandChoiceValue::String(value),
        }
    }

    pub fn new_integer(name: String, value: i64) -> Self {
        Self {
            name,
            name_localizations: None,
            value: CommandChoiceValue::Integer(value),
        }
    }

    /// Adds a localization to the command choice with the given language and name
    pub fn add_localization(mut self, lang: String, name: String) -> Self {
        if self.name_localizations.is_none() {
            self.name_localizations = Some(HashMap::new());
        }

        self.name_localizations.as_mut().unwrap().insert(lang, name);

        self
    }

    pub(crate) fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "name_localizations": self.name_localizations.clone(),
            "value": self.value.to_json(),
        })
    }
}

/// Represents a command choice value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandChoiceValue {
    String(String),
    Integer(i64),
    Double(f64),
}

impl Eq for CommandChoiceValue {}

impl CommandChoiceValue {
    pub fn new_string(s: String) -> Self {
        Self::String(s)
    }

    pub fn new_integer(i: i64) -> Self {
        Self::Integer(i)
    }

    pub fn new_double(d: f64) -> Self {
        Self::Double(d)
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::String(s) => Value::String(s.clone()),
            Self::Integer(i) => Value::Number(Number::from(*i)),
            Self::Double(d) => Value::Number(Number::from_f64(*d).unwrap()),
        }
    }
}