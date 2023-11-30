use bevy::prelude::*;

use crate::server::*;

pub(crate) fn setup_app(app: &mut App) {
    app.add_plugins(DefaultPlugins);

    app.add_state::<GameState>();
    
    // main menu
    app.add_systems(Update, bevy::window::close_on_esc)
        .add_systems(OnEnter(GameState::MainMenu), setup_menu_system)
        .add_systems(Update, menu_system.run_if(in_state(GameState::MainMenu)))
        .add_systems(OnExit(GameState::MainMenu), teardown_menu_system);

    // hosting a server in a client
    app.add_systems(OnEnter(GameState::HostingLobby), start_server_system)
        .add_systems(Update, server_system.run_if(in_state(GameState::HostingLobby)));

    // joining as client
    app.add_systems(OnEnter(GameState::JoiningLobby), start_client_system)
        .add_systems(Update, client_system.run_if(in_state(GameState::JoiningLobby)));

    // every app is client
    // I think the example is more advanced here
}

#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    MainMenu,
    HostingLobby,
    JoiningLobby,
    Running,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GameSystems {
    HostSystems,
    ClientSystems,
}

fn start_client_system() {
    println!("Starting client system");
}

fn client_system() {
    println!("This is a client running");
}

fn stop_client_system() {
    println!("Stopping client system");
}

fn setup_menu_system() {
    println!("Setting up menu");
}

fn menu_system() {
    println!("This is a menu system");
}

fn teardown_menu_system() {
    println!("Tearing down menu");
}
