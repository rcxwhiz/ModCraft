use std::sync::{Arc, Mutex};

use bevy::{app::AppExit, prelude::*};

type Flag = Arc<Mutex<bool>>;

#[derive(Resource)]
struct ClientLeftFlagResource {
    flag: Flag,
}
impl ClientLeftFlagResource {
    fn new(flag: Flag) -> Self {
        Self { flag }
    }
}

pub(crate) struct ClientLeftFlagPlugin {
    flag: Flag,
}
impl ClientLeftFlagPlugin {
    pub(crate) fn new(flag: Flag) -> Self {
        Self { flag }
    }

    fn check_flag(
        client_left_flag: Res<ClientLeftFlagResource>,
        mut app_exit_events: EventWriter<AppExit>,
    ) {
        if *(client_left_flag.flag)
            .lock()
            .expect("Server failed to get lock for client left flag")
        {
            app_exit_events.send(AppExit);
        }
    }
}
impl Plugin for ClientLeftFlagPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientLeftFlagResource::new(Arc::clone(&self.flag)));
        app.add_systems(FixedUpdate, Self::check_flag);
    }
}

#[derive(Resource)]
struct ServerReadyFlagResource {
    flag: Flag,
}
impl ServerReadyFlagResource {
    fn new(flag: Flag) -> Self {
        Self { flag }
    }
}

pub(crate) struct ServerReadyFlagPlugin {
    flag: Flag,
}
impl ServerReadyFlagPlugin {
    pub(crate) fn new(flag: Flag) -> Self {
        Self { flag }
    }

    fn set_flag(server_ready_flag: Res<ServerReadyFlagResource>) {
        *(server_ready_flag.flag)
            .lock()
            .expect("Server failed to get lock for server ready flag") = true;
    }
}
impl Plugin for ServerReadyFlagPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerReadyFlagResource::new(Arc::clone(&self.flag)));
        app.add_systems(PostStartup, Self::set_flag);
    }
}
