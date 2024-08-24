use crate::error::*;

use http::header::{self, HeaderValue};
use http::{HeaderName, Method};
use reqwest::{Body, Request, Response};
use smart_default::SmartDefault;
use url::Url;

#[derive(SmartDefault)]
pub struct Client {
    #[default(Url::parse("https://openspeech.bytedance.com").unwrap())]
    pub base_url: Url,
    pub access_token: String,
    #[default(reqwest::Client::new())]
    pub client: reqwest::Client,
}

impl Client {
    pub fn from_env() -> Result<Self> {
        let _ = dotenv::dotenv()?;
        Ok(Self {
            access_token: std::env::var("VOLCENGINE_ACCESS_TOKEN")?,
            ..Self::default()
        })
    }

    pub fn authorize(&self, req: &mut Request) -> Result<()> {
        req.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer; {}", self.access_token))?,
        );
        Ok(())
    }

    pub async fn call(
        &self,
        method: Method,
        uri: impl AsRef<str>,
        queries: Vec<(String, String)>,
        headers: Vec<(HeaderName, HeaderValue)>,
        body: Option<Body>,
    ) -> Result<Response> {
        let mut builder = self
            .client
            .request(method, self.base_url.join(uri.as_ref())?);

        for (k, v) in headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = body {
            builder = builder.body(body);
        }

        for (k, v) in queries {
            builder = builder.query(&[(k, v)]);
        }

        let mut req = builder.build()?;

        self.authorize(&mut req)?;

        let rep = self.client.execute(req).await?;//.error_for_status()?;

        Ok(rep)
    }
}
