use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
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
    internal_server::{ClientLeftFlagResource, ServerReadyFlagResource},
    protocol::{ClientMessage, ServerMessage},
    server,
};

// #[derive(Event)]
// struct CloseClientConnectionEvent;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum ClientState {
    #[default]
    Menu,
    LaunchingInternalServer,
    ConnectingToServer,
    InGame,
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

#[derive(Resource)]
struct ClientConnectionId(ConnectionId);

fn prompt() {
    println!("Enter an address and port to connect to. Enter blank to self host.")
}

fn announce_leave_server(client: ResMut<Client>) {
    client
        .connection()
        .send_message(ClientMessage::Disconnect {})
        .expect("Client failed to send disconnect server message");
}

fn close_server_connection(
    mut commands: Commands,
    mut client: ResMut<Client>,
    connection_id: Res<ClientConnectionId>,
    client_left_flag: Option<ResMut<ClientLeftFlagResource>>,
) {
    client
        .close_connection(connection_id.0)
        .expect("Error closing client connection to server");
    commands.remove_resource::<ClientConnectionId>();

    if let Some(client_left_flag) = client_left_flag {
        *((*client_left_flag)
            .flag
            .lock()
            .expect("Client failed to get lock for client left flag")) = true;
    }

    commands.remove_resource::<ClientLeftFlagResource>();
    commands.remove_resource::<ServerReadyFlagResource>();
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

fn launch_internal_server(mut commands: Commands) {
    let client_left_flag = Arc::new(Mutex::new(false));
    let server_ready_flag = Arc::new(Mutex::new(false));

    commands.insert_resource(ClientLeftFlagResource::new(Arc::clone(&client_left_flag)));
    commands.insert_resource(ServerReadyFlagResource::new(Arc::clone(&server_ready_flag)));

    thread::spawn(move || {
        thread::spawn(move || {
            server::start_internal_server(
                Arc::clone(&client_left_flag),
                Arc::clone(&server_ready_flag),
            );
        })
        .join()
        .expect("Internal server did not stop correctly");
    });
}

fn check_internal_server_ready(
    server_ready_flag: Res<ServerReadyFlagResource>,
    mut next_client_state: ResMut<NextState<ClientState>>,
) {
    if *((*server_ready_flag)
        .flag
        .lock()
        .expect("Client failed to get lock for server ready flag"))
    {
        next_client_state.set(ClientState::ConnectingToServer);
    }
}

fn check_if_connected(
    mut connection_events: EventReader<ConnectionEvent>,
    mut next_client_state: ResMut<NextState<ClientState>>,
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
            .expect("Could not send join message to server");

        connection_events.clear();

        next_client_state.set(ClientState::InGame);
    }
}

fn handle_menu_input(
    mut commands: Commands,
    mut next_client_state: ResMut<NextState<ClientState>>,
    message: String,
) {
    if message.is_empty() {
        next_client_state.set(ClientState::LaunchingInternalServer);
        commands.insert_resource(ClientConnectionConfig(
            ConnectionConfiguration::from_strings("127.0.0.1:6006", "0.0.0.0:0")
                .expect("The localhost connection config failed"),
        ));
    } else {
        match ConnectionConfiguration::from_strings(&message, "0.0.0.0:0") {
            Ok(connection) => {
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
    users: Res<Users>,
    client: ResMut<Client>,
    message: String,
) {
    if message == "/quit" {
        announce_leave_server(client);
        next_client_state.set(ClientState::Menu);
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
    client_state: Res<State<ClientState>>,
    client: ResMut<Client>,
    users: Res<Users>,
) {
    if let Ok(message) = terminal_messages.try_recv() {
        match client_state.get() {
            ClientState::Menu => handle_menu_input(commands, next_client_state, message),
            ClientState::InGame => handle_game_input(next_client_state, users, client, message),
            _ => warn!("Not in a state to accept messages, disregarding"),
        }
    }
}

fn start_terminal_listener(mut commands: Commands) {
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
    let (connection_id, _) = client
        .open_connection(
            connection_config.0.clone(),
            CertificateVerificationMode::SkipVerification,
        )
        .expect("Could not open client connection to server");
    commands.insert_resource(ClientConnectionId(connection_id));
}

struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // quinnet library plugins
        app.add_plugins((
            ScheduleRunnerPlugin::default(),
            LogPlugin::default(),
            QuinnetClientPlugin::default(),
        ));

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
            OnEnter(ClientState::LaunchingInternalServer),
            launch_internal_server,
        );
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

pub fn client_main() {
    App::new().add_plugins(ClientPlugin).run();
}
