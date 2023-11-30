use bevy::prelude::*;

pub(crate) fn setup_app(app: &mut App) {
    app.add_systems(Startup, start_server_system)
        .add_systems(Update, server_system.run_if(in_state(GameState::Running)))
        .add_systems(OnEnter(GameState::Stopping), stop_server_system);
}

#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Running,
    Stopping,
}

pub(crate) fn start_server_system() {
    println!("Starting server");
}

pub(crate) fn server_system() {
    println!("This is a server running");
}

pub(crate) fn stop_server_system() {
    println!("Stopping server");
}
