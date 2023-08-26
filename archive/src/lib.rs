//! Ce module est basé sur le protocole d'archives `PNDR`
//! Ce protocole a été conçu par [lilevil](https://github.com/lil-evil)

use std::path::PathBuf;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{ Serialize, Deserialize };
use error::{ Result, Error, ArchiveError, FileError };
use crate::constants::{BLOAT, check_magic1, check_magic2, MAGIC1, MAGIC2, OWNER_PID, VERSION};
use crate::security::{bytes_to_string, read_file};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveDataFormat {
    Toml = 0,
    Json = 1,
    Yaml = 2,
}

impl From<ArchiveDataFormat> for u8 {
    fn from(value: ArchiveDataFormat) -> u8 {
        match value {
            ArchiveDataFormat::Toml => 0,
            ArchiveDataFormat::Json => 1,
            ArchiveDataFormat::Yaml => 2,
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct Archive {
    /// The path of the archive
    path: PathBuf,
    /// The format of the informations stored in the archive
    data_type: ArchiveDataFormat,
    /// A bloat of data to make the archive more difficult to read
    bloat: String,
    /// The size of the data
    data_size: u64,
    /// The version used to write the archive
    pub version: String,
    /// The date of creation of the archive
    pub creation: DateTime<Utc>,
    /// The date of the last modification of the archive
    pub last_modification: DateTime<Utc>,
    /// The pid of the user who created the archive
    pub owner_pid: u64,
    /// The data stored in the archive
    body: serde_json::Value
}

impl Archive {
    /// Use Self to format a String that contain the formatted header
    fn format_header(&self) -> String {
        [
            "{".to_string(),
            format!(
                "data_type={},bloat={},data_size={},version={},creation={:?},last_modification={:?},owner_pid={}",
                self.data_type.clone() as u8,
                self.bloat,
                self.data_size,
                self.version,
                self.creation.timestamp_millis(),
                self.last_modification.timestamp_millis(),
                self.owner_pid
            ),
            "}".to_string()
        ].concat()
    }

    /// Use Self to format a String that contain the formatted body
    fn format_body(&self) -> Result<String> {
        match serde_json::to_string(&self.body) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::Archive(ArchiveError::CannotSerializeBody(format!("{:?}", e))))
        }
    }

    /// Encrypt the archive
    fn encrypt(&self) -> Result<Vec<u8>> {
        let encrypted_header = self.format_header();

        let header_size = encrypted_header.len() as u64;
        let encrypted_body = self.format_body()?;
        let encrypted: &mut [u8] = &mut [&MAGIC2[..], encrypted_header.as_bytes(), encrypted_body.as_bytes()].concat()[..];
        security::encrypt(encrypted, header_size);
        Ok([&MAGIC1[..], format!("{header_size}").as_bytes(), b":", encrypted].concat())
    }

    /// Save the archive
    pub fn save(&self) -> Result<()> {
        match std::fs::write(&self.path, self.encrypt()?) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Fs(FileError::CannotWriteFile(e.to_string())))
        }
    }

    /// Create a new archive with the given path and data type
    ///
    /// Will automatically save the archive
    pub fn create(path: PathBuf, data_type: ArchiveDataFormat) -> Self {
        let body = serde_json::Value::Object(serde_json::Map::new());

        let arch = Self {
            path, data_type, body,
            bloat: BLOAT.to_string(),
            data_size: 0,
            version: VERSION.into(),
            creation: Utc::now(),
            last_modification: Utc::now(),
            owner_pid: OWNER_PID,
        };

        let _ = arch.save();
        arch
    }

    /// Open an archive
    pub fn open(path: PathBuf) -> Result<Self> {
        // read file
        let raw = read_file(&path)?;

        // check if the magic1 is present
        if !check_magic1(&raw) { return Err(Error::Archive(ArchiveError::CorruptedArchive("Magic1 wasn't found".into()))); }
        let raw = &mut raw[MAGIC1.len()..].to_vec();

        let (header_position, header_size) = security::get_header_position(raw)?;

        let decrypted = &mut raw[(header_position + 1)..];

        security::decrypt(decrypted, header_size);

        // check if the magic2 is present
        if !check_magic2(decrypted) { return Err(Error::Archive(ArchiveError::CorruptedArchive("Magic2 wasn't found in the decrypted informations".into()))); }

        let decrypted = &mut decrypted[MAGIC2.len()..];
        let header = {
            let raw = match bytes_to_string(&decrypted[..header_size as usize]) {
                Ok(s) => s,
                Err(e) => return Err(Error::Archive(ArchiveError::CorruptedArchive(format!("Cannot convert the header to a string: {:?}", e)))),
            };

            match parser::parse_header(&raw) {
                Ok(h) => h,
                Err(e) => return Err(Error::Archive(ArchiveError::InvalidHeader(format!("Cannot parse the header: {:?}", e)))),
            }
        };

        // get headers informations-
        let bloat = if let Some(raw_bloat) = header.get("bloat") { get_bloat(raw_bloat)? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The bloat is missing".into()))); };
        if bloat != BLOAT { return Err(Error::Archive(ArchiveError::CorruptedArchive("The bloat is invalid".into()))); }

        let version = if let Some(raw_version) = header.get("version") { get_version(raw_version)? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The version is missing".into()))); };
        let data_type = if let Some(raw_data_type) = header.get("data_type") { get_data_type(raw_data_type.clone())? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The data type is missing".into()))); };
        let data_size = if let Some(raw_data_size) = header.get("data_size") { get_data_size(raw_data_size)? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The data size is missing".into()))); };
        let creation = if let Some(raw_creation) = header.get("creation") { get_creation(raw_creation)? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The creation date is missing".into()))); };
        let last_modification = if let Some(raw_last_modification) = header.get("last_modification") { get_last_modification(raw_last_modification)? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The last modification date is missing".into()))); };
        let owner_pid = if let Some(raw_owner) = header.get("owner_pid") { get_owner_pid(raw_owner)? } else { return Err(Error::Archive(ArchiveError::InvalidHeader("The owner is missing".into()))); };

        let body = {
            let raw = match bytes_to_string(&decrypted[(header_size) as usize..]) {
                Ok(s) => s,
                Err(e) => return Err(Error::Archive(ArchiveError::CorruptedArchive(format!("Cannot convert the body to a string: {:?}", e)))),
            };

            match parser::parse_body(data_type.clone(), &raw) {
                Ok(b) => b,
                Err(e) => return Err(Error::Archive(ArchiveError::InvalidBody(format!("Cannot parse the body: {:?}", e)))),
            }
        };

        Ok(Self {
            body, data_type, version, bloat, path, data_size, creation, last_modification, owner_pid
        })
    }

    /// Get the raw value from the archive
    pub fn get_raw(&self, key: &str) -> Option<&serde_json::Value> {
        self.body.get(key)
    }

    /// Get a value from the archive and deserialize it
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        match self.body.get(key) {
            Some(v) => match serde_json::from_value(v.clone()) {
                Ok(v) => Some(v),
                Err(_) => None,
            },
            None => None,
        }
    }

    /// Set a value in the archive
    pub fn set_raw(&mut self, key: &str, value: serde_json::Value) {
        self.body[key] = value
    }

    /// Set a value in the archive and serialize it
    pub fn set<T: serde::Serialize>(&mut self, key: &str, value: T) -> Result<()>  {
        match serde_json::to_value(value) {
            Ok(v) => {
                self.set_raw(key, v);
                Ok(())
            },
            Err(e) => Err(Error::Archive(ArchiveError::InvalidBodyValue(format!("Cannot serialize the value: {:?}", e)))),
        }
    }

    #[cfg(feature = "total_access")]
    pub fn copy_body(&self) -> serde_json::Value { self.body.clone() }

    #[cfg(feature = "total_access")]
    pub fn unsafe_set(&mut self, body: serde_json::Value) { self.body = body; }
}

fn get_data_type(from: serde_json::Value) -> Result<ArchiveDataFormat> {
    match from.as_str() {
        Some("0") => Ok(ArchiveDataFormat::Toml),
        Some("1") => Ok(ArchiveDataFormat::Json),
        Some("2") => Ok(ArchiveDataFormat::Yaml),
        _ => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The data type is invalid: {:?}", from)))),
    }
}

fn get_version(from: &serde_json::Value) -> Result<String> {
    match from {
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The version is invalid: {:?}", from)))),
    }
}

fn get_bloat(from: &serde_json::Value) -> Result<String> {
    match from.as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The bloat is invalid: {:?}", from)))),
    }
}

fn get_data_size(from: &serde_json::Value) -> Result<u64> {
    match from.as_str() {
        Some(s) => match s.parse::<u64>() {
            Ok(n) => Ok(n),
            Err(e) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The data size is invalid: {:?}", e)))),
        },
        None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The data size is invalid: {:?}", from)))),
    }
}

fn get_creation(from: &serde_json::Value) -> Result<DateTime<Utc>> {
    match from.as_str() {
        Some(s) => {
            match s.parse::<u64>() {
                Ok(d) => {
                    match Utc.timestamp_millis_opt(d as i64) {
                        LocalResult::Single(d) => Ok(d),
                        LocalResult::Ambiguous(_, _) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The creation date is ambiguous: {:?}", from)))),
                        LocalResult::None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The creation date is invalid: {:?}", from)))),
                    }
                },
                Err(e) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The creation date is invalid: {:?}", e)))),
            }
        },
        None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The creation date is invalid: {:?}", from)))),
    }
}

fn get_last_modification(from: &serde_json::Value) -> Result<DateTime<Utc>> {
    match from.as_str() {
        Some(s) => {
            match s.parse::<u64>() {
                Ok(d) => {
                    match Utc.timestamp_millis_opt(d as i64) {
                        LocalResult::Single(d) => Ok(d),
                        LocalResult::Ambiguous(_, _) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The last modification date is ambiguous: {:?}", from)))),
                        LocalResult::None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The last modification date is invalid: {:?}", from)))),
                    }
                },
                Err(e) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The last modification date is invalid: {:?}", e)))),
            }
        },
        None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The last modification date is invalid: {:?}", from)))),
    }
}

fn get_owner_pid(from: &serde_json::Value) -> Result<u64> {
    match from.as_str() {
        Some(s) => match s.parse::<u64>() {
            Ok(n) => Ok(n),
            Err(e) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The owner is invalid: {:?}", e)))),
        },
        None => Err(Error::Archive(ArchiveError::InvalidHeader(format!("The owner is invalid: {:?}", from)))),
    }
}

/// A box that contain a lot of tools to work with archives
///
/// Contain:
/// - `parse_header` to parse the header of an archive
/// - `parse_body` to parse the body of an archive
mod parser {
    use std::collections::HashMap;
    use error::{ArchiveError, Error, Result};
    use crate::ArchiveDataFormat;

    /// Parse the header of the archive
    pub(super) fn parse_header(origin: &str) -> Result<HashMap<String, serde_json::Value>> {
        let mut header = HashMap::new();

        {
            let mut key = String::new();
            let mut value = String::new();
            let mut search_key = true;

            for c in origin.split("") {
                match c {
                    "{" | "}" => continue,
                    // "=" is used for the format `key=value`
                    "=" => {
                        if search_key {
                            search_key = false;
                        } else {
                            return Err(Error::Archive(ArchiveError::InvalidHeader(format!("Unexpected '=' at position {}", origin.find(c).unwrap()))));
                        }
                    }
                    // "," is used for the format `key=value, key=value`
                    "," => {
                        if !search_key {
                            search_key = true;
                            header.insert(key.clone(), serde_json::Value::String(value.clone()));
                            key.clear();
                            value.clear();
                        } else {
                            return Err(Error::Archive(ArchiveError::InvalidHeader(format!("Unexpected ',' at position {}", origin.find(c).unwrap()))));
                        }
                    }
                    _ => {
                        if search_key {
                            key.push_str(c);
                        } else {
                            value.push_str(c);
                        }
                    }
                }
            }

            if !key.is_empty() && !value.is_empty() && !search_key {
                header.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }

        Ok(header)
    }

    /// Parse the body of the archive
    pub(super) fn parse_body(data_type: ArchiveDataFormat, origin: &str) -> Result<serde_json::Value> {
        match data_type {
            ArchiveDataFormat::Json => {
                match serde_json::from_str(origin) {
                    Ok(body) => Ok(body),
                    Err(e) => Err(Error::Archive(ArchiveError::InvalidBody(e.to_string())))
                }
            }
            _ => unimplemented!("The data type {:?} is not implemented yet", data_type)
        }
    }
}

/// A container that store useful values
mod constants {
    /// The size of the bloat
    pub(super) const BLOAT: &str = "e2njgwzZVvCHVF9E9eLYMM9mqPgcqBOUNIocKvEzcftOAc9fywad3nOdnW5HgpmAMefcc4gDOrvXyVDCtxoVMp5oD1J2s9GbHK6SRCsOeQF74Dyl2yKXiTbmGyiMh90QfniFhf3QmZh1G6vAigmu514lZ1DIeRfMkhgKVYjq3wLkFlvoNdoLXlx6dIDRAnE3";
    /// The version of the pndr that is implemented
    pub(super) const VERSION: &str = "1.1.0";
    /// The owner pid
    pub(super) const OWNER_PID: u64 = 0;

    /// A bloat of data to make the archive more difficult to read
    pub(super) const BLEP: u64 = 0x15;

    pub(super) const MAGIC1: [u8; 5] = [127u8, 69u8, 42u8, 68u8, 127u8];
    pub(super) const MAGIC2: [u8; 5] = [127u8, 85u8, 42u8, 68u8, 127u8];

    pub(super) fn check_magic1(bytes: &[u8]) -> bool {
        if bytes.len() < MAGIC1.len() { return false; }
        bytes[0..MAGIC1.len()] == MAGIC1[..]
    }
    pub(super) fn check_magic2(bytes: &[u8]) -> bool {
        if bytes.len() < MAGIC2.len() { return false; }
        bytes[0..MAGIC2.len()] == MAGIC2[..]
    }
}

/// A container that store security functions
mod security {
    use std::path::PathBuf;
    use error::{Result, Error, FileError, ArchiveError};
    use crate::constants::BLEP;

    /// Encrypt the data
    /// Will modify the reference
    pub(super) fn encrypt(data: &mut [u8], header_size: u64) {
        for i in 0..(data.len() as u64) {
            if i % 2 == 0 {
                data[i as usize] += ((header_size + i) % BLEP) as u8
            } else {
                data[i as usize] -= ((header_size + i) % BLEP) as u8
            }
        }
    }

    /// Decrypt the data
    /// Will modify the reference
    pub(super) fn decrypt(data: &mut [u8], header_size: u64) {
        for i in 0..(data.len() as u64) {
            if i % 2 == 0 {
                data[i as usize] -= ((header_size + i) % BLEP) as u8
            } else {
                data[i as usize] += ((header_size + i) % BLEP) as u8
            }
        }
    }

    /// Encrypt the data
    pub(super) fn read_file(path: &PathBuf) -> Result<Vec<u8>> {
        match std::fs::read(path) {
            Ok(data) => Ok(data),
            Err(e) => Err(Error::Fs(FileError::IOError(e.to_string())))
        }
    }

    pub(super) fn bytes_to_string(from: &[u8]) -> Result<String> {
        match std::str::from_utf8(from) {
            Ok(data) => Ok(data.to_string()),
            Err(e) => Err(Error::Archive(ArchiveError::InvalidUtf8Sequence(e.to_string())))
        }
    }

    /// Read the bytes to get the header size
    pub(super) fn get_header_position(source: &[u8]) -> Result<(usize, u64)> {
        let data = source.iter().copied()
            .map(|c| std::str::from_utf8(&[c]).unwrap_or(format!("{c:?}").as_str()).to_string())
            .collect::<String>();

        let header_size_position = data.find(':').unwrap_or(0);

        if header_size_position < 1 {
            return Err(Error::Archive(ArchiveError::InvalidHeader("Header size is invalid".to_string())))
        }

        let header_size = &data[..header_size_position];

        match header_size.parse::<u64>() {
            Ok(size) => Ok((header_size_position, size)),
            Err(e) => Err(Error::Archive(ArchiveError::InvalidHeader(format!("Header size is invalid: {}", e))))
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use crate::{Archive, ArchiveDataFormat};

    #[test]
    fn create() {
        let archive = Archive::create(Path::new("test.pndr").to_path_buf(), ArchiveDataFormat::Json);

        archive.save().unwrap();

        assert!(archive.version == "1.1.0");
    }

    #[test]
    fn read() {
        let archive = Archive::open(Path::new("test.pndr").to_path_buf()).unwrap();

        assert!(archive.version == "1.1.0");
    }

    #[test]
    fn play_with_body(){
        {
            let mut archive = Archive::open(Path::new("test.pndr").to_path_buf()).unwrap();
            archive.set::<String>("Hello", "World".into()).unwrap();
            archive.save().unwrap();
        }
        {
            let archive = Archive::open(Path::new("test.pndr").to_path_buf()).unwrap();
            assert!(archive.get::<String>("Hello").unwrap() == "World");
        }
    }
}