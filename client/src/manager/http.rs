use std::fmt::{Display, Formatter};
use std::sync::Arc;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use std::time::Duration;
use futures_util::StreamExt;
use log::{error, warn};
use reqwest::{header, multipart, RequestBuilder, StatusCode};
use reqwest::header::{CONTENT_LENGTH, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use error::{ Result, ApiError, Error };
use crate::models::user::{Application, ClientUser, User, UserId};
#[allow(unused_imports)] // Used in a macro
use crate::constants::API_URL;
use crate::models::channel::{Channel, ChannelId, Dm};
use crate::models::guild::{Guild, GuildId, GuildMember, Role};
use crate::models::interaction::{ApplicationCommand, InteractionCallbackType};
use crate::models::message::{AttachmentBuilder, Message, MessageBuilder};
use crate::models::Snowflake;


/// This type represent the API response, if this an Err(_), well, the api wasn't happy
pub type ApiResult<T> = core::result::Result<T, DiscordApiError>;

/// Convert a response from the Discord API into the wanted type `T` or an error.
///
/// The trick is that we return a Result, where, if this an error, this is normal.
/// But if this is a success, we have a second Result which can be `Ok(T)` or `Err(DiscordError)`.
fn convert_value<T: DeserializeOwned>(mut value: Value, shard: Option<u64>) -> Result<ApiResult<T>> {
    if let Some(s) = shard { value["shard"] = s.into(); }

    if is_api_error(&value) {
        match DiscordApiError::deserialize(value) {
            Ok(err) => Ok(Err(err)),
            Err(err) => Err(err.into())
        }
    } else {
        match serde_path_to_error::deserialize(value) {
            Ok(v) => Ok(Ok(v)),
            Err(err) => Err(error::Error::Api(ApiError::Deserialize(err.to_string())))
        }
    }
}

/// Verify if the json value is an error returned by the Discord API.
fn is_api_error(raw: &Value) -> bool {
    if raw.get("code").is_some() && raw.get("message").is_some() {
        return true;
    }

    false
}


/// Represents an error returned by the Discord API.
///
/// Reference:
/// - [Discord API Errors](https://discord.com/developers/docs/topics/opcodes-and-status-codes#json-json-error-codes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscordError {
    Unknown = -1,
}

/// Represents an error returned by the Discord API.
///
/// Reference:
/// - [Discord API Errors](https://discord.com/developers/docs/reference#error-messages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordApiError {
    pub code: u64,
    pub message: String,
    #[serde(default)]
    pub errors: Option<Value>,
}

impl Display for DiscordApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Discord API Error: {} ({:?})", self.message, self.code)
    }
}

impl std::error::Error for DiscordApiError {}

impl From<serde_json::error::Error> for DiscordApiError {
    fn from(err: serde_json::error::Error) -> Self {
        Self {
            code: 0,
            message: format!("Failed to parse json: {}", err),
            errors: Some(Value::Null)
        }
    }
}

#[derive(Clone)]
pub struct HttpConfiguration {
    pub retry_limit: u64,
    pub connect_timeout: Duration,
}

impl Default for HttpConfiguration {
    fn default() -> Self {
        Self {
            retry_limit: 5,
            connect_timeout: Duration::from_secs(5),
        }
    }
}

/// A request to be sent to the API.
#[derive(Debug, Clone)]
pub struct Request {
    /// The HTTP method to use.
    pub method: reqwest::Method,
    /// The URL to send the request to.
    pub url: String,
    /// The body of the request.
    pub body: Option<String>,
    pub headers: Option<HeaderMap>,
    /// To receive the response.
    pub sender: Arc<Mutex<UnboundedSender<Result<Value>>>>,
    pub multipart: Option<Vec<AttachmentBuilder>>
}

impl Request {
    #[allow(dead_code)]
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "method": format!("{}", self.method),
            "url": self.url,
            "body": self.body,
            "headers": self.headers.clone().map(|headers| Value::Array(headers.into_iter().map(|(k, v)| json!({ format!("{k:?}"): format!("{v:?}") })).collect::<Vec<Value>>()))
        })
    }
}

pub struct HttpManager {
    pub(crate) configuration: HttpConfiguration,
    pub(crate) rest: reqwest::Client,
    pub        client: Arc<Http>,
    pub        run: Arc<Mutex<bool>>,
               queue: Arc<RwLock<UnboundedReceiver<Request>>>,
               tasks: Arc<Mutex<Vec<JoinHandle<()>>>>
}

impl HttpManager {
    pub(crate) fn new(configuration: HttpConfiguration, rest_client: reqwest::Client) -> Self {
        let (tx, rx): (UnboundedSender<Request>, _) = futures_channel::mpsc::unbounded();

        let queue = Arc::new(RwLock::new(rx));

        Self {
            configuration,
            client: Arc::new(Http { queue: Arc::new(RwLock::new(tx)) }),
            rest: rest_client,
            run: Arc::new(Mutex::new(true)),
            tasks: Arc::new(Mutex::new(Vec::new())),
            queue,
        }
    }

    async fn send_request(configuration: HttpConfiguration, request: &Request, rest: &reqwest::Client) -> Result<Value> {
        let mut retries = 0;

        while retries < configuration.retry_limit {
            //let request = if let Some(clone) = request.try_clone() {
            //    clone
            //} else {
            //    return Err(Error::Api(ApiError::RequestError("Failed to clone request".into())))
            //};

            let res = Self::build_request(request.clone(), &configuration, rest).send().await;

            match res {
                Ok(res) => {
                    let status = res.status();

                    if status.is_success() && !status.is_client_error() {
                        if status == StatusCode::NO_CONTENT {
                            return Ok(Value::Null);
                        }

                        match res.json::<Value>().await {
                            Ok(json) => return Ok(json),
                            Err(err) => {
                                error!("Failed to parse response in a successful request: {}", err);
                                retries += 1;
                            }
                        }
                    } else if status.is_server_error() {
                        retries += 1;
                    } else if status == StatusCode::TOO_MANY_REQUESTS {
                        let message: Value = match res.json().await {
                            Ok(json) => json,
                            Err(err) => { return Err(Error::Api(ApiError::RequestStatus(err.to_string()))) }
                        };

                        let retry_after = match message.get("retry_after") {
                            Some(retry_after) => match retry_after.as_f64() {
                                Some(retry_after) => retry_after,
                                None => { return Err(Error::Api(ApiError::RequestStatus("Code 429, but the response is nuts (cannot convert `retry_after` field)".into()))) }
                            },
                            None => { return Err(Error::Api(ApiError::RequestStatus("Code 429, but the response is nuts (no field `retry_after`)".into()))) }
                        };

                        warn!("Rate limited, retrying in {}ms", retry_after);

                        sleep(Duration::from_secs_f64(retry_after)).await;
                        retries += 1;
                        continue;
                    } else {
                        match res.json::<Value>().await {
                            Ok(err) => {
                                return Ok(err)
                            },
                            Err(err) => {
                                error!("Failed to parse response from the api error: {}", err);
                                retries += 1;
                            }
                        }
                    }
                },
                Err(err) => {
                    error!("Failed to send request: {}", err);
                    retries += 1;
                }
            }
        }

        Err(Error::Api(ApiError::TooManyRetry))
    }

    pub(crate) fn start_loop(&self) {
        let queue = self.queue.clone();
        let configuration = self.configuration.clone();
        let rest = self.rest.clone();
        let run = self.run.clone();
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            loop {
                if !*run.lock().await {
                    break;
                }

                let mut queue = queue.write().await;

                let request = match queue.next().await {
                    Some(request) => request,
                    None => continue
                };

                // send request and send the response back to the requester
                let configuration = configuration.clone();
                let rest = rest.clone();
                let tasks = tasks.clone();
                // spawn a new task to send the request
                let task = tokio::spawn(async move {
                    let sender = request.sender.lock().await;

                    if sender.is_closed() { return; }

                    // let mut built_request = rest.request(request.method.clone(), request.url.clone())
                    //     .timeout(configuration.connect_timeout);
                    //     //.body(request.body.clone().unwrap_or_default());
                    //
                    // // set the headers if there are any
                    // if let Some(header_map) = request.headers {
                    //     for (key, value) in header_map.iter() {
                    //         built_request = built_request.header(key, value);
                    //     }
                    // }
                    //
                    // if let Some(multipart) = request.multipart {
                    //     let mut form = multipart::Form::new();
                    //
                    //     form = form.text("payload_json", request.body.unwrap_or_default());
                    //
                    //     for file in multipart.iter() {
                    //         form = form.part(file.filename.clone(), multipart::Part::bytes(file.bytes.to_vec()));
                    //     }
                    //
                    //     built_request = built_request.multipart(form);
                    // } else if request.body.is_some() {
                    //     // set the content type to json if there is a body
                    //     built_request = built_request.header("Content-Type", "application/json").body(request.body.unwrap_or_default());
                    // };


                    let res = Self::send_request(configuration.clone(), &request, &rest).await;

                    if let Err(e) = sender.unbounded_send(res) {
                        error!("Failed to send response back to requester: {}", e);
                    }
                });
                tasks.lock().await.push(task);

                // automatically remove all finished tasks
                {
                    let mut tasks = tasks.lock().await;
                    tasks.retain(|task| !task.is_finished());
                }
            }
        });
    }

    fn build_request(request: Request, configuration: &HttpConfiguration, rest: &reqwest::Client) -> RequestBuilder {
        let mut built_request = rest.request(request.method.clone(), request.url.clone())
            .timeout(configuration.connect_timeout);
        //.body(request.body.clone().unwrap_or_default());

        // set the headers if there are any
        if let Some(header_map) = &request.headers {
            for (key, value) in header_map.iter() {
                built_request = built_request.header(key, value);
            }
        }

        if let Some(multipart) = &request.multipart {
            let mut form = multipart::Form::new();

            form = form.text("payload_json", request.body.unwrap_or_default());

            for file in multipart.iter() {
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(file.content_type.as_str()).unwrap_or(HeaderValue::from_str("text/plain").unwrap())
                );
                form = form.part(
                    format!("files[{}]", file.id),
                    multipart::Part::bytes(file.bytes.to_vec())
                        .headers(headers)
                        .file_name(file.filename.clone())
                );
            }

            built_request = built_request.multipart(form);
        } else if request.body.is_some() {
            // set the content type to json if there is a body
            built_request = built_request.header("Content-Type", "application/json").body(request.body.unwrap_or_default());
        };

        built_request
    }

    pub async fn stop(&self){
        // set the run flag to false
        let mut run = self.run.lock().await;
        *run = false;

        // wait for all tasks to finish
        'stopping: loop {
            let mut tasks = self.tasks.lock().await;
            // we clear all finished tasks
            tasks.retain(|t| !t.is_finished());

            if !tasks.is_empty() {
                drop(tasks);
                sleep(Duration::from_millis(100)).await;
            } else {
                break 'stopping;
            }
        }
    }

    pub async fn get_queue_size(&self) -> usize {
        let queue = self.queue.read().await;

        std::mem::size_of_val(&queue)
    }

    pub async fn get_tasks_size(&self) -> usize {
        let tasks = self.tasks.lock().await;

        std::mem::size_of_val(&tasks)
    }
}

pub struct Http {
    /// A reference to the queue of requests to be sent.
    queue: Arc<RwLock<UnboundedSender<Request>>>
}

impl Http {
    /// Send a request to the API.
    pub async fn send_raw(&self, request: Request, mut rx: UnboundedReceiver<Result<Value>>) -> Result<Value> {
        {
            let queue = self.queue.read().await;

            if queue.is_closed() {
                return Err(Error::Api(ApiError::ChannelClosed));
            }

            if let Err(e) = queue.unbounded_send(request) {
                return Err(Error::Api(ApiError::ChannelSend(e.to_string())));
            }
        }

        match rx.next().await {
            Some(value) => value,
            _ => Err(Error::Api(ApiError::ChannelRecv))
        }
    }

    /// Fetch the application.
    ///
    /// Refer to: https://discord.com/developers/docs/resources/user#get-current-application-information
    pub async fn fetch_application(&self) -> Result<ApiResult<Application>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/oauth2/applications/@me"),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Fetch the client user.
    ///
    /// Refer to: https://discord.com/developers/docs/resources/user#get-current-user
    pub async fn fetch_client_user(&self) -> Result<ApiResult<ClientUser>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/users/@me"),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Fetch a guild by its ID.
    ///
    /// Reference:
    /// - [Get Guild](https://discord.com/developers/docs/resources/guild#get-guild)
    pub async fn fetch_guild(&self, id: &GuildId) -> Result<ApiResult<Guild>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/guilds/{}", id.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Fetch a user by its ID.
    ///
    /// Reference:
    /// - [Get User](https://discord.com/developers/docs/resources/user#get-user)
    pub async fn fetch_user(&self, id: impl ToString) -> Result<ApiResult<User>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/users/{}", id.to_string()),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Fetch a channel by its ID.
    ///
    /// Reference:
    /// - [Get Channel](https://discord.com/developers/docs/resources/channel#get-channel)
    pub async fn fetch_channel(&self, channel: &ChannelId) -> Result<ApiResult<Channel>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/channels/{}", channel.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Fetch all the channels in a guild.
    ///
    /// Reference:
    /// - [Get Guild Channels](https://discord.com/developers/docs/resources/guild#get-guild-channels)
    pub async fn fetch_guild_channels(&self, guild: &GuildId) -> Result<ApiResult<Vec<Channel>>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/guilds/{}/channels", guild.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => {
                let mut channels: Vec<Channel> = Vec::new();

                let mut raw_channels = value.as_array().unwrap().clone();

                for channel in raw_channels.iter_mut() {
                    match convert_value(channel.to_owned(), None) {
                        Ok(channel) => match channel {
                            Ok(channel) => channels.push(channel),
                            Err(e) => return Ok(Err(e))
                        },
                        Err(e) => error!("Failed to convert channel: {e:#?}")
                    }
                }

                Ok(Ok(channels))
            },
            Err(e) => Err(e)
        }
    }

    /// Fetch a guild member by its ID.
    ///
    /// Reference:
    /// - [Get Guild Member](https://discord.com/developers/docs/resources/guild#get-guild-member)
    pub async fn fetch_guild_member(&self, guild: &GuildId, member: &UserId) -> Result<ApiResult<GuildMember>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/guilds/{}/members/{}", guild.0, member.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(mut value) => {
                value["guild_id"] = json!(guild.0);
                convert_value(value, None)
            },
            Err(e) => Err(e)
        }
    }

    /// Fetch all the roles in a guild.
    ///
    /// Reference:
    /// - [Get Guild Roles](https://discord.com/developers/docs/resources/guild#get-guild-roles)
    pub async fn fetch_guild_roles(&self, guild: &GuildId) -> Result<ApiResult<Vec<Role>>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/guilds/{}/roles", guild.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => {
                let mut roles = Vec::new();

                for role in value.as_array().unwrap() {
                    match convert_value(role.to_owned(), None) {
                        Ok(role) => match role {
                            Ok(role) => roles.push(role),
                            Err(e) => return Ok(Err(e))
                        },
                        Err(e) => error!("Failed to convert role: {e:#?}")
                    }
                }

                Ok(Ok(roles))
            },
            Err(e) => Err(e)
        }
    }

    /// Add a role to a member.
    ///
    /// Reference:
    /// - [Add Guild Member Role](https://discord.com/developers/docs/resources/guild#add-guild-member-role)
    pub async fn add_role_to_member(&self, guild: &GuildId, member: &UserId, role: &Snowflake) -> Result<ApiResult<()>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let mut header = HeaderMap::new();
        // we need to set the content-length to 0, otherwise the request will fail
        header.insert(
            CONTENT_LENGTH,
            HeaderValue::from_static("0")
        );

        let request = Request {
            method: reqwest::Method::PUT,
            url: format!("{API_URL}/guilds/{}/members/{}/roles/{}", guild.0, member.0, role.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: Some(header),
            multipart: None
        };


        self.send_raw(request, rx).await?;

        Ok(Ok(()))
    }

    /// Remove a role to a member.
    ///
    /// Reference:
    /// - [Remove Guild Member Role](https://discord.com/developers/docs/resources/guild#remove-guild-member-role)
    pub async fn remove_role_to_member(&self, guild: &GuildId, member: &UserId, role: &Snowflake) -> Result<ApiResult<()>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::DELETE,
            url: format!("{API_URL}/guilds/{}/members/{}/roles/{}", guild, member, role),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        self.send_raw(request, rx).await?;

        Ok(Ok(()))
    }

    /// Fetch a message by its ID in a channel.
    ///
    /// Reference:
    /// - [Get Channel Message](https://discord.com/developers/docs/resources/channel#get-channel-message)
    pub async fn fetch_message(&self, channel: &ChannelId, message: &Snowflake) -> Result<ApiResult<Message>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/channels/{}/messages/{}", channel, message),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(mut value) => {
                value["channel_id"] = json!(channel.0);
                convert_value(value, None)
            },
            Err(e) => Err(e)
        }
    }

    /// Create a DM channel with a user.
    ///
    /// Reference:
    /// - [Create DM](https://discord.com/developers/docs/resources/user#create-dm)
    pub async fn create_dm_channel(&self, recipient: &UserId) -> Result<ApiResult<Dm>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::POST,
            url: format!("{API_URL}/users/@me/channels"),
            body: Some(json!({
                "recipient_id": recipient.0
            }).to_string()),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Send a message to a channel.
    ///
    /// Reference:
    /// - [Create Message](https://discord.com/developers/docs/resources/channel#create-message)
    pub async fn send_message(
        &self,
        channel: &ChannelId,
        payload: MessageBuilder,
        files: Option<Vec<AttachmentBuilder>>
    ) -> Result<ApiResult<Message>>
    {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::POST,
            url: format!("{API_URL}/channels/{}/messages", channel),
            body: Some(payload.to_json().to_string()),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: files
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Send a message to a User channel.
    ///
    /// Reference:
    /// - [Create Message](https://discord.com/developers/docs/resources/channel#create-message)
    pub async fn send_user(&self, user: &UserId, payload: MessageBuilder) -> Result<ApiResult<Message>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::POST,
            url: format!("{API_URL}/channels/{}/messages", user),
            body: Some(payload.to_json().to_string()),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        dbg!(&res);

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Send a response to an interaction.
    ///
    /// Reference:
    /// - [Interaction Response](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-response-object)
    pub async fn reply_interaction(
        &self,
        id: &Snowflake,
        token: &String,
        callback_type: InteractionCallbackType,
        payload: Value,
        files: Option<Vec<AttachmentBuilder>>
    ) -> Result<ApiResult<()>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::POST,
            url: format!("{API_URL}/interactions/{id}/{token}/callback"),
            body: Some(
                json!({
                    "type": callback_type.to_json(),
                    "data": payload
                }).to_string()
            ),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: files
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => {
                if value.is_null() {
                    Ok(Ok(()))
                } else {
                    match DiscordApiError::deserialize(value) {
                        Ok(err) => Ok(Err(err)),
                        Err(err) => Err(err.into())
                    }
                }
            },
            Err(e) => Err(e)
        }
    }

    /// Edit an interaction response.
    ///
    /// Reference:
    /// - [Edit Original Interaction Response](https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response)
    pub async fn edit_interaction(
        &self,
        id: &Snowflake,
        token: &String,
        payload: Value,
        files: Option<Vec<AttachmentBuilder>>
    ) -> Result<ApiResult<Message>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::PATCH,
            url: format!("{API_URL}/webhooks/{id}/{token}/messages/@original"),
            body: Some(payload.to_string()),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: files
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Edit an interaction response.
    ///
    /// Reference:
    /// - [Edit Original Interaction Response](https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response)
    pub async fn get_interaction_response(
        &self,
        id: &Snowflake,
        token: &String,
    ) -> Result<ApiResult<Message>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/webhooks/{id}/{token}/messages/@original"),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Create or update a global application command.
    ///
    /// Reference:
    /// - [Create Global Application Command](https://discord.com/developers/docs/interactions/application-commands#create-global-application-command)
    pub async fn create_global_application_command(&self, application_id: &Snowflake, payload: &ApplicationCommand) -> Result<ApiResult<ApplicationCommand>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::POST,
            url: format!("{API_URL}/applications/{}/commands", application_id.0),
            body: Some(payload.to_json().to_string()),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Create or update a global application command.
    ///
    /// Reference:
    /// - [Create Guild Application Command](https://discord.com/developers/docs/interactions/application-commands#create-guild-application-command)
    pub async fn create_guild_application_command(&self, application_id: &Snowflake, guild_id: &GuildId, payload: &ApplicationCommand) -> Result<ApiResult<ApplicationCommand>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::POST,
            url: format!("{API_URL}/applications/{}/guilds/{}/commands", application_id.0, guild_id.0),
            body: Some(payload.to_json().to_string()),
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => convert_value(value, None),
            Err(e) => Err(e)
        }
    }

    /// Get all commands for your application in a guild.
    ///
    /// Reference:
    /// - [Get Global Application Commands](https://discord.com/developers/docs/interactions/application-commands#get-global-application-commands)
    pub async fn get_guild_commands(&self, application_id: &Snowflake, guild_id: &GuildId) -> Result<ApiResult<Vec<ApplicationCommand>>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/applications/{}/guilds/{}/commands", application_id.0, guild_id.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => {
                let mut commands = Vec::new();

                for command in value.as_array().unwrap() {
                    match convert_value(command.clone(), None) {
                        Ok(command) => match command {
                            Ok(command) => commands.push(command),
                            Err(e) => return Ok(Err(e))
                        },
                        Err(e) => error!("Failed to convert command: {e:#?}")
                    }
                }

                Ok(Ok(commands))
            },
            Err(e) => Err(e)
        }
    }

    /// Delete a guild command.
    ///
    /// Reference:
    /// - [Delete Global Application Command](https://discord.com/developers/docs/interactions/application-commands#delete-global-application-command)
    pub async fn delete_guild_command(&self, application_id: &Snowflake, guild_id: &GuildId, command_id: &Snowflake) -> Result<ApiResult<()>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::DELETE,
            url: format!("{API_URL}/applications/{}/guilds/{}/commands/{}", application_id.0, guild_id.0, command_id.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(_) => Ok(Ok(())),
            Err(e) => Err(e)
        }
    }

    /// Get all the global commands for an application.
    ///
    /// Reference:
    /// - [Get Global Application Commands](https://discord.com/developers/docs/interactions/application-commands#get-global-application-commands)
    pub async fn get_global_commands(&self, application_id: &Snowflake) -> Result<ApiResult<Vec<ApplicationCommand>>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::GET,
            url: format!("{API_URL}/applications/{}/commands", application_id.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(
            request.clone(),
            rx
        ).await;

        match res {
            Ok(value) => {
                let mut commands = Vec::new();

                for command in value.as_array().unwrap() {
                    match convert_value(command.to_owned(), None) {
                        Ok(command) => match command {
                            Ok(command) => commands.push(command),
                            Err(e) => return Ok(Err(e))
                        },
                        Err(e) => error!("Failed to convert command: {e:#?}")
                    }
                }

                Ok(Ok(commands))
            },
            Err(e) => Err(e)
        }
    }

    /// Delete a global command.
    ///
    /// Reference:
    /// - [Delete Global Application Command](https://discord.com/developers/docs/interactions/application-commands#delete-global-application-command)
    pub async fn delete_global_command(&self, application_id: &Snowflake, command_id: &Snowflake) -> Result<ApiResult<()>> {
        let (tx, rx) = futures_channel::mpsc::unbounded();

        let request = Request {
            method: reqwest::Method::DELETE,
            url: format!("{API_URL}/applications/{}/commands/{}", application_id.0, command_id.0),
            body: None,
            sender: Arc::new(Mutex::new(tx)),
            headers: None,
            multipart: None
        };

        let res = self.send_raw(request, rx).await;

        match res {
            Ok(r) => convert_value(r, None),
            Err(e) => Err(e)
        }
    }
}