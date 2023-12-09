use logs::warn;
use crate::{Language, SEPARATOR, TranslationKey, TranslationNode, TRANSLATIONS};
use crate::fmt::constants::{NO_CHILDREN, NO_LANG, NO_NODE};
use crate::fmt::formatter::Formatter;

/// Get the lang and path to locate the node at the given path
///
/// Ensure a TranslationNode but can be a node with an error in the value
pub fn translate(
    lang: impl Into<Language>,
    path: &str,
    formatter: &Formatter
) -> TranslationNode {
    let translations = TRANSLATIONS.read().expect("Translation object is poisoned");
    let final_node = match translations.get(&lang.into()) {
        Some(node) => {
            if path == "#" {
                node.clone()
            } else {
                let path_vec: Vec<&str> = path.split(SEPARATOR).collect();
                get_node(
                    path_vec,
                    node
                )
            }
        },
        None => {
            return TranslationNode {
                children: None,
                value: Some(NO_LANG.into())
            }
        }
    };

    formatter::format(&path.to_string(), final_node, formatter)
}

/// Retrieve a node (if available) from the path
///
/// Will return None if no node was found at the given path
fn get_node(path: Vec<&str>, node: &TranslationNode) -> TranslationNode {
    if path.is_empty() {
        node.clone()
    } else {
        let key = path.first().unwrap_or(&"");
        if node.children.is_none() { return TranslationNode { children: None, value: Some(NO_CHILDREN.into()) } }
        // get the node
        if let Some(node) = node.children.as_ref().unwrap().get(&TranslationKey(key.to_string())) {
            get_node(
                path[1..].to_vec(),
                node
            )
        } else {
            warn!(target: "Translation", "Translation not found for {path:?}");
            TranslationNode { children: None, value: Some(NO_NODE.into()) }
        }
    }
}

pub mod formatter {
    use std::collections::HashMap;
    use std::fmt::Display;
    use std::ops::Deref;
    use serde_json::Value;
    use crate::{FORMATTING_RG, TranslationNode};

    /// Builder & Abstraction structure to simplify the formatting of translations
    #[derive(Debug, Clone, Default)]
    pub struct Formatter {
        arguments: HashMap<String, String>,
    }

    impl Formatter {
        /// Create a new formatter
        pub fn new() -> Self {
            Self::default()
        }

        /// Builder method to add a value with a key
        ///
        /// Doesn't update the value at the key emplacement is the value is already set
        pub fn add<'a>(&mut self, key: impl Into<&'a str>, value: impl Display) -> &mut Self {
            let k = key.into().to_string();
            if self.arguments.get(&k).is_none() {
                self.arguments.insert(k, value.to_string());
            };
            self
        }

        /// Builder method to add a value with a key
        ///
        /// Will update the value at the key emplacement is the value is already set,
        /// otherwise will insert the new value
        pub fn update<T: ToString>(&mut self, key: T, value: T) -> &mut Self {
            self.arguments.insert(key.to_string(), value.to_string());
            self
        }
    }


    pub(super) fn format(path: &String, mut source: TranslationNode, formatter: &Formatter) -> TranslationNode {
        if let Some(v) = source.value.as_mut() {
            source.value = Some(format_value(path, v.deref().to_owned(), formatter));
        }
        if let Some(childrens) = source.children.as_mut() {
            for (_, v) in childrens.iter_mut() {
                *v = format(path, v.deref().to_owned(), formatter);
            }
        };
        source
    }

    #[allow(clippy::only_used_in_recursion)]
    fn format_value(path: &String, mut source: Value, formatter: &Formatter) -> Value {
        if source.is_object() {
            let entries = source.as_object_mut().unwrap();
            for (_, v) in entries.iter_mut() {
                *v = format_value(path, v.clone(), formatter);
            };
            return source
        } else if source.is_array() {
            let array = source.as_array_mut().unwrap();
            for v in array {
                *v = format_value(path, v.clone(), formatter)
            };
            return source
        } else if source.is_string() {
            return Value::String(format_string(source.as_str().unwrap(), formatter))
        }
        source
    }

    fn format_string(from: &str, formatter: &Formatter) -> String {
        FORMATTING_RG.replace_all(from, |caps: &regex::Captures| -> String {
            let k: &str = caps[1].as_ref();
            if let Some(v) = formatter.arguments.get(k) { v.clone() }
            else { format!("{{{t}}}", t = k) }
        }).to_string()
    }
}

mod constants {
    pub(super) const NO_LANG: &str = "INVALID_LANG";
    pub(super) const NO_NODE: &str = "NO_NODE_FOUND";
    pub(super) const NO_CHILDREN: &str = "NO_CHILDREN_FOUND";
}