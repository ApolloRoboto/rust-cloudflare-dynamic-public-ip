use std::net::Ipv4Addr;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use derive_builder::Builder;

#[derive(Debug, Clone, PartialEq)]
pub enum IpMonitorMessage {
    Started(Ipv4Addr),
    IpChanged { old_ip: Ipv4Addr, new_ip: Ipv4Addr },
    CouldNotGetIp,
    NoChange,
}

#[derive(Debug, Default, Clone, PartialEq, Builder)]
pub struct IpMonitorConfig {
    #[builder(default = "Duration::from_secs(60*10)")]
    pub wait_time: Duration,
}

#[derive(Debug)]
pub struct IpMonitor {
    config: IpMonitorConfig,
    tx: Sender<IpMonitorMessage>,
    rx: Receiver<IpMonitorMessage>,
}

impl Default for IpMonitor {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            config: Default::default(),
            tx,
            rx,
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

    pub fn start(&mut self) {
        let wait_time = self.config.wait_time;
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let start_ip = public_ip::addr_v4()
                .await
                .expect("Could not get public IP address");

            let mut old_ip = start_ip;

            tx.send(IpMonitorMessage::Started(start_ip)).unwrap();

            loop {
                if let Some(current_ip) = public_ip::addr_v4().await {
                    if old_ip != current_ip {
                        tx.send(IpMonitorMessage::IpChanged {
                            old_ip,
                            new_ip: current_ip,
                        })
                        .unwrap();
                        old_ip = current_ip;
                    } else {
                        tx.send(IpMonitorMessage::NoChange).unwrap();
                    }
                } else {
                    tx.send(IpMonitorMessage::CouldNotGetIp).unwrap();
                }

                tokio::time::sleep(wait_time).await;
            }
        });
    }

    pub fn listen(&self) -> &Receiver<IpMonitorMessage> {
        &self.rx
    }
}
