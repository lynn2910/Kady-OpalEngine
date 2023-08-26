use serde::{Deserialize, Serialize};
use serde_json::Value;
use error::{Error, ModelError, Result};
use crate::manager::http::HttpRessource;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum StickerFormatType {
    Png = 1,
    Apng = 2,
    Lottie = 3,
    Gif = 4
}

impl HttpRessource for StickerFormatType {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(1) => Ok(Self::Png),
            Some(2) => Ok(Self::Apng),
            Some(3) => Ok(Self::Lottie),
            Some(4) => Ok(Self::Gif),
            _ => Err(Error::Model(ModelError::InvalidPayload("Failed to parse sticker format type".into())))
        }
    }
}