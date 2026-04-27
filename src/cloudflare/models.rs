#![allow(dead_code)]

use std::net::Ipv4Addr;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum CloudFlareClientError {
    Request(reqwest::Error),
    Response(reqwest::Error),
    Api(ErrorResponse),
    Other(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ErrorResponse {
    pub errors: Vec<Message>,
    pub messages: Vec<Message>,
    pub success: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct SuccessResponseList<T> {
    pub errors: Vec<Message>,
    pub messages: Vec<Message>,
    pub success: bool,
    pub result_info: ResultInfo,
    pub result: Vec<T>,
}

impl<T> SuccessResponseList<T> {
    pub fn count(&self) -> usize {
        self.result.len()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct UpdateDNSRecordRequest {
    pub content: String,
    pub name: String,
    pub proxied: Option<bool>,
    pub r#type: DNSType,
    pub comment: Option<String>,
    pub id: String,
    pub tags: Option<Vec<String>>,
    pub ttl: Option<i32>,
}

impl From<DNSRecord> for UpdateDNSRecordRequest {
    fn from(value: DNSRecord) -> Self {
        Self {
            content: value.content,
            name: value.name,
            proxied: value.proxied,
            r#type: value.r#type,
            comment: value.comment,
            id: value.id,
            tags: value.tags,
            ttl: value.ttl,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub code: i32,
    #[serde(default)]
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct ResultInfo {
    pub count: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_count: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
pub struct DNSRecord {
    pub content: String,
    pub name: String,
    #[serde(default)]
    pub proxied: Option<bool>,
    pub r#type: DNSType,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub comment_modified_on: Option<DateTime<Utc>>,
    pub created_on: DateTime<Utc>,
    pub id: String,
    pub meta: Value,
    pub modified_on: DateTime<Utc>,
    pub proxiable: bool,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub tags_modified_on: Option<DateTime<Utc>>,
    pub ttl: Option<i32>,
}

impl DNSRecord {
    pub fn has_tags(&self) -> bool {
        match &self.tags {
            Some(tags) => !tags.is_empty(),
            None => false,
        }
    }

    pub fn content_as_ip(&self) -> Result<Ipv4Addr, std::net::AddrParseError> {
        Ipv4Addr::from_str(&self.content)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
#[repr(i32)]
#[allow(clippy::upper_case_acronyms)]
pub enum DNSType {
    #[default]
    A = 1,
    AAAA = 28,
    CAA = 257,
    CERT = 37,
    CNAME = 5,
    DNSKEY = 48,
    DS = 43,
    HTTPS = 65,
    LOC = 29,
    MX = 15,
    NAPTR = 35,
    NS = 2,
    PTR = 12,
    SMIMEA = 53,
    SRV = 33,
    SSHFP = 44,
    SVCB = 64,
    TLSA = 52,
    TXT = 16,
    URI = 256,
}

impl std::fmt::Display for DNSType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl DNSType {
    /// https://en.wikipedia.org/wiki/List_of_DNS_record_types
    pub fn id(&self) -> i32 {
        self.clone() as i32
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Hash)]
pub struct ZoneId(String);

impl ZoneId {
    pub fn new(value: &str) -> Result<Self, String> {
        if value.len() > 32 {
            return Err(String::from(
                "Invalid ZoneId, must be less than 32 characters.",
            ));
        }
        Ok(Self(value.to_string()))
    }
}

impl TryFrom<String> for ZoneId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ZoneId::new(&value)
    }
}

impl std::fmt::Display for ZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ZoneId> for String {
    fn from(value: ZoneId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_list_count() {
        let mut o = SuccessResponseList::<i32>::default();
        o.result = vec![1, 2, 3, 4, 5];
        assert_eq!(o.count(), 5);
    }

    #[test]
    fn dns_record_has_tags() {
        let mut o = DNSRecord::default();
        o.tags = Some(vec![String::from("a")]);
        assert_eq!(o.has_tags(), true);
    }

    #[test]
    fn dns_record_have_empty_tags() {
        let mut o = DNSRecord::default();
        o.tags = Some(vec![]);
        assert_eq!(o.has_tags(), false);
    }

    #[test]
    fn dns_record_have_none_tags() {
        let mut o = DNSRecord::default();
        o.tags = None;
        assert_eq!(o.has_tags(), false);
    }

    #[test]
    fn dns_record_content_as_ip_pass() {
        let mut o = DNSRecord::default();
        o.content = String::from("127.0.0.1");
        assert_eq!(
            o.content_as_ip().unwrap(),
            std::net::Ipv4Addr::new(127, 0, 0, 1)
        );
    }
}
