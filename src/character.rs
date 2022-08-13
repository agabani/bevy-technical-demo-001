use bevy::prelude::*;

use crate::network::{self, protocol};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawned).add_system(despawned);

        #[cfg(feature = "server")]
        {
            app.add_system(spawn)
                .add_system(despawn)
                .add_system(despawn_network);
        }
    }
}

#[derive(Component)]
pub(crate) struct Character(pub(crate) String);

/// Spawns a character if connection does not have a spawned character.
#[cfg(feature = "server")]
fn spawn(
    mut reader: EventReader<network::protocol::Event>,
    mut writer: EventWriter<network::protocol::Event>,
    characters: Query<(&Character, &network::plugin::ConnectionId)>,
    connections: Query<(&network::plugin::Connection, &network::plugin::ConnectionId)>,
) {
    for event in reader.iter() {
        let _spawn = match &event.data {
            protocol::Data::Payload(payload) => match payload {
                protocol::Payload::V1(v1) => match v1 {
                    protocol::Version1::Spawn(spawn) => spawn,
                    _ => continue,
                },
            },
            _ => continue,
        };

        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        let character_exists = characters
            .iter()
            .any(|(_, connection)| connection.0 == event.connection_id);

        if character_exists {
            info!("character spawn denied");
            continue;
        }

        info!("character spawn approved");

        let spawned = network::protocol::Payload::V1(network::protocol::Version1::Spawned(
            network::protocol::Spawned {
                id: uuid::Uuid::new_v4().to_string(),
                x: 0.0,
                y: 0.0,
            },
        ));

        connections.for_each(|(connection, connection_id)| {
            let span = info_span!("peer", connection_id = ?connection_id.0);
            let _guard = span.enter();

            if let Err(error) = connection.0.send(spawned.clone()) {
                error!(error = ?error, connection_id = connection_id.0, "broadcast failed");
            } else {
                info!("broadcast sent")
            }
        });

        writer.send(network::protocol::Event {
            connection_id: event.connection_id,
            data: network::protocol::Data::Payload(spawned),
        });
    }
}

fn spawned(mut commands: Commands, mut reader: EventReader<network::protocol::Event>) {
    for event in reader.iter() {
        let spawned = match &event.data {
            protocol::Data::Payload(payload) => match payload {
                protocol::Payload::V1(v1) => match v1 {
                    protocol::Version1::Spawned(spawned) => spawned,
                    _ => continue,
                },
            },
            _ => continue,
        };

        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        let entity = commands
            .spawn()
            .insert(Name::new("character"))
            .insert(Character(spawned.id.clone()))
            .insert(network::plugin::ConnectionId(event.connection_id))
            .id();
        info!(entity_id = entity.id(), "character spawned");
    }
}

#[cfg(feature = "server")]
fn despawn(mut events: EventReader<network::protocol::Event>) {
    for event in events.iter() {
        let despawn = match &event.data {
            protocol::Data::Payload(payload) => match payload {
                protocol::Payload::V1(v1) => match v1 {
                    protocol::Version1::Spawned(spawned) => spawned,
                    _ => continue,
                },
            },
            _ => continue,
        };

        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!(payload = ?despawn, "TODO: handle despawn");
    }
}

// Triggers character despawn on network destruction
#[cfg(feature = "server")]
fn despawn_network(
    mut reader: EventReader<network::protocol::Event>,
    mut writer: EventWriter<network::protocol::Event>,
    characters: Query<(&Character, &network::plugin::ConnectionId)>,
    connections: Query<(&network::plugin::Connection, &network::plugin::ConnectionId)>,
) {
    for event in reader.iter() {
        let _destroyed = match &event.data {
            protocol::Data::Connection(connection) => match connection {
                protocol::Connection::Destroyed(destroyed) => destroyed,
                _ => continue,
            },
            _ => continue,
        };

        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!("character network despawn approved");

        let disconnected = characters
            .iter()
            .filter(|(_, connection_id)| connection_id.0 == event.connection_id);

        for (disconnected_character, disconnected_connection_id) in disconnected {
            let despawned = network::protocol::Payload::V1(network::protocol::Version1::Despawned(
                network::protocol::Despawned {
                    id: disconnected_character.0.clone(),
                },
            ));

            connections.for_each(|(connection, connection_id)| {
                let span = info_span!("peer", connection_id = ?connection_id.0);
                let _guard = span.enter();

                if let Err(error) = connection.0.send(despawned.clone()) {
                    let is_despawned_connection = connection_id.0 == disconnected_connection_id.0;
                    if is_despawned_connection {
                        info!(error = ?error, connection_id = connection_id.0, "broadcast failed");
                    } else {
                        error!(error = ?error, connection_id = connection_id.0, "broadcast failed");
                    }
                } else {
                    info!("broadcast sent")
                }
            });

            writer.send(network::protocol::Event {
                connection_id: event.connection_id,
                data: network::protocol::Data::Payload(despawned),
            });
        }
    }
}

fn despawned(
    mut commands: Commands,
    mut reader: EventReader<network::protocol::Event>,
    characters: Query<(Entity, &Character)>,
) {
    for event in reader.iter() {
        let despawned = match &event.data {
            protocol::Data::Payload(payload) => match payload {
                protocol::Payload::V1(v1) => match v1 {
                    protocol::Version1::Despawned(despawned) => despawned,
                    _ => continue,
                },
            },
            _ => continue,
        };

        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        for (entity, character) in characters
            .iter()
            .filter(|(_, character)| character.0 == despawned.id)
        {
            let span = info_span!("character", character_id = character.0);
            let _guard = span.enter();

            commands.entity(entity).despawn();
            info!(entity_id = entity.id(), "character despawned");
        }
    }
}
