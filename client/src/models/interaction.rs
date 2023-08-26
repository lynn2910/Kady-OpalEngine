use std::collections::HashMap;
use serde::{Serialize, Deserialize };
use serde_json::{json, Number, Value};
use error::{Error, ModelError, Result};
use crate::manager::http::{ApiResult, Http, HttpRessource};
use crate::models::channel::{Channel, ChannelId, ChannelKind};
use crate::models::components::message_components::ComponentType;
use crate::models::guild::{GuildId, GuildMember, Role};
use crate::models::message::{Attachment, AttachmentBuilder, Message, MessageBuilder, MessageFlags};
use crate::models::Snowflake;
use crate::models::user::{User, UserId};

/// The type of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-types)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutocomplete = 4,
    ModalSubmit = 5,
}

impl HttpRessource for InteractionType {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::Ping),
            Some(2) => Ok(Self::ApplicationCommand),
            Some(3) => Ok(Self::MessageComponent),
            Some(4) => Ok(Self::ApplicationCommandAutocomplete),
            Some(5) => Ok(Self::ModalSubmit),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction type".into())))
        }
    }
}

/// Represents an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-structure)
#[derive(Debug)]
pub struct Interaction {
    pub id: Snowflake,
    pub application_id: Snowflake,
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
}

impl HttpRessource for Interaction {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {

        let id = if let Some(id) = raw.get("id") {
            if let Some(id) = id.as_str() {
                Snowflake::from_raw(id.into(), shard)?
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction id".into())))
            }
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction id".into())))
        };

        let application_id = if let Some(application_id) = raw.get("application_id") {
            if let Some(application_id) = application_id.as_str() {
                Snowflake::from_raw(application_id.into(), shard)?
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction application_id".into())))
            }
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction application_id".into())))
        };

        let interaction_type = if let Some(interaction_type) = raw.get("type") {
            InteractionType::from_raw(interaction_type.clone(), shard)?
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction type".into())))
        };

        let data = if let Some(data) = raw.get("data") {
            Some(InteractionData::from_raw(data.clone(), shard)?)
        } else {
            None
        };

        let guild_id = if let Some(guild_id) = raw.get("guild_id") {
            if let Some(guild_id) = guild_id.as_str() {
                Some(GuildId::from_raw(guild_id.into(), shard)?)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction guild_id".into())))
            }
        } else {
            None
        };

        let channel = if let Some(channel) = raw.get("channel_id") {
            if let Ok(channel) = Channel::from_raw(channel.clone(), shard) {
                Some(channel)
            } else {
                None
            }
        } else {
            None
        };

        let channel_id = if let Some(channel_id) = raw.get("channel_id") {
            if let Some(channel_id) = channel_id.as_str() {
                Some(ChannelId::from_raw(channel_id.into(), shard)?)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction channel_id".into())))
            }
        } else {
            None
        };

        let member = if let Some(mut member) = raw.get("member").cloned() {
            if let Some(object) = member.as_object_mut() {
                object.insert("guild_id".to_string(), json!(guild_id.clone()));
                Some(GuildMember::from_raw(Value::Object(object.clone()), shard)?)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("The GuildMember object is not an object".into())))
            }
        } else {
            None
        };

        let user = if let Some(user) = raw.get("user") {
            Some(User::from_raw(user.clone(), shard)?)
        } else {
            None
        };

        let token = if let Some(token) = raw.get("token") {
            if let Some(token) = token.as_str() {
                token.into()
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction token".into())))
            }
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction token".into())))
        };

        let app_permissions = if let Some(app_permissions) = raw.get("app_permissions") {
            if let Some(app_permissions) = app_permissions.as_str() {
                Some(app_permissions.into())
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction app_permissions".into())))
            }
        } else {
            None
        };

        let locale = if let Some(locale) = raw.get("locale") {
            if let Some(locale) = locale.as_str() {
                Some(locale.into())
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction locale".into())))
            }
        } else {
            None
        };

        let guild_locale = if let Some(guild_locale) = raw.get("guild_locale") {
            if let Some(guild_locale) = guild_locale.as_str() {
                Some(guild_locale.into())
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction guild_locale".into())))
            }
        } else {
            None
        };


        Ok(Self {
            id,
            application_id,
            interaction_type,
            data,
            guild_id,
            channel,
            channel_id,
            member,
            user,
            token,
            app_permissions,
            locale,
            guild_locale
        })
    }
}

impl Interaction {
    /// Respond to an interaction
    ///
    /// Reference:
    /// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object-interaction-callback-type)
    pub async fn reply(&self, http: &Http, content: MessageBuilder) -> Result<ApiResult<()>> {
        let json = content.to_json();

        http.reply_interaction(
            &self.id,
            &self.token,
            InteractionCallbackType::ChannelMessageWithSource,
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
}

/// Represents the type of an interaction
pub enum InteractionCallbackType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
    ApplicationCommandAutocompleteResult = 8,
    Modal = 9
}

impl InteractionCallbackType {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Pong => 1,
            Self::ChannelMessageWithSource => 4,
            Self::DeferredChannelMessageWithSource => 5,
            Self::DeferredUpdateMessage => 6,
            Self::UpdateMessage => 7,
            Self::ApplicationCommandAutocompleteResult => 8,
            Self::Modal => 9,
        }.into()
    }
}

/// Represents the data of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-structure)
#[derive(Debug, Clone)]
pub struct InteractionData {
    /// The ID of the invoked command
    pub id: Option<Snowflake>,
    /// The name of the invoked command
    pub name: Option<String>,
    /// The type of the invoked command
    pub command_type: Option<ApplicationCommandType>,
    /// ID of the user or message targeted by a user or message command
    pub target_id: Option<Snowflake>,
    /// Converted users + roles + channels + attachments
    pub resolved: Option<InteractionDataResolved>,
    /// The params + values from the user
    pub options: Option<Vec<InteractionDataOption>>,

    /// The type of the component
    pub component_type: Option<ComponentType>,
    /// The custom ID of the component
    pub custom_id: Option<String>
}

impl HttpRessource for InteractionData {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = if let Some(id) = raw.get("id") {
            if let Some(id) = id.as_str() {
                Some(Snowflake::from_raw(id.into(), shard)?)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data id".into())))
            }
        } else {
            None
        };
        let name: Option<String> = if let Some(name) = raw.get("name") {
            if let Some(name) = name.as_str() {
                Some(name.into())
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data name".into())))
            }
        } else {
            None
        };

        let command_type = if let Some(command_type) = raw.get("type") {
            Some(ApplicationCommandType::from_raw(command_type.clone(), shard)?)
        } else {
            None
        };

        let target_id = if let Some(target_id) = raw.get("target_id") {
            if let Some(target_id) = target_id.as_str() {
                Some(Snowflake::from_raw(target_id.into(), shard)?)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data target_id".into())))
            }
        } else {
            None
        };

        let resolved = if let Some(resolved) = raw.get("resolved") {
            Some(InteractionDataResolved::from_raw(resolved.clone(), shard)?)
        } else {
            None
        };

        let options = if let Some(options) = raw.get("options") {
            if let Some(options) = options.as_array() {
                let mut options_vec = Vec::new();
                for option in options {
                    options_vec.push(InteractionDataOption::from_raw(option.clone(), shard)?);
                }
                Some(options_vec)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data options".into())))
            }
        } else {
            None
        };

        let custom_id = if let Some(custom_id) = raw.get("custom_id") {
            if let Some(custom_id) = custom_id.as_str() {
                Some(custom_id.into())
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction custom_id".into())))
            }
        } else {
            None
        };

        let component_type = if let Some(custom_id) = raw.get("component_type") {
            Some(ComponentType::from_raw(custom_id.clone(), shard)?)
        } else {
            None
        };

        Ok(Self {
            id,
            name,
            command_type,
            target_id,
            resolved,
            options,
            component_type,
            custom_id
        })
    }
}

/// Represents the resolved data of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data-resolved-structure)
#[derive(Debug, Clone)]
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

impl HttpRessource for InteractionDataResolved {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let users = if let Some(users) = raw.get("users") {
            if users.is_object() {
                if let Some(users) = users.as_object() {
                    let mut users_map = HashMap::new();
                    for (key, value) in users {
                        if let Ok(key) = UserId::from_raw(key.clone().into(), shard) {
                            match User::from_raw(value.clone(), shard) {
                                Ok(user) => {
                                    users_map.insert(key, user);
                                },
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved users".into())))
                        }
                    }
                    Some(users_map)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved users".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved users".into())))
            }
        } else {
            None
        };

        let members = if let Some(members) = raw.get("members") {
            if members.is_object() {
                if let Some(members) = members.as_object() {
                    let mut members_map = HashMap::new();
                    for (key, value) in members {
                        if let Ok(key) = Snowflake::from_raw(key.clone().into(), shard)  {
                            match GuildMember::from_raw(value.clone(), shard) {
                                Ok(member) => {
                                    members_map.insert(key, member);
                                },
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved members".into())))
                        }
                    }
                    Some(members_map)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved members".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved members".into())))
            }
        } else {
            None
        };

        let roles = if let Some(roles) = raw.get("roles") {
            if roles.is_object() {
                if let Some(roles) = roles.as_object() {
                    let mut roles_map = HashMap::new();
                    for (key, value) in roles {
                        if let Ok(key) = Snowflake::from_raw(key.clone().into(), shard)  {
                            match Role::from_raw(value.clone(), shard) {
                                Ok(role) => {
                                    roles_map.insert(key, role);
                                },
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved roles".into())))
                        }
                    }
                    Some(roles_map)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved roles".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved roles".into())))
            }
        } else {
            None
        };

        let channels = if let Some(channels) = raw.get("channels") {
            if channels.is_object() {
                if let Some(channels) = channels.as_object() {
                    let mut channels_map = HashMap::new();
                    for (key, value) in channels {
                        if let Ok(key) = ChannelId::from_raw(key.clone().into(), shard)  {
                            match Channel::from_raw(value.clone(), shard) {
                                Ok(channel) => {
                                    channels_map.insert(key, channel);
                                },
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved channels".into())))
                        }
                    }
                    Some(channels_map)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved channels".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved channels".into())))
            }
        } else {
            None
        };

        let messages = if let Some(messages) = raw.get("messages") {
            if messages.is_object() {
                if let Some(messages) = messages.as_object() {
                    let mut messages_map = HashMap::new();
                    for (key, value) in messages {
                        if let Ok(key) = Snowflake::from_raw(key.clone().into(), shard)  {
                            match Message::from_raw(value.clone(), shard) {
                                Ok(message) => {
                                    messages_map.insert(key, message);
                                },
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved messages".into())))
                        }
                    }
                    Some(messages_map)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved messages".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved messages".into())))
            }
        } else {
            None
        };

        let attachments = if let Some(attachments) = raw.get("attachments") {
            if attachments.is_object() {
                if let Some(attachments) = attachments.as_object() {
                    let mut attachments_map = HashMap::new();
                    for (key, value) in attachments {
                        if let Ok(key) = Snowflake::from_raw(key.clone().into(), shard)  {
                            match Attachment::from_raw(value.clone(), shard) {
                                Ok(attachment) => {
                                    attachments_map.insert(key, attachment);
                                },
                                Err(e) => return Err(e)
                            }
                        } else {
                            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved attachments".into())))
                        }
                    }
                    Some(attachments_map)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved attachments".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data resolved attachments".into())))
            }
        } else {
            None
        };

        Ok(Self {
            users,
            members,
            roles,
            channels,
            messages,
            attachments
        })
    }
}

/// Represents the options of an interaction
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-application-command-interaction-data-option-structure)
#[derive(Debug, Clone)]
pub struct InteractionDataOption {
    pub name: String,
    pub option_type: ApplicationCommandOptionType,
    pub value: Option<InteractionDataOptionValue>,
    pub options: Option<Vec<InteractionDataOption>>,
    pub focused: Option<bool>,
}

impl HttpRessource for InteractionDataOption {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let name = if let Some(name) = raw.get("name") {
            if name.is_string() {
                name.as_str().unwrap().to_string()
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option name".into())))
            }
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option name".into())))
        };

        let option_type = if let Some(option_type) = raw.get("type") {
            ApplicationCommandOptionType::from_raw(option_type.clone(), shard)?
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option type".into())))
        };

        let value = if let Some(value) = raw.get("value") {
            Some(InteractionDataOptionValue::from_raw(value.clone(), shard)?)
        } else {
            None
        };

        let options = if let Some(options) = raw.get("options") {
            if options.is_array() {
                if let Some(options) = options.as_array() {
                    let mut options_vec = Vec::new();
                    for option in options {
                        options_vec.push(InteractionDataOption::from_raw(option.clone(), shard)?);
                    }
                    Some(options_vec)
                } else {
                    return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option options (cannot get the array)".into())))
                }
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option options (not an array)".into())))
            }
        } else {
            None
        };

        let focused = if let Some(focused) = raw.get("focused") {
            if focused.is_boolean() {
                Some(focused.as_bool().unwrap())
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option focused".into())))
            }
        } else {
            None
        };

        Ok(Self {
            name,
            option_type,
            value,
            options,
            focused,
        })
    }
}

/// Value of the option resulting from the user input
#[derive(Debug, Clone)]
pub enum InteractionDataOptionValue {
    String(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
}

impl HttpRessource for InteractionDataOptionValue {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        if raw.is_string() {
            Ok(Self::String(raw.as_str().unwrap().to_string()))
        } else if raw.is_i64() {
            Ok(Self::Integer(raw.as_i64().unwrap()))
        } else if raw.is_f64() {
            Ok(Self::Double(raw.as_f64().unwrap()))
        } else if raw.is_boolean() {
            Ok(Self::Boolean(raw.as_bool().unwrap()))
        } else {
            Err(Error::Model(ModelError::InvalidPayload("Failed to parse interaction data option value".into())))
        }
    }
}














/// The type of an application command
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-types)
#[derive(Debug, Clone)]
pub enum ApplicationCommandType {
    /// Slash commands; a text-based command that shows up when a user types /
    ChatInput = 1,
    /// Slash commands; a text-based command that shows up when a user types /
    User = 2,
    /// A UI-based command that shows up when you right click or tap on a message
    Message = 3,
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

impl HttpRessource for ApplicationCommandType {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::ChatInput),
            Some(2) => Ok(Self::User),
            Some(3) => Ok(Self::Message),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse application command type".into())))
        }
    }
}


/// Represents a Command
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-structure)
pub struct ApplicationCommand {
    pub id: Snowflake,
    pub command_type: ApplicationCommandType,
    pub application_id: Snowflake,
    pub guild_id: Option<GuildId>,
    pub name: String,
    pub name_localizations: Option<HashMap<String, String>>,
    pub description: String,
    pub description_localizations: Option<HashMap<String, String>>,
    /// Parameters for the command, max of 25
    ///
    /// Available only for ChatInput commands
    pub options: Option<Vec<ApplicationCommandOption>>,
    /// Set of permissions represented as a bit set
    pub default_member_permissions: Option<String>,
    pub dm_permission: bool,
    /// Indicates whether the command is age-restricted, defaults to false
    pub nsfw: bool,
    /// Auto-incrementing version identifier updated during substantial record changes
    pub version: Snowflake,
}

impl HttpRessource for ApplicationCommand {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let command_type = ApplicationCommandType::from_raw(raw["type"].clone(), shard)?;
        let application_id = Snowflake::from_raw(raw["application_id"].clone(), shard)?;
        let guild_id = GuildId::from_raw(raw["guild_id"].clone(), shard).ok();
        let name = raw["name"].as_str().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command name".into())))?.to_string();
        let name_localizations = if let Some(localizations) = raw.get("name_localizations") {
            if localizations.is_null() {
                None
            } else {
                let mut map = HashMap::new();
                for (lang, name) in localizations.as_object().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command name localizations".into())))? {
                    map.insert(lang.clone(), name.as_str().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command name localization".into())))?.to_string());
                }
                Some(map)
            }
        } else {
            None
        };
        let description = raw["description"].as_str().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command description".into())))?.to_string();
        let description_localizations = if let Some(localizations) = raw.get("description_localizations") {
            if localizations.is_null() {
                None
            } else {
                let mut map = HashMap::new();
                for (lang, description) in localizations.as_object().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command description localizations".into())))? {
                    map.insert(lang.clone(), description.as_str().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command description localization".into())))?.to_string());
                }
                Some(map)
            }
        } else {
            None
        };
        let options = raw["options"].as_array().map(|options| {
            options.iter().map(|option| {
                ApplicationCommandOption::from_raw(option.clone(), shard)
            }).collect::<Result<Vec<ApplicationCommandOption>>>()
        }).transpose()?;
        let default_member_permissions = raw["default_permission"].as_str().map(|permissions| permissions.to_string());
        let dm_permission = if let Some(dm_permission) = raw.get("dm_permission") {
            dm_permission.as_bool().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command dm_permission".into())))?
        } else {
            false
        };
        let nsfw = raw["nsfw"].as_bool().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command nsfw".into())))?;
        let version = Snowflake::from_raw(raw["version"].clone(), shard)?;

        Ok(Self {
            id,
            command_type,
            application_id,
            guild_id,
            name,
            name_localizations,
            description,
            description_localizations,
            options,
            default_member_permissions,
            dm_permission,
            nsfw,
            version,
        })
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
pub struct ApplicationCommandOption {
    pub option_type: ApplicationCommandOptionType,
    pub name: String,
    pub name_localizations: Option<HashMap<String, String>>,
    pub description: String,
    pub description_localizations: Option<HashMap<String, String>>,
    pub required: bool,
    pub choices: Option<Vec<ApplicationCommandOptionChoice>>,
    pub options: Option<Vec<ApplicationCommandOption>>,
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
            self.options = Some(Vec::new())
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

impl HttpRessource for ApplicationCommandOption {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let option_type = ApplicationCommandOptionType::from_raw(raw["type"].clone(), shard)?;
        let name = raw["name"].as_str().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command option name".into())))?.to_string();
        let description = raw["description"].as_str().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command option description".into())))?.to_string();
        let required = if let Some(required) = raw.get("required") {
            required.as_bool().ok_or_else(|| Error::Model(ModelError::InvalidPayload("Failed to parse command option required".into())))?
        } else {
            false
        };
        let choices = raw["choices"].as_array().map(|choices| {
            choices.iter().map(|choice| {
                ApplicationCommandOptionChoice::from_raw(choice.clone(), shard)
            }).collect::<Result<Vec<ApplicationCommandOptionChoice>>>()
        }).transpose()?;
        let options = raw["options"].as_array().map(|options| {
            options.iter().map(|option| {
                ApplicationCommandOption::from_raw(option.clone(), shard)
            }).collect::<Result<Vec<ApplicationCommandOption>>>()
        }).transpose()?;
        let channel_types = raw["channel_types"].as_array().map(|channel_types| {
            channel_types.iter().map(|channel_type| {
                ChannelKind::from_raw(channel_type.clone(), shard)
            }).collect::<Result<Vec<ChannelKind>>>()
        }).transpose()?;
        let min_value = raw["min_value"].as_i64().map(OptionValue::Integer);
        let max_value = raw["max_value"].as_i64().map(OptionValue::Integer);
        let min_length = raw["min_length"].as_u64();
        let max_length = raw["max_length"].as_u64();
        let autocomplete = raw["autocomplete"].as_bool();

        Ok(Self {
            option_type,
            name,
            name_localizations: None,
            description,
            description_localizations: None,
            required,
            choices,
            options,
            channel_types,
            min_value,
            max_value,
            min_length,
            max_length,
            autocomplete,
        })
    }
}

/// Used for the min_value and max_value fields of ApplicationCommandOption
pub enum OptionValue {
    Integer(i64),
    Double(f64),
}

impl OptionValue {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Integer(int) => Value::Number(Number::from(*int)),
            Self::Double(double) => Value::Number(Number::from_f64(*double).unwrap_or(Number::from_f64(f64::MIN).expect("Failed to parse f64 min"))),
        }
    }
}

impl HttpRessource for OptionValue {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        if let Some(int) = raw.as_i64() {
            Ok(Self::Integer(int))
        } else if let Some(double) = raw.as_f64() {
            Ok(Self::Double(double))
        } else {
            Err(Error::Model(ModelError::InvalidPayload("Failed to parse option value".into())))
        }
    }
}

/// Represents a command option type
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-type)
#[derive(Debug, Clone)]
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

impl ApplicationCommandOptionType {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::SubCommand => Value::Number(Number::from(1)),
            Self::SubCommandGroup => Value::Number(Number::from(2)),
            Self::String => Value::Number(Number::from(3)),
            Self::Integer => Value::Number(Number::from(4)),
            Self::Boolean => Value::Number(Number::from(5)),
            Self::User => Value::Number(Number::from(6)),
            Self::Channel => Value::Number(Number::from(7)),
            Self::Role => Value::Number(Number::from(8)),
            Self::Mentionable => Value::Number(Number::from(9)),
            Self::Number => Value::Number(Number::from(10)),
            Self::Attachment => Value::Number(Number::from(11)),
        }
    }
}

impl HttpRessource for ApplicationCommandOptionType {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::SubCommand),
            Some(2) => Ok(Self::SubCommandGroup),
            Some(3) => Ok(Self::String),
            Some(4) => Ok(Self::Integer),
            Some(5) => Ok(Self::Boolean),
            Some(6) => Ok(Self::User),
            Some(7) => Ok(Self::Channel),
            Some(8) => Ok(Self::Role),
            Some(9) => Ok(Self::Mentionable),
            Some(10) => Ok(Self::Number),
            Some(11) => Ok(Self::Attachment),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse command option type".into())))
        }
    }
}

/// Represents a command choice
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-choice-structure)
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

impl HttpRessource for ApplicationCommandOptionChoice {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let name = if let Some(name) = raw.get("name") {
            if let Some(name) = name.as_str() {
                name.to_string()
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice name".into())))
            }
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice name".into())))
        };

        let name_localizations = if let Some(name_localizations) = raw.get("name_localizations") {
            if let Some(name_localizations) = name_localizations.as_object() {
                let mut map = HashMap::new();
                for (k, v) in name_localizations {
                    if let Some(v) = v.as_str() {
                        map.insert(k.clone(), v.to_string());
                    } else {
                        return Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice name localizations".into())))
                    }
                }
                Some(map)
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice name localizations".into())))
            }
        } else {
            None
        };

        let value = if let Some(value) = raw.get("value") {
            CommandChoiceValue::from_raw(value.clone(), shard)?
        } else {
            return Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice value".into())))
        };

        Ok(Self {
            name,
            name_localizations,
            value,
        })
    }
}

/// Represents a command choice value
pub enum CommandChoiceValue {
    String(String),
    Integer(i64),
    Double(f64),
}

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

impl HttpRessource for CommandChoiceValue {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw {
            Value::String(s) => Ok(Self::String(s)),
            Value::Number(n) => {
                if n.is_i64() {
                    Ok(Self::Integer(n.as_i64().unwrap()))
                } else if n.is_f64() {
                    Ok(Self::Double(n.as_f64().unwrap()))
                } else {
                    Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice value".into())))
                }
            },
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse command choice value".into())))
        }
    }
}