use std::io::Write;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use chronosync_core::{Heartbeat, HeartbeatPayload, IngestResponse, StatsSummary};
use flate2::Compression;
use flate2::write::GzEncoder;
use reqwest::header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE, HeaderMap, HeaderValue};

pub struct ChronoClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl ChronoClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            base_url,
            api_key,
        }
    }

    fn build_headers(&self, gzip: bool) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if gzip {
            headers.insert(CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        }

        if let Some(key) = &self.api_key {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", key)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        headers
    }

    pub async fn ping(&self) -> Result<Duration> {
        let url = format!("{}/health", self.base_url);
        let start = Instant::now();
        let res = self.client.get(&url).send().await?;

        if !res.status().is_success() {
            bail!("Health check failed with HTTP {}", res.status());
        }

        Ok(start.elapsed())
    }

    pub async fn send_heartbeat(&self, hb: &Heartbeat) -> Result<IngestResponse> {
        let url = format!("{}/api/v1/heartbeat", self.base_url);
        let payload = HeartbeatPayload::Single(hb.clone());
        let headers = self.build_headers(false);

        let res = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to chronosync server")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            bail!("Server returned error {}: {}", status, text);
        }

        let resp: IngestResponse = res.json().await?;
        Ok(resp)
    }

    #[allow(dead_code)]
    pub async fn send_batch_gzip(&self, batch: &[Heartbeat]) -> Result<IngestResponse> {
        let url = format!("{}/api/v1/heartbeat", self.base_url);
        let payload = HeartbeatPayload::Batch(batch.to_vec());
        let json_bytes = serde_json::to_vec(&payload)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json_bytes)?;
        let compressed_bytes = encoder.finish()?;

        let headers = self.build_headers(true);

        let res = self
            .client
            .post(&url)
            .headers(headers)
            .body(compressed_bytes)
            .send()
            .await
            .context("Failed to send compressed batch to chronosync server")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            bail!("Server returned error {}: {}", status, text);
        }

        let resp: IngestResponse = res.json().await?;
        Ok(resp)
    }

    pub async fn get_summary(&self, days: Option<u32>) -> Result<StatsSummary> {
        let mut url = format!("{}/api/v1/stats/summary", self.base_url);
        if let Some(d) = days {
            url.push_str(&format!("?days={}", d));
        }

        let headers = self.build_headers(false);
        let res = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to query summary from chronosync server")?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            bail!("Server returned error {}: {}", status, text);
        }

        let summary: StatsSummary = res.json().await?;
        Ok(summary)
    }
}
