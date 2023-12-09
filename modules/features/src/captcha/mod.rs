use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::prelude::SliceRandom;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use uuid::Uuid;
use client::typemap::Type;
use crate::captcha::generator::Difficulty;
use crate::captcha::generator::fonts::Font;

pub mod generator;

/// The default time in seconds after which a captcha instance will be deleted.
///
/// Expressed in seconds.
///
/// Each captcha instance will be unvalidated after this time, aka 60 seconds.
const DEFAULT_TIMEOUT: u64 = 60;

/// This structure will contain each captcha instance and its data.
#[derive(Clone)]
pub struct CaptchaContainer {
    /// The captcha instances, mapped by their unique identifier.
    instances: Arc<RwLock<HashMap<Uuid, CaptchaInstance>>>,
    /// The task that will clean up expired captcha instances.
    #[allow(unused)]
    cleaner: Arc<JoinHandle<()>>
}

impl Type for CaptchaContainer {
    type Value = Self;
}

impl CaptchaContainer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let timeout = Arc::new(RwLock::new(Duration::from_secs(DEFAULT_TIMEOUT)));
        let instances = Arc::new(RwLock::new(HashMap::new()));

        Self {
            cleaner: Arc::new(
                Self::cleaner_task(instances.clone(), timeout.clone())
            ),
            instances
        }
    }

    fn cleaner_task(
        instances: Arc<RwLock<HashMap<Uuid, CaptchaInstance>>>,
        timeout: Arc<RwLock<Duration>>
    ) -> JoinHandle<()>
    {
        tokio::spawn(async move {
            loop {
                // we update each second, so we don't need to wait for the timeout
                sleep(Duration::from_secs(1)).await;

                let mut instances = instances.write().await;
                let timeout = *timeout.read().await;

                // remove all expired instances
                instances.retain(|_, instance| instance.creation.elapsed() < timeout);

                drop(instances)
            }
        })
    }

    /// Insert a new captcha instance into the container and return its unique identifier.
    pub async fn push_new_instance(&mut self, code: Vec<char>, user: String) -> Uuid {
        let mut instances = self.instances.write().await;

        // generate a new unique identifier
        let mut id = Uuid::new_v4();
        while instances.contains_key(&id) {
            id = Uuid::new_v4();
        }

        // push the new instance
        instances.insert(id, CaptchaInstance {
            id,
            code,
            user,
            creation: Instant::now()
        });

        id
    }

    pub async fn remove_instance(&mut self, id: Uuid) {
        let mut instances = self.instances.write().await;
        instances.remove(&id);
    }

    pub async fn get_instance_from_user(&self, user: String) -> Option<Uuid> {
        let instances = self.instances.read().await;
        let instance = instances.iter().find(|(_, i)| i.user == user);

        instance.map(|(id, _)| id).copied()
    }

    pub async fn get(&self, id: impl Into<Uuid>) -> Option<CaptchaInstance> {
        let instances = self.instances.read().await;
        instances.get(&id.into()).cloned()
    }
}

/// This structure will contain the data for a captcha instance.
#[derive(Clone)]
pub struct CaptchaInstance {
    /// The unique identifier for this captcha instance.
    pub id: Uuid,
    /// The code that the user must enter to pass the captcha.
    pub code: Vec<char>,
    /// The time at which this captcha was created.
    creation: Instant,
    /// The ID of the user that requested it
    pub user: String
}



pub fn generate_random_code_chunk(nb: usize, each_length: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();

    let f: Box<dyn Font> = Box::new(generator::fonts::Default::new());
    let available_chars = f.chars();

    let mut chunk = Vec::new();

    for _ in 0..nb {
        let mut code = Vec::new();

        for _ in 0..each_length {
            if let Some(c) = available_chars.choose(&mut rng) { code.push(c) };
        }

        chunk.push(code.iter().map(|c| c.to_string()).collect());
    }

    chunk
}

/// Return a number of bad codes that will be displayed WITH the good code
pub fn chunk_nb_from_difficulty(difficulty: Difficulty) -> usize {
    match difficulty {
        Difficulty::Easy => 3,
        Difficulty::Medium => 4,
        Difficulty::Hard => 5
    }
}