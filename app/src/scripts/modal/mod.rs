use client::models::components::message_components::Component;
use client::models::interaction::InteractionDataOptionValue;
use std::collections::HashMap;

pub(crate) mod cookie_quiz_answer;
pub(crate) mod kady;

pub(self) fn get_modal_textinput(components: &[Component]) -> HashMap<String, InteractionDataOptionValue> {
    let mut texts = HashMap::new();
    for component in components {
        match component {
            Component::ActionRow(row) => {
                let results = get_modal_textinput(&row.components);
                texts.extend(results);
            }
            Component::TextInput(t) => {
                texts.insert(t.custom_id.clone(), t.value.clone().unwrap_or_default());
            },
            _ => {}
        }
    }
    texts
}