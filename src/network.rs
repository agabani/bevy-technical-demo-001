use bevy::prelude::*;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<crate::protocol::ConnectionCreatedEvent>()
            .add_event::<crate::protocol::ConnectionDestroyedEvent>()
            .add_event::<crate::protocol::PayloadReceivedEvent>()
            .add_system(multiplex)
            .add_system(create_connection)
            .add_system(destroy_connection)
            .add_system(read_payload);
    }
}

#[derive(Component)]
pub(crate) struct Connection {
    connection_id: usize,
    _sender: tokio::sync::mpsc::UnboundedSender<crate::protocol::Payload>,
}

fn multiplex(
    mut receiver: ResMut<tokio::sync::mpsc::UnboundedReceiver<crate::protocol::Event>>,
    mut connection_created_writer: EventWriter<crate::protocol::ConnectionCreatedEvent>,
    mut connection_destroyed_writer: EventWriter<crate::protocol::ConnectionDestroyedEvent>,
    mut payload_received_writer: EventWriter<crate::protocol::PayloadReceivedEvent>,
) {
    match receiver.try_recv() {
        Ok(event) => match event {
            crate::protocol::Event::ConnectionCreated(event) => {
                connection_created_writer.send(event)
            }
            crate::protocol::Event::ConnectionDestroyed(event) => {
                connection_destroyed_writer.send(event)
            }
            crate::protocol::Event::PayloadReceived(event) => payload_received_writer.send(event),
        },
        Err(err) => match err {
            tokio::sync::mpsc::error::TryRecvError::Empty => {}
            tokio::sync::mpsc::error::TryRecvError::Disconnected => {}
        },
    }
}

fn create_connection(
    mut commands: Commands,
    mut reader: EventReader<crate::protocol::ConnectionCreatedEvent>,
) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = event.connection_id);
        let _guard = span.enter();

        info!("creating connection");

        let connection = Connection {
            connection_id: event.connection_id,
            _sender: event.sender.clone(),
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
    mut reader: EventReader<crate::protocol::ConnectionDestroyedEvent>,
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

fn read_payload(mut reader: EventReader<crate::protocol::PayloadReceivedEvent>) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        debug!(payload = ?event.payload, "read");
    }
}
