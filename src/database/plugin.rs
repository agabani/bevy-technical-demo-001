use bevy::prelude::*;

use crate::database;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(multiplex);
    }
}

fn multiplex(mut receiver: ResMut<database::protocol::Receiver>) {
    match receiver.try_recv() {
        Ok(response) => {
            info!("database multiplex {:?}", response);
        }
        Err(error) => match error {
            tokio::sync::mpsc::error::TryRecvError::Empty => {}
            tokio::sync::mpsc::error::TryRecvError::Disconnected => {}
        },
    }
}
