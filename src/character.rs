use bevy::prelude::*;

use crate::protocol;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn)
            .add_system(spawned)
            .add_system(despawn)
            .add_system(despawned);
    }
}

fn spawn(mut reader: EventReader<protocol::Endpoint<protocol::Spawn>>) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!(payload = ?event.data, "read");
    }
}

fn spawned(mut reader: EventReader<protocol::Endpoint<protocol::Spawned>>) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!(payload = ?event.data, "read");
    }
}

fn despawn(mut reader: EventReader<protocol::Endpoint<protocol::Despawn>>) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!(payload = ?event.data, "read");
    }
}

fn despawned(mut reader: EventReader<protocol::Endpoint<protocol::Despawned>>) {
    for event in reader.iter() {
        let span = info_span!("connection", connection_id = ?event.connection_id);
        let _guard = span.enter();

        info!(payload = ?event.data, "read");
    }
}
