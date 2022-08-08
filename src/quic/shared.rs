use bevy::prelude::*;
use futures::StreamExt as _;

use crate::protocol;

pub(super) async fn handle_connection(
    connection: quinn::NewConnection,
    sender: tokio::sync::mpsc::UnboundedSender<protocol::Event>,
) -> crate::Result<()> {
    let span = info_span!(
        "connection",
        remote_address = ?connection.connection.remote_address(),
        connection_id = connection.connection.stable_id(),
        protocol = connection
            .connection
            .handshake_data()
            .unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>()
            .unwrap()
            .protocol
            .map_or_else(
                || "<none>".into(),
                |x| String::from_utf8_lossy(&x).into_owned()
            )
    );
    let _guard = span.enter();

    info!("established");

    let quinn::NewConnection {
        connection,
        uni_streams,
        bi_streams,
        ..
    } = connection;

    let (s, r) = tokio::sync::mpsc::unbounded_channel();

    sender.send(protocol::Event::ConnectionCreated(
        protocol::ConnectionCreatedEvent {
            connection_id: connection.stable_id(),
            sender: s,
        },
    ))?;

    let result = tokio::select! {
        result = handle_incoming_bi_streams(connection.clone(), sender.clone(), bi_streams) => result,
        result = handle_incoming_uni_streams(connection.clone(), sender.clone(), uni_streams) => result,
        result = handle_outgoing_keep_alive(connection.clone()) => result,
        result = handle_outgoing_stream(connection.clone(), r) => result,
    };

    sender.send(protocol::Event::ConnectionDestroyed(
        protocol::ConnectionDestroyedEvent {
            connection_id: connection.stable_id(),
        },
    ))?;

    result
}

pub(super) async fn handle_incoming_bi_streams(
    connection: quinn::Connection,
    sender: tokio::sync::mpsc::UnboundedSender<protocol::Event>,
    mut bi_streams: quinn::IncomingBiStreams,
) -> crate::Result<()> {
    while let Some(stream) = bi_streams.next().await {
        let (send, recv) = match stream {
            Ok(stream) => stream,
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                info!("connection closed");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };

        tokio::spawn(handle_incoming_bi_request(
            connection.clone(),
            sender.clone(),
            recv,
            send,
        ));
    }

    Ok(())
}

pub(super) async fn handle_incoming_uni_streams(
    connection: quinn::Connection,
    sender: tokio::sync::mpsc::UnboundedSender<protocol::Event>,
    mut uni_streams: quinn::IncomingUniStreams,
) -> crate::Result<()> {
    while let Some(stream) = uni_streams.next().await {
        let recv = match stream {
            Ok(stream) => stream,
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                info!("connection closed");
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };

        tokio::spawn(handle_incoming_uni_request(
            connection.clone(),
            sender.clone(),
            recv,
        ));
    }

    Ok(())
}

pub(super) async fn handle_incoming_bi_request(
    connection: quinn::Connection,
    sender: tokio::sync::mpsc::UnboundedSender<protocol::Event>,
    recv: quinn::RecvStream,
    mut send: quinn::SendStream,
) -> crate::Result<()> {
    let request_bytes = recv.read_to_end(64 * 1024).await?;
    let request = protocol::Payload::deserialize(&request_bytes)?;

    match &request {
        protocol::Payload::V1(v1) => match v1 {
            protocol::Version1::Ping => {
                let response = protocol::Payload::V1(protocol::Version1::Pong);
                send.write_all(&response.serialize()?).await?;
            }
            protocol::Version1::Pong => todo!(),
        },
    };

    send.finish().await?;

    sender.send(protocol::Event::PayloadReceived(
        protocol::PayloadReceivedEvent {
            connection_id: connection.stable_id(),
            payload: request,
        },
    ))?;

    Ok(())
}

pub(super) async fn handle_incoming_uni_request(
    connection: quinn::Connection,
    sender: tokio::sync::mpsc::UnboundedSender<protocol::Event>,
    recv: quinn::RecvStream,
) -> crate::Result<()> {
    let payload = recv.read_to_end(64 * 1024).await?;
    let request = protocol::Payload::deserialize(&payload)?;

    sender.send(protocol::Event::PayloadReceived(
        protocol::PayloadReceivedEvent {
            connection_id: connection.stable_id(),
            payload: request,
        },
    ))?;

    Ok(())
}

pub(super) async fn handle_outgoing_keep_alive(connection: quinn::Connection) -> crate::Result<()> {
    loop {
        let (mut send, recv) = connection.open_bi().await?;

        let request = protocol::Payload::V1(protocol::Version1::Ping);
        send.write_all(&request.serialize()?).await?;
        send.finish().await?;

        let _response = protocol::Payload::deserialize(&recv.read_to_end(64 * 1024).await?)?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

pub(super) async fn handle_outgoing_stream(
    connection: quinn::Connection,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<protocol::Payload>,
) -> crate::Result<()> {
    while let Some(payload) = receiver.recv().await {
        let mut send = connection.open_uni().await?;

        send.write_all(&payload.serialize()?).await?;
        send.finish().await?;
    }

    Ok(())
}
