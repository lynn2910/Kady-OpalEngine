use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use chrono::{DateTime, Utc};
use log::error;
use serde::Serialize;
use tokio::fs;
use uuid::Uuid;
use client::manager::http::Http;
use client::models::channel::ChannelId;
use client::models::components::Color;
use client::models::components::embed::{Embed, Field};
use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
use client::models::guild::GuildId;
use client::models::message::{AttachmentBuilder, MessageAttachmentBuilder, MessageBuilder};
use client::models::user::UserId;


const ERROR_BROADCASTER_CHANNEL: &str = "1154827849048543232";


/// Store the informations about an interaction for the error broadcaster
///
/// # Methods
/// - `default` - Creates a new BroadcastError
/// - `set_guild` - Sets the guild id
/// - `set_channel` - Sets the channel id
#[derive(Default, Debug, Serialize)]
pub struct BroadcastInteraction {
    pub interaction_type: Option<BroadcastInteractionType>,
    pub name: Option<String>
}

impl BroadcastInteraction {
    pub fn set_name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn set_type(mut self, interaction_type: BroadcastInteractionType) -> Self {
        self.interaction_type = Some(interaction_type);
        self
    }
}

/// Defines the type of interaction
#[derive(Debug, Serialize)]
#[allow(unused)]
pub enum BroadcastInteractionType {
    SlashCommand,
    Button,
    Modal,
    SelectMenu
}

impl Display for BroadcastInteractionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            BroadcastInteractionType::SlashCommand => "Slash Command",
            BroadcastInteractionType::Button => "Button",
            BroadcastInteractionType::Modal => "Modal",
            BroadcastInteractionType::SelectMenu => "Select Menu"
        })
    }
}

/// Store the informations about the localisation for the error broadcaster
///
/// # Methods
/// - `default` - Creates a new BroadcastError
/// - `set_guild` - Sets the guild id
/// - `set_channel` - Sets the channel id
/// - `set_user` - Sets the user id
/// - `set_code_path` - Sets the code path
#[derive(Default, Debug, Serialize)]
pub struct BroadcastLocalisation {
    pub guild: Option<GuildId>,
    pub channel: Option<ChannelId>,
    pub user: Option<UserId>,
    pub code_path: Option<String>
}

impl BroadcastLocalisation {
    pub fn set_guild(mut self, guild: Option<impl Into<GuildId>>) -> Self {
        self.guild = guild.map(|g| g.into());
        self
    }

    pub fn set_channel(mut self, channel: Option<impl Into<ChannelId>>) -> Self {
        self.channel = channel.map(|c| c.into());
        self
    }

    pub fn set_user(mut self, user: Option<impl Into<UserId>>) -> Self {
        self.user = user.map(|u| u.into());
        self
    }

    pub fn set_code_path(mut self, code_path: impl ToString) -> Self {
        self.code_path = Some(code_path.to_string());
        self
    }
}

/// Store additional informations about the error for the error broadcaster
///
/// # Methods
/// - `default` - Creates a new BroadcastError
/// - `add` - Adds a new information
#[derive(Default, Debug, Serialize)]
pub struct BroadcastDetails {
    pub informations: HashMap<String, String>
}

impl BroadcastDetails {
    pub fn add(mut self, k: impl ToString, v: impl ToString) -> Self {
        self.informations.insert(k.to_string(), v.to_string());
        self
    }
}


/// Define informations about an error that will be sent to the broadcast channel
///
/// # Methods
/// - `default` - Creates a new BroadcastError
/// - `to_json` - Converts the BroadcastError to a json string
#[derive(Default, Debug, Serialize)]
pub struct BroadcastError {
    pub id: Uuid,
    pub date: DateTime<Utc>,
    pub localisation: BroadcastLocalisation,
    pub details: BroadcastDetails,
    pub interaction: BroadcastInteraction
}

impl BroadcastError {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }


    pub async fn save(&self) -> io::Result<()> {
        fs::create_dir_all("errors/broadcaster/").await?;
        fs::write(
            format!("errors/broadcaster/{}", self.id),
            self.to_json()
        ).await
    }

    #[allow(unused)]
    pub fn set_date(mut self, date: DateTime<Utc>) -> Self {
        self.date = date;
        self
    }

    pub async fn broadcast(&self, http: &Http) {
        if let Err(e) = self.save().await {
            error!(target: "BroadcastErrorHandler", "Cannot save the error '{}' to a file: {e:#?}", self.id)
        };

        let file = AttachmentBuilder {
            bytes: self.to_json().as_bytes().to_vec(),
            content_type: "application/json".into(),
            description: None,
            filename: "report.json".into(),
            id: 0
        };

        let mut builder = MessageBuilder::new().add_component(
            Component::ActionRow(
                ActionRow::new()
                    .add_component(
                        Component::Button(
                            Button::new("DATE")
                                .set_label(self.date.to_rfc3339())
                                .set_style(ButtonStyle::Secondary)
                                .set_disabled(true)
                        )
                    )
            )
        )
        .add_embed(
            Embed::new()
                .set_description("> **New report**")
                .set_color(Color::from_rgb(255, 51, 51))
        )
        .add_attachment(MessageAttachmentBuilder {
            description: None,
            name: "report.json".to_string(),
            content_type: "application/json".to_string(),
            id: 0,
        });

        if !(self.localisation.guild.is_none() && self.localisation.user.is_none() && self.localisation.channel.is_none() && self.localisation.code_path.is_none()) {
            let mut embed = Embed::new()
                .set_title("Localisation")
                .set_color(Color::from_rgb(51, 255, 153));

            if let Some(guild) = &self.localisation.guild {
                embed = embed.add_field(
                    Field::new()
                        .set_name("Guild")
                        .set_value(format!("```\n{guild}```"))
                        .set_inline(false)
                )
            }

            if let Some(user) = &self.localisation.user {
                embed = embed.add_field(
                    Field::new()
                        .set_name("User")
                        .set_value(format!("<@{user}>\n```\n{user}```"))
                        .set_inline(false)
                )
            }

            if let Some(channel) = &self.localisation.channel {
                embed = embed.add_field(
                    Field::new().set_name("Channel")
                        .set_value(format!("<#{channel}>\n```\n{channel}```"))
                        .set_inline(false)
                );
            }

            if let Some(code_path) = &self.localisation.code_path {
                embed = embed.add_field(
                    Field::new()
                        .set_name("Code path")
                        .set_value(format!("```\n{code_path}```"))
                        .set_inline(false)
                );
            }

            builder = builder.add_embed(embed)
        }

        if !self.details.informations.is_empty() {
            let mut embed = Embed::new()
                .set_title("Details")
                .set_color(Color::from_rgb(51, 122, 255));

            for (k, v) in &self.details.informations {
                embed = embed.add_field(
                    Field::new()
                        .set_name(k)
                        .set_value(format!("```\n{v}```"))
                        .set_inline(false)
                );
            }

            builder = builder.add_embed(embed)
        }

        if let Some(interaction_type) = &self.interaction.interaction_type {
            let mut embed = Embed::new()
                .set_title("Interaction")
                .set_color(Color::from_rgb(255, 51, 141));

            if let Some(name) = &self.interaction.name {
                embed = embed.add_field(
                    Field::new()
                        .set_name("Name")
                        .set_value(format!("```\n{name}```"))
                        .set_inline(false)
                );
            }

            embed = embed.add_field(
                Field::new()
                    .set_name("Type")
                    .set_value(format!("```\n{interaction_type}```"))
                    .set_inline(false)
            );

            builder = builder.add_embed(embed)
        }

        let _ = http.send_message(
            &ERROR_BROADCASTER_CHANNEL.into(),
            builder,
            Some(vec![file])
        ).await;
    }
}


// Macro definition to create a new BroadcastError

/// Define the broadcast structure with provided arguments and send it
///
/// The macro cannot called outside of an async runtime (async function or tokio::task)
#[macro_export]
macro_rules! broadcast_error {
    (localisation: $localisation:expr, interaction: $interaction:expr, details: $details:expr, $http:expr) => {
        ($crate::crates::error_broadcaster::BroadcastError {
            id: uuid::Uuid::new_v4(),
            date: chrono::Utc::now(),
            localisation: $localisation,
            details: $details,
            interaction: $interaction
        }).broadcast($http).await;
    };
    (localisation: $localisation:expr, details: $details:expr, $http:expr) => {
        ($crate::crates::error_broadcaster::BroadcastError {
            id: uuid::Uuid::new_v4(),
            date: chrono::Utc::now(),
            localisation: $localisation,
            details: $details,
            interaction: $crate::crates::error_broadcaster::BroadcastInteraction::default()
        }).broadcast($http).await;
    };
    (localisation: $localisation:expr, interaction: $interaction:expr, $http:expr) => {
        ($crate::crates::error_broadcaster::BroadcastError {
            id: uuid::Uuid::new_v4(),
            date: chrono::Utc::now(),
            localisation: $localisation,
            details: $crate::crates::error_broadcaster::BroadcastDetails::default(),
            interaction: $interaction
        }).broadcast($http).await;
    };
    (localisation: $localisation:expr, $http:expr) => {
        ($crate::crates::error_broadcaster::BroadcastError {
            id: uuid::Uuid::new_v4(),
            date: chrono::Utc::now(),
            localisation: $localisation,
            details: $crate::crates::error_broadcaster::BroadcastDetails::default(),
            interaction: $crate::crates::error_broadcaster::BroadcastInteraction::default()
        }).broadcast($http).await;
    };
    (details: $details:expr, interaction: $interaction:expr, $http:expr) => {
        ($crate::crates::error_broadcaster::BroadcastError {
            id: uuid::Uuid::new_v4(),
            date: chrono::Utc::now(),
            localisation: $crate::crates::error_broadcaster::BroadcastLocalisation::default(),
            details: $details,
            interaction: $interaction
        }).broadcast($http).await;
    };
    (details: $details:expr, $http:expr) => {
        ($crate::crates::error_broadcaster::BroadcastError {
            id: uuid::Uuid::new_v4(),
            date: chrono::Utc::now(),
            localisation: $crate::crates::error_broadcaster::BroadcastLocalisation::default(),
            details: $details,
            interaction: $crate::crates::error_broadcaster::BroadcastInteraction::default()
        }).broadcast($http).await;
    };
    ($broadcast_message:expr, $http:expr) => {
        $broadcast_message.broadcast($http).await;
    };
}