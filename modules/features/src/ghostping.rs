use std::collections::HashMap;
use client::models::message::Message;
use client::models::user::UserId;

/// Represents a ghost ping object
pub struct GhostPing {
    /// Contains each user that was mentioned in the message
    pub mentions: Vec<Mention>
}

impl GhostPing {
    pub fn from_message(message: &Message) -> Option<Self> {
        if message.mentions.is_empty() {
            return None;
        }

        let mut mentions: HashMap<UserId, Mention> = HashMap::new();

        for mention in message.mentions.iter() {
            // skip bots
            if mention.bot.unwrap_or(false) {
                continue;
            }

            if mentions.contains_key(&mention.id) {
                let mention = mentions.get_mut(&mention.id).unwrap();
                mention.count += 1;
            } else {
                mentions.insert(mention.id.clone(), Mention {
                    user: mention.id.clone(),
                    count: 1
                });
            }
        }

        Some(Self { mentions: mentions.drain().map(|(_, mention)| mention).collect() })
    }

    pub fn is_single_mention(&self) -> bool {
        self.mentions.len() == 1
    }
}

/// Represents a mention of a user
pub struct Mention {
    /// The user that was mentioned
    pub user: UserId,
    /// The amount of times the user was mentioned
    pub count: u32
}
