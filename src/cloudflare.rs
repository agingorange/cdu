use std::net::Ipv4Addr;

use anyhow::anyhow;
use anyhow::Context;
use reqwest::blocking::Client as RqClient;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::AUTHORIZATION;
use serde_json::json;
use serde_json::Value;
use tracing::trace;

const BASE_URL: &str = "https://api.cloudflare.com/client/v4/zones";

#[derive(Debug)]
pub struct Handler {
    client: RqClient,
    headers: HeaderMap,
    zone_id: String,
    record_id: Option<String>,
}

impl Handler {
    pub fn try_new(api_key: &str, zone_id: &str) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))?,
        );

        Ok(Self {
            client: RqClient::new(),
            headers,
            zone_id: zone_id.to_string(),
            record_id: None,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn get_a_record(&mut self, domain: &str) -> anyhow::Result<Ipv4Addr> {
        let url = format!(
            "{BASE_URL}/{}/dns_records?type=A&name={domain}",
            self.zone_id
        );

        let response = self
            .client
            .get(url)
            .headers(self.headers.clone())
            .send()
            .context("Failed to send request to Cloudflare API")?
            .text()
            .context("Failed to read response text from Cloudflare API")?;
        trace!("Response: {response}");

        let v: Value = serde_json::from_str(&response)
            .context("Failed to parse JSON response from Cloudflare API")?;

        if let Some(errors) = v["errors"].as_array() {
            if !errors.is_empty() {
                let message = errors
                    .first()
                    .map(|e| e["message"].as_str().unwrap_or_default())
                    .unwrap_or_default()
                    .to_string();

                anyhow::bail!("Cloudflare API error: {message}");
            }
        }

        let records = v["result"]
            .as_array()
            .ok_or_else(|| anyhow!("No 'result' field found in JSON response"))?;

        for record in records {
            if let (Some(record_type), Some(record_name), Some(record_id), Some(content)) = (
                record["type"].as_str(),
                record["name"].as_str(),
                record["id"].as_str(),
                record["content"].as_str(),
            ) {
                if record_type == "A" && record_name == domain {
                    self.record_id = Some(record_id.into());
                    return content
                        .parse::<Ipv4Addr>()
                        .map_err(|e| anyhow!("Invalid IP address: {}", e));
                }
            }
        }

        Err(anyhow!("A record not found for domain: {}", domain))
    }

    #[tracing::instrument(skip_all)]
    pub fn set_a_record(&self, domain: &str, new_ip_v4_addr: Ipv4Addr) -> anyhow::Result<()> {
        let Some(ref record_id) = self.record_id else {
            anyhow::bail!("Missing record_id")
        };
        let url = format!("{}/{}/dns_records/{}", BASE_URL, self.zone_id, record_id);

        let body = json!({
            "type": "A",
            "name": domain,
            "content": new_ip_v4_addr.to_string(),
        });

        let response = self
            .client
            .put(url)
            .headers(self.headers.clone())
            .json(&body)
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text()?;
            anyhow::bail!("Failed to update A record: {error_text}");
        }
    }
}
