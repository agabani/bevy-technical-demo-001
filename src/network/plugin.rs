use bevy::prelude::*;

#[cfg(feature = "server")]
use crate::database;

use crate::network::protocol;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            // forwarding...
            .add_event::<protocol::Event>()
            .add_system_to_stage(CoreStage::First, forward)
            // connection
            .add_system(create_connection)
            .add_system(destroy_connection);

        #[cfg(feature = "server")]
        {
            app.add_startup_system(setup).add_system(update_last_seen);
        }
    }
}

#[derive(Component)]
pub(crate) struct Connection(pub(crate) tokio::sync::mpsc::UnboundedSender<protocol::Payload>);

#[derive(Component)]
pub(crate) struct ConnectionId(pub(crate) usize);

struct ServerDiscoveryTimer(Timer);

pub(crate) struct ServerPublicId(pub(crate) uuid::Uuid);

pub(crate) struct ServerEndpoint {
    pub(crate) ip_address: String,
    pub(crate) port: u16,
}

#[cfg(feature = "server")]
fn setup(
    mut commands: Commands,
    server_public_id: Res<ServerPublicId>,
    server_endpoint: Res<ServerEndpoint>,
    sender: ResMut<database::protocol::Sender>,
) {
    commands.insert_resource(ServerDiscoveryTimer(Timer::new(
        std::time::Duration::from_secs(1),
        true,
    )));

    sender
        .send(database::protocol::Request::ServerRegister {
            public_id: server_public_id.0.clone(),
            ip_address: server_endpoint.ip_address.clone(),
            port: server_endpoint.port,
        })
        .unwrap();
}

#[cfg(feature = "server")]
fn update_last_seen(
    time: Res<Time>,
    mut timer: ResMut<ServerDiscoveryTimer>,
    server_public_id: Res<ServerPublicId>,
    sender: ResMut<database::protocol::Sender>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        sender
            .send(database::protocol::Request::ServerUpdateServerLastSeen {
                public_id: server_public_id.0.clone(),
            })
            .unwrap();
    }
}

/// Forwards events from tokio runtime to bevy runtime.
fn forward(mut receiver: ResMut<protocol::Receiver>, mut writer: EventWriter<protocol::Event>) {
    loop {
        match receiver.try_recv() {
            Ok(event) => writer.send(event),
            Err(_) => return,
        }
    }
}

/// Creates a connection entity when a new connection is created.
fn create_connection(mut commands: Commands, mut reader: EventReader<protocol::Event>) {
    for event in reader.iter() {
        if let protocol::Data::Connection(connection) = &event.data {
            if let protocol::Connection::Created(created) = connection {
                let span = info_span!("connection", connection_id = event.connection_id);
                let _guard = span.enter();

                let entity = commands
                    .spawn()
                    .insert(Name::new("connection"))
                    .insert(Connection(created.sender.clone()))
                    .insert(ConnectionId(event.connection_id))
                    .id();
                info!(entity_id = entity.id(), "connection created");

                // TODO: start workflow from UI
                #[cfg(feature = "client")]
                created
                    .sender
                    .send(protocol::Payload::V1(protocol::Version1::Spawn(
                        protocol::Spawn,
                    )))
                    .unwrap();
            }
        }
    }
}

/// Destroys a connection entity when a connection is destroyed.
fn destroy_connection(
    mut commands: Commands,
    mut reader: EventReader<protocol::Event>,
    query: Query<(Entity, &Connection, &ConnectionId)>,
) {
    for event in reader.iter() {
        if let protocol::Data::Connection(connection) = &event.data {
            if let protocol::Connection::Destroyed(_) = connection {
                let span = info_span!("connection", connection_id = event.connection_id);
                let _guard = span.enter();

                for (entity, _, _) in query
                    .iter()
                    .filter(|(_, _, connection_id)| event.connection_id == connection_id.0)
                {
                    commands.entity(entity).despawn();
                    info!(entity_id = entity.id(), "connection destroyed");
                }
            }
        }
    }
}
