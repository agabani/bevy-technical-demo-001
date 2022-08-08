use bevy::prelude::*;

use crate::protocol;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(multiplex)
            // connection
            .add_event::<protocol::Endpoint<protocol::Created>>()
            .add_event::<protocol::Endpoint<protocol::Destroyed>>()
            .add_system(create_connection)
            .add_system(destroy_connection)
            // spawn
            .add_event::<protocol::Endpoint<protocol::Spawn>>()
            .add_event::<protocol::Endpoint<protocol::Spawned>>()
            // despawn
            .add_event::<protocol::Endpoint<protocol::Despawn>>()
            .add_event::<protocol::Endpoint<protocol::Despawned>>();
    }
}

#[derive(Component)]
pub(crate) struct Connection {
    connection_id: usize,
    _sender: tokio::sync::mpsc::UnboundedSender<protocol::Payload>,
}

fn multiplex(
    mut receiver: ResMut<tokio::sync::mpsc::UnboundedReceiver<protocol::Event>>,
    // connection
    mut created_writer: EventWriter<protocol::Endpoint<protocol::Created>>,
    mut destroyed_writer: EventWriter<protocol::Endpoint<protocol::Destroyed>>,
    // spawn
    mut spawn_writer: EventWriter<protocol::Endpoint<protocol::Spawn>>,
    mut spawned_writer: EventWriter<protocol::Endpoint<protocol::Spawned>>,
    // despawn
    mut despawn_writer: EventWriter<protocol::Endpoint<protocol::Despawn>>,
    mut despawned_writer: EventWriter<protocol::Endpoint<protocol::Despawned>>,
) {
    match receiver.try_recv() {
        Ok(event) => match event.data {
            protocol::Data::Connection(connection) => match connection {
                protocol::Connection::Created(data) => created_writer.send(protocol::Endpoint {
                    connection_id: event.connection_id,
                    data,
                }),
                protocol::Connection::Destroyed(data) => {
                    destroyed_writer.send(protocol::Endpoint {
                        connection_id: event.connection_id,
                        data,
                    })
                }
            },
            protocol::Data::Payload(payload) => match payload {
                protocol::Payload::V1(v1) => match v1 {
                    protocol::Version1::Ping => {}
                    protocol::Version1::Pong => {}
                    protocol::Version1::Spawn(data) => spawn_writer.send(protocol::Endpoint {
                        connection_id: event.connection_id,
                        data,
                    }),
                    protocol::Version1::Spawned(data) => spawned_writer.send(protocol::Endpoint {
                        connection_id: event.connection_id,
                        data,
                    }),
                    protocol::Version1::Despawn(data) => despawn_writer.send(protocol::Endpoint {
                        connection_id: event.connection_id,
                        data,
                    }),
                    protocol::Version1::Despawned(data) => {
                        despawned_writer.send(protocol::Endpoint {
                            connection_id: event.connection_id,
                            data,
                        })
                    }
                },
            },
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

        #[cfg(feature = "client")]
        event
            .data
            .sender
            .send(protocol::Payload::V1(protocol::Version1::Spawn(
                protocol::Spawn,
            )))
            .unwrap();
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
