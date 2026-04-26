use log::warn;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum IpMonitorMessage {
    Started,
    IpChanged { old_ip: Ipv4Addr, new_ip: Ipv4Addr },
    Error(Error),
    NoChange,
}

#[derive(Debug, Clone, PartialEq, Setters)]
#[setters(prefix = "with_", generate_private = false, strip_option)]
pub struct IpMonitorConfig {
    pub wait_time: Duration,
    pub persistent_file: Option<PathBuf>,
}

impl Default for IpMonitorConfig {
    fn default() -> Self {
        Self {
            wait_time: Duration::from_secs(60 * 10),
            persistent_file: None,
        }
    }
}

#[derive(Debug)]
pub struct IpMonitor {
    config: IpMonitorConfig,
    tx: Sender<IpMonitorMessage>,
    rx: Receiver<IpMonitorMessage>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl Default for IpMonitor {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            config: Default::default(),
            tx,
            rx,
            handle: None,
        }
    }
}

impl IpMonitor {
    pub fn new(config: IpMonitorConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub async fn start(&mut self) {
        if self.is_started() {
            return;
        }

        let wait_time = self.config.wait_time;
        let persistent_file = self.config.persistent_file.clone();
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            let mut old_ip = None;

            // find initial ip from path if it exists
            if let Some(ref path) = persistent_file {
                if let Ok(data) = PersistentData::read_from_file(path).await {
                    old_ip = Some(data.last_known_ip);
                }
            }

            tx.send(IpMonitorMessage::Started).unwrap();

            loop {
                let current_ip = loop {
                    match public_ip::addr_v4().await {
                        Some(ip) => {
                            break ip;
                        }
                        None => {
                            tx.send(IpMonitorMessage::Error(Error::msg(
                                "Failed to get public IP address",
                            )))
                            .unwrap();
                            continue;
                        }
                    }
                };

                let ip_changed = old_ip.map_or(true, |ip| ip != current_ip);

                if ip_changed {
                    if let Some(prev_ip) = old_ip {
                        tx.send(IpMonitorMessage::IpChanged {
                            old_ip: prev_ip,
                            new_ip: current_ip,
                        })
                        .unwrap();
                    }

                    old_ip = Some(current_ip);

                    if let Some(ref path) = persistent_file {
                        let data = PersistentData::now(current_ip);
                        if let Err(error) = data.write_to_file(path).await {
                            warn!("Failed to write to file: {error:?}");
                        }
                    }
                } else {
                    tx.send(IpMonitorMessage::NoChange).unwrap();
                }

                tokio::time::sleep(wait_time).await;
            }
        });

        self.handle = Some(handle);
    }

    #[allow(unused)]
    pub async fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        };
    }

    pub fn is_started(&self) -> bool {
        self.handle.is_some()
    }

    pub fn listen(&self) -> &Receiver<IpMonitorMessage> {
        &self.rx
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistentData {
    pub last_known_ip: Ipv4Addr,
    pub time: DateTime<Utc>,
}
impl PersistentData {
    pub fn now(ip: Ipv4Addr) -> Self {
        Self {
            last_known_ip: ip,
            time: Utc::now(),
        }
    }

    async fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        tokio::fs::create_dir_all(&path.as_ref().parent().unwrap()).await?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?;

        let json = serde_json::to_string(&self)?;
        file.write_all(json.as_bytes()).await?;
        Ok(())
    }

    async fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(serde_json::from_str(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serialize_deserialize() {
        let file = tempfile::NamedTempFile::new().unwrap();

        let test_ips = [Ipv4Addr::new(255, 255, 255, 255), Ipv4Addr::new(0, 0, 0, 0)];
        let now = Utc::now();

        for ip in test_ips {
            let data = PersistentData {
                last_known_ip: ip,
                time: now,
            };
            data.write_to_file(&file.path().to_path_buf())
                .await
                .unwrap();

            let content = std::fs::read_to_string(&file).unwrap();
            let data: PersistentData = serde_json::from_str(&content).unwrap();
            assert_eq!(data.last_known_ip, ip);
        }
    }
}
