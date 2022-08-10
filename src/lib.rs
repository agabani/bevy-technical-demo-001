pub mod app;
mod character;
pub mod config;
#[cfg(feature = "server")]
mod database;
#[cfg(feature = "server")]
mod http_server;
mod network;
pub mod protocol;
mod quic;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
