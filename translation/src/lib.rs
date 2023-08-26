//! This module contain the translation system
//!
//! The system is structured with a binary tree that can contain more than 2 branches
//!
//! Each language has his own folder that contain obligatory a file named `declaration.json` as follow:
//! ```json
//! {
//!   "name": "Francais",
//!   "locale": "fr"
//! }
//! ```
//!
//! In parallel, these folders can contain multiples JSON files who contain the followed structure:
//! ```json
//! {
//!     "path": "...",
//!     "data": {...}
//! }
//! ```
//!
//! Please note that the path is each branch names who are separated with `#`
//! The root is always #
//! Example:
//! `#command#help`

mod parser;
pub mod macros;
pub mod fmt;

use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use logs::error;
use serde_json::Value;
use error::Result;
use regex::Regex;
use crate::parser::parse_lang_files;

const SEPARATOR: &str = "::";

/// Represents a translation key.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct TranslationKey(String);

/// Represent a language.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Language(pub String);

impl From<String> for Language {
    fn from(s: String) -> Self {
        Language(s)
    }
}

impl From<&String> for Language {
    fn from(s: &String) -> Self {
        Language(s.deref().to_string())
    }
}

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        Language(s.to_string())
    }
}

impl From<&Language> for Language {
    fn from(s: &Language) -> Self {
        s.clone()
    }
}

/// Represents a translation node.
///
/// When a node has children, it is a directory.
/// Otherwise, it is a sheet.
///
/// This system is based on the concept of a binary tree.
#[derive(Debug, Clone, Default)]
pub struct TranslationNode {
    children: Option<HashMap<TranslationKey, TranslationNode>>,
    value: Option<Value>
}

impl ToString for TranslationNode {
    fn to_string(&self) -> String {
        if let Some(value) = &self.value {
            value.as_str().unwrap_or("TRANSLATION_CONVERSION_ERROR").to_string()
        } else {
            "NO_VALUE_IN_NODE".to_string()
        }
    }
}

impl From<TranslationNode> for String {
    fn from(value: TranslationNode) -> Self {
        if let Some(string) = value.value {
            string.as_str().unwrap_or("TRANSLATION_CONVERSION_ERROR").to_string()
        } else {
            "NO_VALUE_IN_NODE".to_string()
        }
    }
}

impl From<TranslationNode> for Value {
    fn from(node: TranslationNode) -> Self {
        if let Some(value) = node.value {
            value
        } else {
            let mut map = serde_json::Map::new();

            for (key, value) in node.children.unwrap_or(HashMap::new()) {
                map.insert(key.0, Value::from(value));
            }

            Value::Object(map)
        }
    }
}

lazy_static! {
    pub static ref TRANSLATIONS: Arc<RwLock<HashMap<Language, TranslationNode>>> = Arc::new(RwLock::new(HashMap::new()));

    static ref FORMATTING_RG: Regex = Regex::new(r"(?:\{([^{}]+)\})").unwrap();
}

pub fn load_translations(dir: &Path) -> Result<()> {
    let langs = parser::get_langs_dir(dir)?;

    for lang in langs {
        // for each lang, we parse it
        let files = parser::get_files(Path::new(dir).join(lang.clone()).as_path())?;

        match parse_lang_files(files) {
            Ok(node) => {
                let mut translations = TRANSLATIONS.write().expect("Failed to get write lock on TRANSLATIONS");
                if let Some(lang) = translations.get_mut(&Language(lang.clone())) {
                    *lang = node;
                } else {
                    translations.insert(Language(lang), node);
                }
            },
            Err(e) => {
                error!(target: "Translation", "{:?}", e)
            }
        }
    }

    Ok(())
}