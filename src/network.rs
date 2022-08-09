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
pub(crate) struct Connection(pub(crate) tokio::sync::mpsc::UnboundedSender<protocol::Payload>);

#[derive(Component)]
pub(crate) struct ConnectionId(pub(crate) usize);

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
    mut events: EventReader<protocol::Endpoint<protocol::Created>>,
) {
    for event in events.iter() {
        let span = info_span!("connection", connection_id = event.connection_id);
        let _guard = span.enter();

        let entity = commands
            .spawn()
            .insert(Name::new("connection"))
            .insert(Connection(event.data.sender.clone()))
            .insert(ConnectionId(event.connection_id))
            .id();
        info!(entity_id = entity.id(), "connection created");

        // TODO: start workflow from UI
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
    mut events: EventReader<protocol::Endpoint<protocol::Destroyed>>,
    query: Query<(Entity, &Connection, &ConnectionId)>,
) {
    for event in events.iter() {
        for (entity, _, connection_id) in query
            .iter()
            .filter(|(_, _, connection_id)| event.connection_id == connection_id.0)
        {
            let span = info_span!("connection", connection_id = connection_id.0);
            let _guard = span.enter();

            commands.entity(entity).despawn();
            info!(entity_id = entity.id(), "connection destroyed");
        }
    }
}
