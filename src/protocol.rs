#[derive(Debug)]
pub enum Event {
    ConnectionCreated(ConnectionCreatedEvent),
    ConnectionDestroyed(ConnectionDestroyedEvent),
    PayloadReceived(PayloadReceivedEvent),
}

#[derive(Debug)]
pub struct ConnectionCreatedEvent {
    pub connection_id: usize,
    pub sender: tokio::sync::mpsc::UnboundedSender<crate::protocol::Payload>,
}

#[derive(Debug)]
pub struct ConnectionDestroyedEvent {
    pub connection_id: usize,
}

#[derive(Debug)]
pub struct PayloadReceivedEvent {
    pub connection_id: usize,
    pub payload: Payload,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "version", content = "payload")]
pub enum Payload {
    #[serde(rename = "1")]
    V1(Version1),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", content = "message")]
pub enum Version1 {
    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "pong")]
    Pong,
}

impl Payload {
    pub fn deserialize(payload: &[u8]) -> crate::Result<Payload> {
        Ok(serde_json::from_slice(payload)?)
    }

    pub fn serialize(&self) -> crate::Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let result = Payload::V1(Version1::Ping).serialize().unwrap();
        assert_eq!(
            String::from_utf8_lossy(&result),
            "{\"version\":\"1\",\"payload\":{\"type\":\"ping\"}}"
        );

        let result = Payload::V1(Version1::Pong).serialize().unwrap();
        assert_eq!(
            String::from_utf8_lossy(&result),
            "{\"version\":\"1\",\"payload\":{\"type\":\"pong\"}}"
        );
    }
}
