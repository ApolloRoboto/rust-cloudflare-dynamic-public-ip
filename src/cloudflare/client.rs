#![allow(dead_code)]

use super::models::*;

use log::{debug, warn};
use std::time::Duration;

use reqwest::{Request, StatusCode};

pub struct CloudFlareClient {
    client: reqwest::Client,
    token: String,
    pub zone_id: ZoneId,
    base_url: String,
}

impl CloudFlareClient {
    pub fn new(token: &str, zone_id: ZoneId) -> Self {
        CloudFlareClient::new_with_url(token, zone_id, "https://api.cloudflare.com")
    }

    pub fn new_with_url(token: &str, zone_id: ZoneId, url: &str) -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        Self {
            client,
            token: String::from(token),
            zone_id,
            base_url: String::from(url),
        }
    }

    async fn send_request(&self, request: Request) -> Result<reqwest::Response, reqwest::Error> {
        let mut attempts = 0;

        loop {
            let request = request.try_clone().expect("Failed to clone request");

            debug!("{} {}", request.method(), request.url());
            match self.client.execute(request).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    let delay = Duration::from_millis(match attempts {
                        0 => 100,
                        1 => 200,
                        2 => 400,
                        3 => 800,
                        4 => 1200,
                        _ => return Err(e),
                    });

                    warn!("Request failed, retrying in {:?}", delay);
                    tokio::time::sleep(delay).await;

                    attempts += 1;
                    continue;
                }
            }
        }
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, url);

        let request = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .build()
            .unwrap();

        self.send_request(request).await
    }

    async fn patch_body(
        &self,
        url: &str,
        body: impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, url);

        let request = self
            .client
            .patch(url)
            .json(&body)
            .bearer_auth(&self.token)
            .build()
            .unwrap();

        self.send_request(request).await
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records
    pub async fn get_dns_records(
        &self,
    ) -> Result<SuccessResponseList<DNSRecord>, CloudFlareClientError> {
        //TODO: implement paging ?page=1

        let url = format!(
            "/client/v4/zones/{}/dns_records?per_page=5000000",
            self.zone_id
        );

        let res = match self.get(&url).await {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        let status = res.status();
        let text = res.text().await.map_err(CloudFlareClientError::Response)?;

        match status {
            StatusCode::OK => {
                let parsed_json = serde_json::from_str(&text)
                    .map_err(|e| CloudFlareClientError::Other(e.to_string()))?;
                Ok(parsed_json)
            }
            _ => {
                let parsed_json = serde_json::from_str(&text)
                    .map_err(|e| CloudFlareClientError::Other(e.to_string()))?;
                Err(CloudFlareClientError::Api(parsed_json))
            }
        }
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records
    pub async fn get_dns_records_with_content(
        &self,
        content: &str,
    ) -> Result<SuccessResponseList<DNSRecord>, CloudFlareClientError> {
        //TODO: implement paging ?page=1

        let url = format!(
            "/client/v4/zones/{}/dns_records?per_page=5000000&content={}",
            self.zone_id, content
        );

        let res = match self.get(&url).await {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        let status = res.status();
        let text = res.text().await.map_err(CloudFlareClientError::Response)?;

        match status {
            StatusCode::OK => {
                let parsed_json = serde_json::from_str(&text)
                    .map_err(|e| CloudFlareClientError::Other(e.to_string()))?;
                Ok(parsed_json)
            }
            _ => {
                let parsed_json = serde_json::from_str(&text)
                    .map_err(|e| CloudFlareClientError::Other(e.to_string()))?;
                Err(CloudFlareClientError::Api(parsed_json))
            }
        }
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-patch-dns-record
    pub async fn set_dns_record(
        &self,
        request: UpdateDNSRecordRequest,
    ) -> Result<(), CloudFlareClientError> {
        let url = format!(
            "/client/v4/zones/{}/dns_records/{}",
            self.zone_id, request.id
        );

        let res = match self.patch_body(&url, request).await {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        let status = res.status();
        let text = res.text().await.map_err(CloudFlareClientError::Response)?;

        match status {
            StatusCode::OK => Ok(()),
            _ => {
                let parsed_json = serde_json::from_str(&text)
                    .map_err(|e| CloudFlareClientError::Other(e.to_string()))?;
                Err(CloudFlareClientError::Api(parsed_json))
            }
        }
    }

    /// api doc: https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-patch-dns-record
    pub async fn set_dns_record_content(
        &self,
        id: &str,
        content: &str,
    ) -> Result<(), CloudFlareClientError> {
        let url = format!("/client/v4/zones/{}/dns_records/{}", self.zone_id, id);

        #[derive(serde::Serialize)]
        struct Body {
            content: String,
        }

        let res = match self
            .patch_body(
                &url,
                Body {
                    content: String::from(content),
                },
            )
            .await
        {
            Ok(res) => res,
            Err(e) => return Err(CloudFlareClientError::Request(e)),
        };

        let status = res.status();
        let text = res.text().await.map_err(CloudFlareClientError::Response)?;

        match status {
            StatusCode::OK => Ok(()),
            _ => {
                let parsed_json = serde_json::from_str(&text)
                    .map_err(|e| CloudFlareClientError::Other(e.to_string()))?;
                Err(CloudFlareClientError::Api(parsed_json))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use httpmock::prelude::*;

    use crate::cloudflare::{
        client::CloudFlareClient,
        models::{
            CloudFlareClientError, DNSRecord, DNSType, ErrorResponse, Message, ResultInfo,
            SuccessResponseList, ZoneId,
        },
    };

    fn simple_dnsrecord_reponse() -> SuccessResponseList<DNSRecord> {
        SuccessResponseList::<DNSRecord> {
            errors: vec![],
            messages: vec![],
            success: true,
            result_info: ResultInfo {
                count: 1,
                page: 1,
                per_page: 500,
                total_count: 1,
            },
            result: vec![DNSRecord {
                content: String::from(""),
                name: String::from(""),
                proxied: Some(false),
                r#type: DNSType::A,
                comment: None,
                comment_modified_on: None,
                created_on: DateTime::from_timestamp(0, 0).unwrap(),
                id: String::from(""),
                meta: serde_json::Value::Null,
                modified_on: DateTime::from_timestamp(0, 0).unwrap(),
                proxiable: true,
                tags: None,
                tags_modified_on: None,
                ttl: None,
            }],
        }
    }
    fn simple_api_error() -> ErrorResponse {
        ErrorResponse {
            errors: vec![Message {
                code: 0,
                message: String::from("error"),
            }],
            messages: vec![],
            success: false,
        }
    }

    #[tokio::test]
    async fn get_dns_records_with_content_backend_returns_ok() {
        let server = MockServer::start();
        let cloudflare_mock = server.mock(|when, then| {
            when.method(GET).path_includes("/dns_records");
            then.status(200)
                .header("content-type", "application/json")
                .body(&serde_json::to_string(&simple_dnsrecord_reponse()).unwrap());
        });

        let client = CloudFlareClient::new_with_url(
            "",
            ZoneId::new("12345678901234567890123456789012").unwrap(),
            &server.url("/"),
        );

        let response = client.get_dns_records_with_content("test").await;

        cloudflare_mock.assert();

        assert_eq!(response.unwrap(), simple_dnsrecord_reponse());
    }

    #[tokio::test]
    async fn get_dns_records_with_content_backend_returns_404() {
        let server = MockServer::start();
        let cloudflare_mock = server.mock(|when, then| {
            when.method(GET).path_includes("/dns_records");
            then.status(404)
                .header("content-type", "application/json")
                .body(&serde_json::to_string(&simple_api_error()).unwrap());
        });

        let client = CloudFlareClient::new_with_url(
            "",
            ZoneId::new("12345678901234567890123456789012").unwrap(),
            &server.url("/"),
        );

        let response = client.get_dns_records_with_content("test").await;

        cloudflare_mock.assert();

        if let CloudFlareClientError::Api(error) = response.unwrap_err() {
            assert_eq!(error, simple_api_error());
        } else {
            panic!("wrong");
        }
    }
}
