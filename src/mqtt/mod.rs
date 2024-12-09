use std::net::Ipv4Addr;

use bincode::ErrorKind;
use bytes::Bytes;
use log::{debug, error, trace};
use rumqttc::{AsyncClient, ClientError, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tokio::task;

pub struct MqttClient {
    client: AsyncClient,
    base_topic: String,
}

impl MqttClient {
    pub async fn new(host: &str, port: u16, id: &str, base_topic: &str) -> Self {
        let mut mqttoptions = MqttOptions::new(id, host, port);
        mqttoptions
            .set_keep_alive(std::time::Duration::from_secs(60))
            .set_clean_session(true);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 10);

        debug!("MQTT options: {:?}", mqttoptions);

        task::spawn(async move {
            trace!("Starting MQTT event loop");
            loop {
                match eventloop.poll().await {
                    Ok(v) => {
                        trace!("MQTT Event = {v:?}");
                    }
                    Err(e) => {
                        error!("MQTT Event Error = {e:?}");
                    }
                }
            }
        });

        // todo: configure the authentication

        Self {
            client,
            base_topic: String::from(base_topic),
        }
    }

    pub async fn publish_ip_change(&self, payload: IpChangeMessage) -> Result<(), ClientError> {
        let topic = format!("{}/ipchange", self.base_topic);
        debug!("MQTT publishing to {}", topic);

        let payload: Bytes = payload.into();

        self.client
            .publish(&topic, QoS::AtLeastOnce, false, payload)
            .await
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IpChangeMessage {
    pub old: Ipv4Addr,
    pub new: Ipv4Addr,
}

impl From<&IpChangeMessage> for Vec<u8> {
    fn from(value: &IpChangeMessage) -> Self {
        bincode::serialize(value).unwrap()
    }
}

impl From<IpChangeMessage> for Vec<u8> {
    fn from(value: IpChangeMessage) -> Self {
        bincode::serialize(&value).unwrap()
    }
}

impl TryFrom<&[u8]> for IpChangeMessage {
    type Error = Box<ErrorKind>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize(value)
    }
}

impl From<IpChangeMessage> for Bytes {
    fn from(value: IpChangeMessage) -> Bytes {
        let json = serde_json::to_vec(&value).expect("Failed to serialize IpChangeMessage");
        Bytes::from(json)
    }
}
