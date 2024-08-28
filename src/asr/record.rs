use std::time::Duration;

use crate::{client::Client, error::*, types::*};
use http::{header, HeaderValue, Method};
use reqwest::Body;
use serde_with::skip_serializing_none;
use tracing::{error, trace};

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct RecordAsrResponse {
    pub code: i32,
    pub message: String,
    pub id: String,
    #[serde(default)]
    pub appid: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub cluster: String,
}

impl RecordAsrResponse {
    pub async fn waiting_result(self, client: &Client, retry: Duration) -> Result<RecordAsrResult> {
        let Self {
            id,
            appid,
            token,
            cluster,
            ..
        } = self;

        let body = serde_json::json!( {
            "appid": appid,
            "token": token,
            "cluster": cluster,
            "id": id,
        });

        for l in serde_json::to_string_pretty(&body)?.lines() {
            trace!("REQ: {}", l);
        }

        let rep = loop {
            let rep = client
                .call(
                    Method::POST,
                    "/api/v1/auc/query",
                    vec![],
                    vec![(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str("application/json")?,
                    )],
                    Some(Body::from(serde_json::to_string(&body)?)),
                )
                .await?;

            let rep = rep.bytes().await?;

            let mut rep: serde_json::Value = serde_json::from_slice(rep.as_ref())?;

            let rep = rep.get_mut("resp").ok_or(Error::RecordAsrResponse)?.take();

            for l in serde_json::to_string_pretty(&rep)?.lines() {
                trace!("REP: {}", l);
            }

            match rep.get("code").and_then(|v| v.as_i64()) {
                Some(1000) => break rep,
                _ => {
                    tokio::time::sleep(retry).await;
                    continue;
                }
            }
        };

        for l in serde_json::to_string_pretty(&rep)?.lines() {
            trace!("REP: {}", l);
        }

        Ok(serde_json::from_value(rep)?)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct RecordAsrRequest {
    pub app: App,
    pub user: User,
    pub audio: Audio,
    pub request: Option<Request>,
    pub additions: Option<Addtions>,
}

impl RecordAsrRequest {
    pub fn builder() -> RecordAsrRequestBuilder {
        RecordAsrRequestBuilder::default()
    }

    pub async fn call(self, client: &Client) -> Result<RecordAsrResponse> {
        let body = serde_json::to_string(&self)?;

        let rep = client
            .call(
                Method::POST,
                "/api/v1/auc/submit",
                vec![],
                // vec![],
                vec![(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/json")?,
                )],
                Some(body.into()),
            )
            .await?;

        let status = rep.status();

        let body = rep.bytes().await?;

        let mut body = serde_json::from_slice::<serde_json::Value>(body.as_ref())?;

        match status.is_success() {
            true => {
                for l in serde_json::to_string_pretty(&body)?.lines() {
                    trace!("REP: {}", l);
                }
            }
            false => {
                for l in serde_json::to_string_pretty(&body)?.lines() {
                    error!("REP: {}", l);
                }
            }
        }

        let rep = body.get_mut("resp").ok_or(Error::RecordAsrResponse)?.take();

        let mut rep = serde_json::from_value::<RecordAsrResponse>(rep)?;

        rep.appid = self.app.appid;
        rep.token = self.app.token;
        rep.cluster = self.app.cluster;

        Ok(rep)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct App {
    pub appid: String,
    pub token: String,
    pub cluster: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct User {
    pub uid: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Audio {
    pub url: String,
    pub format: Option<String>,
    pub codec: Option<String>,
    pub rate: Option<i32>,
    pub bits: Option<i32>,
    pub channel: Option<i32>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Request {
    pub callback: Option<String>,
    pub boosting_table_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Addtions {
    pub language: Option<String>,
    pub use_itn: Option<Boolean>,
    pub use_punc: Option<Boolean>,
    pub use_ddc: Option<Boolean>,
    pub with_speaker_info: Option<Boolean>,
    pub enable_query: Option<Boolean>,
    pub channel_split: Option<Boolean>,
}

#[skip_serializing_none]
#[derive(
    Debug, Clone, serde::Deserialize, serde::Serialize, smart_default::SmartDefault, PartialEq,
)]
pub struct RecordAsrRequestBuilder {
    pub appid: Option<String>,
    pub token: Option<String>,
    pub cluster: Option<String>,
    pub uid: Option<String>,
    pub url: Option<String>,
    pub format: Option<String>,
    pub codec: Option<String>,
    pub rate: Option<i32>,
    pub bits: Option<i32>,
    pub channel: Option<i32>,
    pub callback: Option<String>,
    pub boosting_table_name: Option<String>,
    pub language: Option<String>,
    pub use_itn: Option<Boolean>,
    pub use_punc: Option<Boolean>,
    pub use_ddc: Option<Boolean>,
    pub with_speaker_info: Option<Boolean>,
    pub enable_query: Option<Boolean>,
    pub channel_split: Option<Boolean>,
}

impl RecordAsrRequestBuilder {
    pub fn build(self) -> Result<RecordAsrRequest> {
        let Self {
            appid,
            token,
            cluster,
            uid,
            url,
            format,
            codec,
            rate,
            bits,
            channel,
            callback,
            boosting_table_name,
            language,
            use_itn,
            use_punc,
            use_ddc,
            with_speaker_info,
            enable_query,
            channel_split,
        } = self;

        let app = App {
            appid: appid.ok_or(Error::RecordRequestBuild)?,
            token: token.ok_or(Error::RecordRequestBuild)?,
            cluster: cluster.ok_or(Error::RecordRequestBuild)?,
        };

        let user = User {
            uid: uid.ok_or(Error::RecordRequestBuild)?,
        };

        let audio = Audio {
            url: url.ok_or(Error::RecordRequestBuild)?,
            format,
            codec,
            rate,
            bits,
            channel,
        };

        let request = match callback.is_some() || boosting_table_name.is_some() {
            true => Some(Request {
                callback,
                boosting_table_name,
            }),
            false => None,
        };

        let additions = match language.is_some()
            || use_itn.is_some()
            || use_punc.is_some()
            || use_ddc.is_some()
            || with_speaker_info.is_some()
            || enable_query.is_some()
            || channel_split.is_some()
        {
            true => Some(Addtions {
                language,
                use_itn,
                use_punc,
                use_ddc,
                with_speaker_info,
                enable_query,
                channel_split,
            }),
            false => None,
        };

        Ok(RecordAsrRequest {
            app,
            user,
            audio,
            request,
            additions,
        })
    }
}

macro_rules! impl_with {
    ($param: ident, $typ:ty) => {
        impl RecordAsrRequestBuilder {
            pub fn $param(mut self, $param: impl Into<$typ>) -> Self {
                self.$param = Some($param.into());
                self
            }
        }
    };
}

impl_with!(appid, String);
impl_with!(token, String);
impl_with!(cluster, String);
impl_with!(uid, String);
impl_with!(url, String);
impl_with!(format, String);
impl_with!(codec, String);
impl_with!(rate, i32);
impl_with!(bits, i32);
impl_with!(channel, i32);
impl_with!(callback, String);
impl_with!(boosting_table_name, String);
impl_with!(language, String);
impl_with!(use_itn, Boolean);
impl_with!(use_punc, Boolean);
impl_with!(use_ddc, Boolean);
impl_with!(with_speaker_info, Boolean);
impl_with!(enable_query, Boolean);
impl_with!(channel_split, Boolean);

#[skip_serializing_none]
#[derive(
    Debug, Clone, serde::Deserialize, serde::Serialize, smart_default::SmartDefault, PartialEq,
)]
pub struct Word {
    pub start_time: i64,
    pub end_time: i64,
    pub text: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct UAddtions {
    pub event: Option<String>,
    pub speaker: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Utterance {
    pub start_time: i64,
    pub end_time: i64,
    pub text: String,
    pub words: Vec<Word>,
    pub additions: Option<UAddtions>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct RecordAsrResult {
    pub id: String,
    pub code: i32,
    pub additions: Addtions,
    pub message: String,
    pub text: Option<String>,
    pub utterances: Vec<Utterance>,
}

#[cfg(test)]
#[tokio::test]
async fn test_record_asr_ok() -> Result<()> {
    let _ = dotenv::dotenv();
    tracing_subscriber::fmt::init();

    let appid = std::env::var("VOLCENGINE_APP_ID")?;
    let token = std::env::var("VOLCENGINE_ACCESS_TOKEN")?;
    let cluster = std::env::var("VOLCENGINE_CLUSTER")?;
    let mp3 = std::env::var("OSS_MP3")?;

    let client = Client::from_env()?;

    let req = RecordAsrRequest::builder()
        .appid(&appid)
        .token(&token)
        .cluster(&cluster)
        .use_punc(true)
        .uid("388808087185088_demo")
        .with_speaker_info(Boolean::True)
        .url(mp3)
        .format("mp3")
        .build()?;

    let rep = req.call(&client).await?.waiting_result(&client, Duration::from_secs(10)).await?;

    for l in serde_json::to_string_pretty(&rep)?.lines() {
        tracing::info!("{}", l);
    }

    Ok(())
}
