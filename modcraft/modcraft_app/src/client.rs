use std::{collections::HashMap, thread};

use bevy::{prelude::*, log::LogPlugin};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode,
        connection::{ConnectionConfiguration, ConnectionEvent, ConnectionId},
        Client, QuinnetClientPlugin,
    },
    shared::ClientId,
};
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::mpsc;

use crate::{
    protocol::{ClientMessage, ServerMessage},
    server::{self, InternalServerState},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum ClientState {
    #[default]
    Menu,
    LaunchingInternalServer,
    ConnectingToServer,
    InGame,
}

// TODO!! clear this after leaving server!
#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    names: HashMap<ClientId, String>,
}

#[derive(Resource, Deref, DerefMut)]
struct TerminalReceiver(mpsc::Receiver<String>);

#[derive(Resource)]
struct ClientConnectionConfig(ConnectionConfiguration);

#[derive(Resource)]
struct ClientConnectionId(ConnectionId);

fn prompt() {
    println!("Enter an address and port to connect to. Enter blank to self host.")
}

fn announce_leave_server(client: ResMut<Client>) {
    info!("Announcing leaving server!");

    client
        .connection()
        .send_message(ClientMessage::Disconnect {})
        .expect("Client failed to send disconnect server message");
}

fn close_server_connection(
    mut commands: Commands,
    mut client: ResMut<Client>,
    connection_id: Res<ClientConnectionId>,
) {
    info!("Closing server connection");

    client
        .close_connection(connection_id.0)
        .expect("Error closing client connection to server");
    commands.remove_resource::<ClientConnectionId>();
    commands.remove_resource::<ClientConnectionConfig>();
}

fn handle_server_messages(
    mut users: ResMut<Users>,
    mut client: ResMut<Client>,
    mut next_client_state: ResMut<NextState<ClientState>>,
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
                next_client_state.set(ClientState::Menu);
            }
        }
    }
}

fn check_internal_server_ready(
    internal_server_state: Res<State<server::InternalServerState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
) {
    if let server::InternalServerState::Running = **internal_server_state {
        info!("Internal server is ready, beginning to connect!");

        next_client_state.set(ClientState::ConnectingToServer)
    }
}

fn check_if_connected(
    mut connection_events: EventReader<ConnectionEvent>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    client: ResMut<Client>,
) {
    if !connection_events.is_empty() {
        info!("Got a connection event!");

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
            .expect("Could not send join message to server");

        connection_events.clear();

        next_client_state.set(ClientState::InGame);
    }
}

fn handle_menu_input(
    mut commands: Commands,
    mut next_client_state: ResMut<NextState<ClientState>>,
    mut next_internal_server_state: ResMut<NextState<server::InternalServerState>>,
    message: String,
) {
    info!("Handling a menu input: {}", &message);

    if message.is_empty() {
        info!("Launching internal server!");

        next_client_state.set(ClientState::LaunchingInternalServer);
        commands.insert_resource(ClientConnectionConfig(
            ConnectionConfiguration::from_strings("127.0.0.1:6006", "0.0.0.0:0")
                .expect("The localhost connection config failed"),
        ));
        next_internal_server_state.set(server::InternalServerState::Launching);
    } else {
        match ConnectionConfiguration::from_strings(&message, "0.0.0.0:0") {
            Ok(connection) => {
                info!("Connecting to {}", &message);

                next_client_state.set(ClientState::ConnectingToServer);
                commands.insert_resource(ClientConnectionConfig(connection));
            }
            Err(e) => {
                error!("{} not a valid server address: {}", message, e);
            }
        }
    }
}

fn handle_game_input(
    mut next_client_state: ResMut<NextState<ClientState>>,
    internal_server_state: Res<State<server::InternalServerState>>,
    mut next_internal_server_state: ResMut<NextState<server::InternalServerState>>,
    users: Res<Users>,
    client: ResMut<Client>,
    message: String,
) {
    info!("Handling a game input: {}", &message);

    if message == "/quit" {
        announce_leave_server(client);
        next_client_state.set(ClientState::Menu);
        // this guard isn't really necessary since it would be off otherwise..?
        if let server::InternalServerState::Running = **internal_server_state {
            next_internal_server_state.set(server::InternalServerState::Off);
        }
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
            .send_message(ClientMessage::ChatMessage { message })
            .expect("Failed to send chat message to server");
    }
}

fn handle_terminal_messages(
    commands: Commands,
    mut terminal_messages: ResMut<TerminalReceiver>,
    next_client_state: ResMut<NextState<ClientState>>,
    next_internal_server_state: ResMut<NextState<InternalServerState>>,
    client_state: Res<State<ClientState>>,
    internal_server_state: Res<State<InternalServerState>>,
    client: ResMut<Client>,
    users: Res<Users>,
) {
    if let Ok(message) = terminal_messages.try_recv() {
        info!("Got a terminal message!");

        match client_state.get() {
            ClientState::Menu => handle_menu_input(
                commands,
                next_client_state,
                next_internal_server_state,
                message,
            ),
            ClientState::InGame => handle_game_input(
                next_client_state,
                internal_server_state,
                next_internal_server_state,
                users,
                client,
                message,
            ),
            _ => warn!("Not in a state to accept messages, disregarding"),
        }
    }
}

fn start_terminal_listener(mut commands: Commands) {
    info!("Starting terminal listener!");

    let (from_terminal_sender, from_terminal_receiver) = mpsc::channel::<String>(100);

    thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read a line from stdin");
        from_terminal_sender
            .try_send(buffer.trim_end().to_string())
            .expect("Failed to send input buffer to terminal sender?");
    });

    commands.insert_resource(TerminalReceiver(from_terminal_receiver));
}

fn start_connection(
    mut commands: Commands,
    mut client: ResMut<Client>,
    connection_config: Res<ClientConnectionConfig>,
) {
    info!("Opening connection to server!");

    let (connection_id, _) = client
        .open_connection(
            connection_config.0.clone(),
            CertificateVerificationMode::SkipVerification,
        )
        .expect("Could not open client connection to server");
    commands.insert_resource(ClientConnectionId(connection_id));
}

pub(crate) struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // library plugins
        app.add_plugins((MinimalPlugins, LogPlugin::default(), QuinnetClientPlugin::default()));
        // crate plugins
        app.add_plugins(server::InternalServerPlugin);

        // add states and events
        app.add_state::<ClientState>();
        app.init_resource::<Users>();

        // input systems
        app.add_systems(Startup, start_terminal_listener);
        app.add_systems(Update, handle_terminal_messages);

        // menu systems
        app.add_systems(OnEnter(ClientState::Menu), prompt);

        // hosting systems
        app.add_systems(
            Update,
            check_internal_server_ready.run_if(in_state(ClientState::LaunchingInternalServer)),
        );

        // connecting systems
        app.add_systems(OnEnter(ClientState::ConnectingToServer), start_connection);
        app.add_systems(
            Update,
            check_if_connected.run_if(in_state(ClientState::ConnectingToServer)),
        );

        // game systems
        app.add_systems(
            Update,
            handle_server_messages.run_if(in_state(ClientState::InGame)),
        );
        app.add_systems(OnExit(ClientState::InGame), close_server_connection);
    }
}
