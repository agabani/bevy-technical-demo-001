pub(crate) type Receiver = tokio::sync::mpsc::UnboundedReceiver<Response>;
pub(crate) type Sender = tokio::sync::mpsc::UnboundedSender<Request>;

pub(crate) type InternalReceiver = tokio::sync::mpsc::UnboundedReceiver<Request>;
pub(crate) type InternalSender = tokio::sync::mpsc::UnboundedSender<Response>;

#[derive(Debug)]
pub(crate) enum Request {
    ServerRegister {
        public_id: uuid::Uuid,
        ip_address: String,
        port: u16,
    },
    ServerUpdateServerLastSeen {
        public_id: uuid::Uuid,
    },
}

#[derive(Debug)]
pub(crate) enum Response {
    ServerRegistered,
    ServerRegisterConflicted,
    ServerDeregistered,
}
