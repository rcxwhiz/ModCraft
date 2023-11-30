use std::{
    collections::HashMap,
    env,
    thread::{self, sleep},
    time::Duration,
};

use bevy::{
    app::AppExit,
    prelude::*,
    utils::tracing::{info, warn},
};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode,
        connection::{ConnectionConfiguration, ConnectionEvent},
        Client, QuinnetClientPlugin,
    },
    shared::ClientId, server::QuinnetServerPlugin,
};
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::mpsc;

use crate::{protocol::{ClientMessage, ServerMessage}, server};

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    names: HashMap<ClientId, String>,
}

#[derive(Resource, Deref, DerefMut)]
struct TerminalReceiver(mpsc::Receiver<String>);

#[derive(Resource)]
struct ServerAddress(ConnectionConfiguration);

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, client: Res<Client>) {
    if !app_exit_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        sleep(Duration::from_secs_f32(0.1));
    }
}

fn handle_server_messages(mut users: ResMut<Users>, mut client: ResMut<Client>) {
    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::ClientConnected {
                client_id,
                username,
            } => {
                info!("{} joined", username);
                users.names.insert(client_id, username);
            }
            ServerMessage::ClientDisconnected { client_id } => {
                if let Some(username) = users.names.remove(&client_id) {
                    println!("{} left", username);
                } else {
                    warn!("ClientDisconnected for an unknown client_id: {}", client_id);
                }
            }
            ServerMessage::ChatMessage { client_id, message } => {
                if let Some(username) = users.names.get(&client_id) {
                    if client_id != users.self_id {
                        println!("{}: {}", username, message);
                    }
                } else {
                    warn!("Chat message from an unknown client_id: {}", client_id);
                }
            }
            ServerMessage::InitClient {
                client_id,
                usernames,
            } => {
                users.self_id = client_id;
                users.names = usernames;
            }
        }
    }
}

fn handle_terminal_messages(
    mut terminal_messages: ResMut<TerminalReceiver>,
    mut app_exit_events: EventWriter<AppExit>,
    client: Res<Client>,
) {
    while let Ok(message) = terminal_messages.try_recv() {
        if message == "quit" {
            app_exit_events.send(AppExit);
        } else {
            client
                .connection()
                .try_send_message(ClientMessage::ChatMessage { message });
        }
    }
}

fn start_terminal_listener(mut commands: Commands) {
    let (from_terminal_sender, from_terminal_receiver) = mpsc::channel::<String>(100);

    thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        from_terminal_sender
            .try_send(buffer.trim_end().to_string())
            .unwrap();
    });

    commands.insert_resource(TerminalReceiver(from_terminal_receiver));
}

fn start_connection(mut client: ResMut<Client>, server_address: Res<ServerAddress>) {
    client
        .open_connection(
            server_address.0.clone(),
            CertificateVerificationMode::SkipVerification,
        )
        .unwrap();
}

fn handle_client_events(
    mut connection_events: EventReader<ConnectionEvent>,
    client: ResMut<Client>,
) {
    if !connection_events.is_empty() {
        let username: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        println!("--- Joining with name: {}", username);
        println!("--- Type 'quit' to disconnect");

        client
            .connection()
            .send_message(ClientMessage::Join { name: username })
            .unwrap();

        connection_events.clear();
    }
}

pub(crate) fn setup_app(app: &mut App) {
    // client stuff
    app.add_plugins(QuinnetClientPlugin::default());
    app.init_resource::<Users>();

    app.add_systems(Startup, (start_terminal_listener, start_connection))
        .add_systems(
            Update,
            (
                handle_terminal_messages,
                handle_server_messages,
                handle_client_events,
            ),
        )
        .add_systems(PostUpdate, on_app_exit);

    // self hosting flag
    let mut self_hosting = false;
    // determine if self hosting by finding a valid server ip to connect to
    let args: Vec<String> = env::args().collect();
    if let Some(server_address) = args.get(1) {
        if let Ok(server_config) =
            ConnectionConfiguration::from_strings(server_address, "0.0.0.0:0")
        {
            app.insert_resource(ServerAddress(server_config));
            info!("Starting client connected to server at {}", server_address);
        } else {
            panic!(
                "Got an invalid server address to connect to: {}",
                server_address
            );
        }
    } else {
        app.insert_resource(ServerAddress(
            ConnectionConfiguration::from_strings("127.0.0.1:6006", "0.0.0.0:0").unwrap(),
        ));
        self_hosting = true;
        info!("Starting a client with a self-hosted server");
    }

    if self_hosting { // there should be a more organized way to do this (plugins?)
        app.add_plugins(QuinnetServerPlugin::default())
            .init_resource::<server::Users>()
            .add_systems(Startup, server::start_listening.before(start_connection))
            .add_systems(Update, (server::handle_client_messages, server::handle_server_events));
    }
}
