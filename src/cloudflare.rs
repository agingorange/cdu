#![allow(dead_code)]

// Please note that ListDnsRecordsOrder, OrderDirection, and SearchMatch are enums that you
// might need to adjust based on your specific requirements.
use std::net::{Ipv4Addr, Ipv6Addr};

use chrono::offset::Utc;
use chrono::DateTime;
use reqwest::blocking::Client as RqClient;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json::json;

const BASE_URL: &str = "https://api.cloudflare.com/client/v4/zones";

#[derive(Debug)]
pub struct ListDnsRecords<'a> {
    pub zone_identifier: &'a str,
    pub params: ListDnsRecordsParams,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ListDnsRecordsParams {
    #[serde(flatten)]
    pub record_type: Option<DnsContent>,
    pub name: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<ListDnsRecordsOrder>,
    pub direction: Option<OrderDirection>,
    pub search_match: Option<SearchMatch>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ListDnsRecordsOrder {
    Type,
    Name,
    Content,
    Ttl,
    Proxied,
}

#[derive(Serialize, Clone, Debug)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Serialize, Clone, Debug)]
pub enum SearchMatch {
    All,
    Any,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DnsContent {
    A { content: Ipv4Addr },
    AAAA { content: Ipv6Addr },
    CNAME { content: String },
    NS { content: String },
    MX { content: String, priority: u16 },
    TXT { content: String },
    SRV { content: String },
}

#[derive(Deserialize, Debug)]
pub struct DnsRecord {
    pub meta: Meta,
    pub locked: bool,
    pub name: String,
    pub ttl: u32,
    pub zone_id: String,
    pub modified_on: DateTime<Utc>,
    pub created_on: DateTime<Utc>,
    pub proxiable: bool,
    #[serde(flatten)]
    pub content: DnsContent,
    pub id: String,
    pub proxied: bool,
    pub zone_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub auto_added: bool,
}

#[derive(Deserialize)]
struct Response {
    result: Vec<DnsRecord>,
}

pub struct Handler {
    client: RqClient,
    headers: HeaderMap,
    zone_id: String,
    dns_record: Option<DnsRecord>,
}

impl Handler {
    /// # Errors
    /// Returns an error if the `api_key` contains invalid characters not allowed in an
    /// HTTP header.
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
            dns_record: None,
        })
    }

    /// Retrieves the A record for the provided d domain name.
    /// # Errors
    /// Returns an error if the A record for the provided domain is not found.
    pub fn get_a_record(&mut self, domain: &str) -> anyhow::Result<Ipv4Addr> {
        let response: Response = self
            .client
            .get(format!("{}/{}/dns_records", BASE_URL, self.zone_id,))
            .headers(self.headers.clone())
            .send()?
            .json()?;

        for record in response.result {
            log::debug!("Record: {record:#?}");
            if let DnsContent::A { content } = record.content {
                if record.name == domain {
                    self.dns_record = Some(record);
                    log::debug!("Found A record for {domain}: {content}");
                    return Ok(content);
                }
            }
        }

        anyhow::bail!("A record for {domain} not found")
    }

    /// Updates A record with the provided IP address.
    ///
    /// # Errors
    /// Returns an error if there is no A record to update or if the update fails.
    pub fn update_a_record(&self, new_ip: Ipv4Addr) -> anyhow::Result<()> {
        log::debug!("Will update A record with: {new_ip}");
        if let Some(record) = &self.dns_record {
            let url = format!("{BASE_URL}/{0}/dns_records/{1}", self.zone_id, record.id);
            log::debug!("URL: {}", url);

            let body = json!({
                "type": "A",
                "name": record.name,
                "content": new_ip.to_string(),
                "ttl": 120,
                "proxied": false
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
                log::error!("Failed to update A record: {error_text}");
                anyhow::bail!("Failed to update A record: {error_text}");
            }
        } else {
            log::error!("No DNS record to update");
            anyhow::bail!("No DNS record to update");
        }
    }
}