use std::collections::HashMap;

use crate::{client::Client, error::*, types::*};
use http::{header, HeaderValue, Method};
use reqwest::Body;
use serde_json::{json, Value};
use smart_default::SmartDefault;
use std::path::PathBuf;
use tracing::*;
use serde_with::skip_serializing_none;

#[derive(Debug, Clone, SmartDefault)]
pub struct SubtitleRequestBuilder {
    pub params: HashMap<String, String>,
    pub source: Option<SubtitleSource>,
}

macro_rules! impl_with_params {
    ($fun: ident) => {
        impl SubtitleRequestBuilder {
            pub fn $fun(mut self, value: impl Into<String>) -> Self {
                self.params.insert(stringify!($fun).into(), value.into());
                self
            }
        }
    };
    ($fun: ident, $($more_fun: ident), +) => {
        impl_with_params!($fun);
        impl_with_params!($($more_fun),+);
    }
}

impl_with_params!(
    appid,
    words_per_line,
    max_lines,
    use_itn,
    language,
    caption_type,
    use_punc,
    use_ddc,
    boosting_table_id,
    boosting_table_name,
    asr_appid,
    with_speaker_info
);

impl SubtitleRequestBuilder {
    pub fn source(mut self, source: impl Into<SubtitleSource>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn build(self) -> Result<SubtitleRequest> {
        let Self { params, source } = self;
        let source = source.ok_or(Error::SubtitleRequestBuild)?;
        if !params.contains_key("appid") {
            return Err(Error::SubtitleRequestBuild);
        }
        Ok(SubtitleRequest { params, source })
    }
}

impl SubtitleRequest {
    pub fn builder() -> SubtitleRequestBuilder {
        SubtitleRequestBuilder::default()
    }

    pub async fn call(&self, client: &Client) -> Result<SubtitleResponse> {
        let Self { params, source } = self;
        let queries = params
            .iter()
            .map(|a| (a.0.to_owned(), a.1.to_owned()))
            .collect::<Vec<_>>();
        let rep = match source {
            SubtitleSource::Binary { typ, data } => {
                let rep = client
                    .call(
                        Method::POST,
                        "/api/v1/vc/submit",
                        queries,
                        vec![(
                            header::CONTENT_TYPE,
                            HeaderValue::from_str(&format!("audio/{}", typ))?,
                        )],
                        Some(Body::from(data.to_owned())),
                    )
                    .await?; //.error_for_status()?;
                let val: Value = serde_json::from_slice(rep.bytes().await?.as_ref())?;
                for l in serde_json::to_string_pretty(&val)?.lines() {
                    trace!("REP: {}", l);
                }
                serde_json::from_value(val)?
            }
            SubtitleSource::Url(url) => {
                let body = json!({"url": url});
                let rep = client
                    .call(
                        Method::POST,
                        "/api/v1/vc/submit",
                        queries,
                        vec![(
                            header::CONTENT_TYPE,
                            HeaderValue::from_str("application/json")?,
                        )],
                        Some(Body::from(serde_json::to_string(&body)?)),
                    )
                    .await?
                    .error_for_status()?;
                serde_json::from_slice(rep.bytes().await?.as_ref())?
            }
        };
        Ok(rep)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SubtitleResponse {
    pub code: i64,
    pub message: String,
    pub id: String,
}

impl SubtitleResponse {
    pub async fn waiting_result(
        &self,
        appid: impl AsRef<str>,
        client: &Client,
    ) -> Result<SubtitleResult> {
        let uri = "/api/v1/vc/query";
        trace!("waiting appid={}, id={}", appid.as_ref(), self.id);
        let rep = client
            .call(
                Method::GET,
                uri,
                vec![
                    ("appid".into(), appid.as_ref().to_string()),
                    ("id".into(), self.id.to_owned()),
                    ("blocking".into(), "1".into()),
                ],
                vec![],
                None,
            )
            .await?;
        let rep: Value = serde_json::from_slice(rep.bytes().await?.as_ref())?;
        for l in serde_json::to_string_pretty(&rep)?.lines() {
            trace!("REP: {}", l);
        }
        let rep: SubtitleResult = serde_json::from_value(rep)?;
        Ok(rep)
    }
}

pub struct SubtitleRequest {
    pub params: HashMap<String, String>,
    pub source: SubtitleSource,
}

#[derive(Debug, Clone)]
pub enum SubtitleSource {
    Binary { typ: String, data: Vec<u8> },
    Url(String),
}

impl SubtitleSource
{
    pub fn from_local_file(value: impl Into<PathBuf>) -> Result<Self> {
        let value = value.into();
        let typ = value
            .extension()
            .ok_or(Error::NoExtension)?
            .to_str()
            .ok_or(Error::NoExtension)?
            .to_string();
        let data = std::fs::read(&value)?;
        Ok(Self::Binary { typ, data })
    }
}

impl From<PathBuf> for SubtitleSource {
    fn from(value: PathBuf) -> Self {
        Self::from_local_file(value).expect("failed to read file")
    }
}


#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Extra {
    pub asr_service: String,
    pub caption_type: String,
    pub is_mandarin: Boolean,
    pub is_speech: Boolean,
    pub language: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Attribute {
    pub extra: Option<Extra>,
    pub event: Option<String>,
    pub speaker: Option<String>
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SubtitleResult {
    pub code: i64,
    pub duration: f32,
    pub id: String,
    pub message: String,
    pub attribute: Attribute,
    pub utterances: Vec<Utterance>,
}



#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Word
{
    pub attribute: Attribute,
    pub start_time: i64,
    pub end_time: i64,
    pub text: String,
}


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UtteranceAttribute {
    pub event: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Utterance {
    pub start_time: i64,
    pub end_time: i64,
    pub text: String,
    pub words: Vec<Word>,
    pub attribute: Attribute,
}

#[cfg(test)]
#[tokio::test]
async fn test_subtitle_ok() -> Result<()> {
    let _ = dotenv::dotenv();
    let _ = tracing_subscriber::fmt::try_init();

    let appid = std::env::var("VOLCENGINE_APP_ID")?;

    let test_mp3 = PathBuf::from(std::env::var("TEST_MP3_FILE")?);

    let client = Client::from_env()?;

    let rep = SubtitleRequest::builder()
        .appid(appid.clone())
        .with_speaker_info(Boolean::True)
        .source(test_mp3)
        .build()?
        .call(&client)
        .await?
        .waiting_result(appid, &client)
        .await?;

    for l in serde_json::to_string_pretty(&rep)?.lines() {
        info!("REP: {}", l);
    }

    Ok(())
}
