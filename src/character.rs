use bevy::prelude::*;

use crate::network::{self, protocol};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn)
            .add_system(spawned)
            .add_system(despawn)
            .add_system(despawn_network)
            .add_system(despawned);
    }
}

#[derive(Component)]
pub(crate) struct Character(pub(crate) String);

fn spawn(
    mut events: EventReader<protocol::Endpoint<protocol::Spawn>>,
    mut spawned: EventWriter<protocol::Endpoint<protocol::Spawned>>,
    characters: Query<(&Character, &network::plugin::ConnectionId)>,
    connections: Query<(&network::plugin::Connection, &network::plugin::ConnectionId)>,
) {
    for event in events.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        #[cfg(feature = "server")]
        {
            if characters
                .iter()
                .any(|(_, connection)| connection.0 == event.connection_id)
            {
                info!("character spawn denied");
            } else {
                let id = uuid::Uuid::new_v4().to_string();

                let span = info_span!("character", character_id = id);
                let _guard = span.enter();

                let message = protocol::Spawned { id, x: 0.0, y: 0.0 };

                info!("character spawn approved");

                connections.for_each(|(connection, connection_id)| {
                    let span = info_span!("peer", connection_id = ?connection_id.0);
                    let _guard = span.enter();

                    if let Err(error) =
                        connection
                            .0
                            .send(protocol::Payload::V1(protocol::Version1::Spawned(
                                message.clone(),
                            )))
                    {
                        error!(error = ?error, connection_id = connection_id.0, "broadcast failed");
                    } else {
                        info!("broadcast sent")
                    }
                });

                spawned.send(protocol::Endpoint {
                    connection_id: event.connection_id,
                    data: message,
                });
            }
        }
    }
}

fn spawned(mut commands: Commands, mut events: EventReader<protocol::Endpoint<protocol::Spawned>>) {
    for event in events.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        let entity = commands
            .spawn()
            .insert(Name::new("character"))
            .insert(Character(event.data.id.clone()))
            .insert(network::plugin::ConnectionId(event.connection_id))
            .id();
        info!(entity_id = entity.id(), "character spawned");
    }
}

fn despawn(mut events: EventReader<protocol::Endpoint<protocol::Despawn>>) {
    for event in events.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!(payload = ?event.data, "read");
    }
}

fn despawn_network(
    mut events: EventReader<protocol::Endpoint<protocol::Destroyed>>,
    mut despawned: EventWriter<protocol::Endpoint<protocol::Despawned>>,
    characters: Query<(&Character, &network::plugin::ConnectionId)>,
    connections: Query<(&network::plugin::Connection, &network::plugin::ConnectionId)>,
) {
    for event in events.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        #[cfg(feature = "server")]
        {
            for (character, disconnected_connection_id) in characters
                .iter()
                .filter(|(_, connection_id)| event.connection_id == connection_id.0)
            {
                let message = protocol::Despawned {
                    id: character.0.clone(),
                };

                info!("character network despawn approved");

                connections.for_each(|(connection, connection_id)| {
                    let span = info_span!("peer", connection_id = ?connection_id.0);
                    let _guard = span.enter();

                    if let Err(error) =
                        connection
                            .0
                            .send(protocol::Payload::V1(protocol::Version1::Despawned(
                                message.clone(),
                            )))
                    {
                        if connection_id.0 == disconnected_connection_id.0 {
                            info!(error = ?error, connection_id = connection_id.0, "broadcast failed");
                        } else {
                            error!(error = ?error, connection_id = connection_id.0, "broadcast failed");
                        }
                    } else {
                        info!("broadcast sent")
                    }
                });

                despawned.send(protocol::Endpoint {
                    connection_id: event.connection_id,
                    data: message,
                });
            }
        }
    }
}

fn despawned(
    mut commands: Commands,
    mut events: EventReader<protocol::Endpoint<protocol::Despawned>>,
    characters: Query<(Entity, &Character)>,
) {
    for event in events.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        for (entity, character) in characters
            .iter()
            .filter(|(_, character)| character.0 == event.data.id)
        {
            let span = info_span!("character", character_id = character.0);
            let _guard = span.enter();

            commands.entity(entity).despawn();
            info!(entity_id = entity.id(), "character despawned");
        }
    }
}
