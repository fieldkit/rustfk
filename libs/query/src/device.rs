use anyhow::Result;
use prost::Message;
use reqwest::header::HeaderMap;
use reqwest::RequestBuilder;
use std::io::Cursor;
use std::time::Duration;
use tracing::*;

pub mod http {
    include!(concat!(env!("OUT_DIR"), "/fk_app.rs"));
}

pub use http::*;

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        let sdk_version = std::env!("CARGO_PKG_VERSION");
        let user_agent = format!("rustfk ({})", sdk_version);
        headers.insert("user-agent", user_agent.parse()?);

        let client = reqwest::ClientBuilder::new()
            .user_agent("rustfk")
            .timeout(Duration::from_secs(10))
            .default_headers(headers)
            .build()?;

        Ok(Self { client })
    }

    pub async fn query_status(&self, addr: &str) -> Result<HttpReply> {
        self.execute(self.new_request(addr)?.build()?).await
    }

    pub async fn query_readings(&self, addr: &str) -> Result<HttpReply> {
        let mut query = HttpQuery::default();
        query.r#type = QueryType::QueryGetReadings as i32;
        let encoded = query.encode_length_delimited_to_vec();
        let req = self.new_request(addr)?.body(encoded).build()?;
        self.execute(req).await
    }

    async fn execute(&self, req: reqwest::Request) -> Result<HttpReply> {
        let url = req.url().clone();

        debug!("{} querying", &url);
        let response = self.client.execute(req).await?;
        let bytes = response.bytes().await?;

        debug!("{} queried, got {} bytes", &url, bytes.len());
        Ok(HttpReply::decode_length_delimited(bytes)?)
    }

    fn new_request(&self, addr: &str) -> Result<RequestBuilder> {
        let url = format!("http://{}/fk/v1", addr);
        Ok(self.client.post(&url).timeout(Duration::from_secs(5)))
    }
}

pub fn parse_http_reply(data: &[u8]) -> Result<HttpReply> {
    let mut cursor = Cursor::new(data);
    Ok(HttpReply::decode_length_delimited(&mut cursor)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_status() -> Result<()> {
        let data = include_bytes!("../examples/status_1.fkpb");
        let mut cursor = Cursor::new(data);
        let data = HttpReply::decode_length_delimited(&mut cursor)?;
        let status = data.status.unwrap();
        assert_eq!(status.identity.unwrap().device, "Early Impala 91");
        Ok(())
    }

    #[test]
    pub fn test_parse_status_with_logs() -> Result<()> {
        let data = include_bytes!("../examples/status_2_logs.fkpb");
        let mut cursor = Cursor::new(data);
        let data = HttpReply::decode_length_delimited(&mut cursor)?;
        let status = data.status.unwrap();
        assert_eq!(status.identity.unwrap().device, "Early Impala 91");
        assert_eq!(status.logs.len(), 32266);
        Ok(())
    }

    #[test]
    pub fn test_parse_status_with_readings() -> Result<()> {
        let data = include_bytes!("../examples/status_3_readings.fkpb");
        let mut cursor = Cursor::new(data);
        let data = HttpReply::decode_length_delimited(&mut cursor)?;
        let status = data.status.unwrap();
        assert_eq!(status.identity.unwrap().device, "Early Impala 91");
        let modules = &data.live_readings[0].modules;
        assert_eq!(modules.len(), 3);
        Ok(())
    }
}
