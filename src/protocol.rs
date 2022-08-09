#[derive(Debug)]
pub struct Event {
    pub connection_id: usize,
    pub data: Data,
}

#[derive(Debug)]
pub struct Endpoint<T> {
    pub connection_id: usize,
    pub data: T,
}

#[derive(Debug)]
pub enum Data {
    Connection(Connection),
    Payload(Payload),
}

#[derive(Debug)]
pub enum Connection {
    Created(Created),
    Destroyed(Destroyed),
}

#[derive(Debug)]
pub struct Created {
    pub sender: tokio::sync::mpsc::UnboundedSender<crate::protocol::Payload>,
}

#[derive(Debug)]
pub struct Destroyed;

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

    #[serde(rename = "spawn")]
    Spawn(Spawn),

    #[serde(rename = "spawned")]
    Spawned(Spawned),

    #[serde(rename = "despawn")]
    Despawn(Despawn),

    #[serde(rename = "despawned")]
    Despawned(Despawned),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Spawn;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Spawned {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "x")]
    pub x: f32,

    #[serde(rename = "y")]
    pub y: f32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Despawn;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Despawned {
    #[serde(rename = "id")]
    pub id: String,
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
