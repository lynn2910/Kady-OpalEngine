use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use logs::{error, warn};
use regex::Regex;
use serde_json::Value;
use error::{Error, FileError, Result};
use crate::{SEPARATOR, TranslationKey, TranslationNode};

/// Return a list of all languages directories in the given folder
pub(crate) fn get_langs_dir(dir: &Path) -> Result<Vec<String>> {
    let mut langs = Vec::new();
    let folder = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => return Err(Error::Fs(FileError::CannotReadDir(e.to_string())))
    };

    for entry in folder.flatten() {
        // use "check_extension" function
        let directory = entry.path();
        if directory.is_dir() {
            if let Some(file) = entry.file_name().to_str() {
                langs.push(file.to_string())
            }
        }
    }
    Ok(langs)
}

/// Return all files with the extension "json" inside a directory
pub(crate) fn get_files(dir: &Path) -> Result<Vec<String>> {
    // Check if the path is a directory
    if !dir.is_dir() { return Err(Error::Fs(FileError::InvalidPath("Not a directory".into()))) }


    let mut files = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(etr) => etr,
        Err(e) => return Err(Error::Fs(FileError::CannotReadDir(e.to_string())))
    };

    for file in entries {
        let file = match file {
            Ok(f) => f,
            Err(e) => return Err(Error::Fs(FileError::IOError(e.to_string())))
        };

        // If this is a folder, we get each file INSIDE
        if file.path().is_dir() {
            for entry in get_files(file.path().as_path())? {
                files.push(entry)
            }
        } else if file.path().extension() == Some(OsStr::new("json")) {
            if let Some(path) = file.path().to_str() {
                files.push(path.to_string())
            }
        } else {
            warn!(target: "Translation", "A unknown file was found inside {:?}: {:?}", file.path(), file.path().extension())
        }
    }

    Ok(files)
}

/// Check if a file is in a valid format
fn check_file_integrity(content: &Value) -> bool {
    if let Some(path) = content.get("path") {
        if !check_path(path.as_str().unwrap_or("")) { return false }
    } else { return false }

    if content.get("data").is_none() {
        return false
    }
    true
}

/// Check if a path is valid or not
fn check_path(path: &str) -> bool {
    let reg: Regex = Regex::new(r"^(#|(\w+(?:::\w+)*))$").unwrap();
    reg.is_match(path)
}



/// Read each files provided to get the JSON Value of each one
pub(crate) fn get_files_content(files: &Vec<String>) -> Result<Vec<Value>> {
    let mut contents = Vec::new();
    for dir in files {
        match std::fs::read_to_string(dir.clone()) {
            Ok(cnt) => {
                match serde_json::from_str::<Value>(cnt.as_str()) {
                    Ok(v) => {
                        if check_file_integrity(&v) || is_declaration_file(&v) {
                            contents.push(v)
                        } else {
                            error!(target: "Translation", "Invalid content at {:?}", dir)
                        }
                    },
                    Err(e) => return Err(Error::Fs(FileError::CannotReadFile(format!("{:?}", (dir, e.to_string())))))
                }
            },
            Err(e) => return Err(Error::Fs(FileError::IOError(format!("{:?}", (dir, e.to_string())))))
        }
    };
    Ok(contents)
}

fn manage_declaration(node: &mut TranslationNode, data: Value) {
    if node.children.is_none() {
        node.children = Some(HashMap::new())
    }

    if let Some(children) = node.children.as_mut() {
        children.insert(
            TranslationKey("__name".to_string()),
            TranslationNode { children: None, value: Some(data["name"].clone()) }
        );
        children.insert(
            TranslationKey("__locale".to_string()),
            TranslationNode { children: None, value: Some(data["locale"].clone()) }
        );
    }
}

fn is_declaration_file(d: &Value) -> bool {
    d.get("name").is_some() && d.get("locale").is_some() && d.get("path").is_none() && d.get("data").is_none()
}

pub(crate) fn parse_lang_files(files: Vec<String>) -> Result<TranslationNode> {
    let datas = get_files_content(&files)?;
    let mut lang = TranslationNode::default();

    // We add each value to the node
    for data in datas {
        if is_declaration_file(&data) {
            manage_declaration(&mut lang, data);
            continue
        }

        let path = data.get("path").unwrap().as_str().unwrap().to_string();
        if path.trim() != "#" {
            push_translation(
                &mut lang,
                path.replace('#', "").split(SEPARATOR).collect(),
                data.get("data").unwrap().clone()
            )
        } else {
            error!(target: "Translation", "Root file aren't supported yet");
            warn!(target: "Translation", "This information will be ignored: {:#?}", data);
        }
    }

    Ok(lang)
}

/// Add the datas to the node with a path
pub(crate) fn push_translation(node: &mut TranslationNode, path: Vec<&str>, data: Value) {
    if path.len() > 1 {
        let key = if let Some(k) = path.first() { TranslationKey(k.to_string()) } else { return; };
        // we ensure that we can add branches
        if node.children.is_none() { node.children = Some(HashMap::new()) }
        // we add the node to the current key
        node.children.as_mut().unwrap().entry(key.clone()).or_insert(TranslationNode::default());

        // The path is greater than 1 branch, it's recursive time
        if let Some(children) = node.children.as_mut() {
            push_translation(
                children.get_mut(&key).unwrap(),
                path[1..].to_vec(),
                data
            )
        }
    } else {
        let key = if let Some(k) = path.first() { TranslationKey(k.to_string()) } else { return; };
        // we ensure that we can add branches
        if node.children.is_none() {
            node.children = Some(HashMap::new())
        }
        // we add it
        if let Some(children) = &mut node.children {
            let mut child_node = TranslationNode::default();
            add_to_node(&mut child_node, &key, &data);

            if let Some(child_node_children) = child_node.children.as_ref() {
                if let Some(child) = child_node_children.get(&key) {
                    children.insert(key.clone(), child.clone());
                }
            }
        }
    }
}

/// Transform a JSON into a beautiful Binary Tree
pub(crate) fn add_to_node(node: &mut TranslationNode, key: &TranslationKey, data: &Value) {
    if node.children.is_none() {
        node.children = Some(HashMap::new())
    }

    if data.is_object() {
        let entries = data.as_object().unwrap();

        node.children.as_mut().unwrap().entry(key.clone()).or_insert(TranslationNode::default());

        let new_node = node.children.as_mut().unwrap().get_mut(key).unwrap();

        for (k, v) in entries.iter() {
            add_to_node(new_node, &TranslationKey(k.to_owned()), v)
        }
    } else if data.is_string() || data.is_boolean() || data.is_number() || data.is_null() || data.is_array() {
        node.children.as_mut().unwrap().insert(
            key.clone(),
            TranslationNode {
                children: None,
                value: Some(data.clone())
            }
        );
    };
}