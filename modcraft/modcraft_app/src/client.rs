use std::{
    collections::HashMap,
    thread::{self},
};

use bevy::prelude::*;
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode,
        connection::{ConnectionConfiguration, ConnectionEvent},
        Client, QuinnetClientPlugin,
    },
    server::QuinnetServerPlugin,
    shared::ClientId,
};
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::mpsc;

use crate::{
    protocol::{ClientMessage, ServerMessage},
    server::{self, SetHostEvent},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum ClientState {
    #[default]
    Menu,
    ClientToServer,
    HostingServer,
}

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    names: HashMap<ClientId, String>,
}

#[derive(Resource, Deref, DerefMut)]
struct TerminalReceiver(mpsc::Receiver<String>);

#[derive(Resource)]
struct ClientConnectionConfig(ConnectionConfiguration);

fn prompt() {
    println!("Enter an address to connect to. Enter blank to self host.")
}

fn leave_server(mut client: ResMut<Client>) {
    client
        .connection()
        .send_message(ClientMessage::Disconnect {})
        .unwrap();
    client.close_all_connections().unwrap(); // idk if this counts as a graceful termination
}

fn handle_server_messages(
    mut users: ResMut<Users>,
    mut client: ResMut<Client>,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::ClientConnected {
                client_id,
                username,
            } => {
                println!("{} joined", username);
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
            ServerMessage::ServerStopping => {
                info!("Server shutdown!");
                client_state.set(ClientState::Menu);
            }
        }
    }
}

fn handle_terminal_messages(
    mut terminal_messages: ResMut<TerminalReceiver>,
    mut client_state: ResMut<NextState<ClientState>>,
    client: Res<Client>,
    users: Res<Users>,
) {
    while let Ok(message) = terminal_messages.try_recv() {
        if message == "/quit" {
            client_state.set(ClientState::Menu);
            // app_exit_events.send(AppExit); // change this to set a state
        } else if message == "/list" {
            println!("{} online", &users.names.len());
            for (c_id, name) in &users.names {
                println!(
                    "{}{}",
                    name,
                    if c_id == &users.self_id { " (you)" } else { "" }
                );
            }
        } else {
            client
                .connection()
                .try_send_message(ClientMessage::ChatMessage { message });
        }
    }
}

fn handle_menu_input(
    mut commands: Commands,
    mut terminal_messages: ResMut<TerminalReceiver>,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    if let Ok(message) = terminal_messages.try_recv() {
        if message.is_empty() {
            client_state.set(ClientState::HostingServer);
            commands.insert_resource(ClientConnectionConfig(
                ConnectionConfiguration::from_strings("127.0.0.1:6006", "0.0.0.0:0").unwrap(),
            ));
        } else {
            match ConnectionConfiguration::from_strings(&message, "0.0.0.0:0") {
                Ok(connection) => {
                    client_state.set(ClientState::ClientToServer);
                    commands.insert_resource(ClientConnectionConfig(connection));
                }
                Err(e) => {
                    error!("{} not a valid server address: {}", message, e);
                }
            }
        }
    }
    // drain messages? since only the first one was taken?
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

fn start_connection(mut client: ResMut<Client>, connection_config: Res<ClientConnectionConfig>) {
    client
        .open_connection(
            connection_config.0.clone(),
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

        println!("Joining with name: {}", username);
        println!("Type '/quit' to disconnect");

        client
            .connection()
            .send_message(ClientMessage::Join { name: username })
            .unwrap();

        connection_events.clear();
    }
}

pub(crate) struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // stuff that should always happen
        app.add_plugins((
            QuinnetClientPlugin::default(),
            QuinnetServerPlugin::default(),
        ));
        app.add_state::<ClientState>();
        app.add_event::<SetHostEvent>();

        // stuff that should happen for hosting?
        app.init_resource::<Users>();
        app.init_resource::<server::Users>(); // this sucks

        app.add_systems(Startup, start_terminal_listener);

        // menu systems
        app.add_systems(OnEnter(ClientState::Menu), prompt);
        app.add_systems(
            Update,
            handle_menu_input.run_if(in_state(ClientState::Menu)),
        );
        app.add_systems(
            OnExit(ClientState::Menu),
            (
                (server::start_listening, server::handle_set_host)
                    .run_if(in_state(ClientState::HostingServer)),
                start_connection.after(server::start_listening),
            ),
        );

        // connected or hosting systems
        app.add_systems(
            Update,
            (
                handle_terminal_messages,
                handle_server_messages,
                handle_client_events,
            )
                .run_if(
                    in_state(ClientState::ClientToServer)
                        .or_else(in_state(ClientState::HostingServer)),
                ),
        );
        app.add_systems(OnExit(ClientState::ClientToServer), leave_server);

        // hosting systems
        app.add_systems(
            Update,
            (server::handle_client_messages, server::handle_server_events)
                .run_if(in_state(ClientState::HostingServer)),
        );
        app.add_systems(OnExit(ClientState::HostingServer), server::end_server);
    }
}
