use bevy::prelude::*;

use crate::protocol;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<protocol::Endpoint<protocol::Created>>()
            .add_event::<protocol::Endpoint<protocol::Destroyed>>()
            .add_event::<protocol::Endpoint<protocol::Payload>>()
            .add_system(multiplex)
            .add_system(create_connection)
            .add_system(destroy_connection)
            .add_system(read_payload);
    }
}

#[derive(Component)]
pub(crate) struct Connection {
    connection_id: usize,
    _sender: tokio::sync::mpsc::UnboundedSender<protocol::Payload>,
}

fn multiplex(
    mut receiver: ResMut<tokio::sync::mpsc::UnboundedReceiver<protocol::Event>>,
    mut connection_created_writer: EventWriter<protocol::Endpoint<protocol::Created>>,
    mut connection_destroyed_writer: EventWriter<protocol::Endpoint<protocol::Destroyed>>,
    mut payload_received_writer: EventWriter<protocol::Endpoint<protocol::Payload>>,
) {
    match receiver.try_recv() {
        Ok(event) => match event.data {
            protocol::Data::Connection(connection) => match connection {
                protocol::Connection::Created(created) => {
                    connection_created_writer.send(protocol::Endpoint {
                        connection_id: event.connection_id,
                        data: created,
                    })
                }
                protocol::Connection::Destroyed(destroyed) => {
                    connection_destroyed_writer.send(protocol::Endpoint {
                        connection_id: event.connection_id,
                        data: destroyed,
                    })
                }
            },
            protocol::Data::Payload(payload) => payload_received_writer.send(protocol::Endpoint {
                connection_id: event.connection_id,
                data: payload,
            }),
        },
        Err(err) => match err {
            tokio::sync::mpsc::error::TryRecvError::Empty => {}
            tokio::sync::mpsc::error::TryRecvError::Disconnected => {}
        },
    }
}

fn create_connection(
    mut commands: Commands,
    mut reader: EventReader<protocol::Endpoint<protocol::Created>>,
) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = event.connection_id);
        let _guard = span.enter();

        info!("creating connection");

        let connection = Connection {
            connection_id: event.connection_id,
            _sender: event.data.sender.clone(),
        };

        commands
            .spawn()
            .insert(Name::new(format!(
                "connection {}",
                connection.connection_id
            )))
            .insert(connection);
    }
}

fn destroy_connection(
    mut commands: Commands,
    mut query: Query<(Entity, &Connection)>,
    mut reader: EventReader<protocol::Endpoint<protocol::Destroyed>>,
) {
    for event in reader.iter() {
        for (entity, connection) in query.iter_mut() {
            let span = info_span!("connection", connection_id = connection.connection_id);
            let _guard = span.enter();

            if connection.connection_id == event.connection_id {
                info!("destroying connection");
                commands.entity(entity).despawn();
            }
        }
    }
}

fn read_payload(mut reader: EventReader<protocol::Endpoint<protocol::Payload>>) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        debug!(payload = ?event.data, "read");
    }
}
