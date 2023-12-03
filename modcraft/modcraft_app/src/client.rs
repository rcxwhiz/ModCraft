use std::{collections::HashMap, thread};

use bevy::prelude::*;
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode,
        connection::{ConnectionConfiguration, ConnectionEvent, ConnectionId},
        Client, QuinnetClientPlugin,
    },
    server::QuinnetServerPlugin,
    shared::ClientId,
};
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::mpsc;

use crate::{
    protocol::{ClientMessage, ServerMessage},
    server,
};

#[derive(Event)]
struct CloseClientConnectionEvent;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum ClientState {
    #[default]
    Menu,
    ClientToServer,
    HostingServer,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum GameSystems {
    ServerSystems,
    ClientSystems,
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

fn announce_leave_server(
    client: ResMut<Client>,
    mut close_connection_events: EventWriter<CloseClientConnectionEvent>,
) {
    client
        .connection()
        .send_message(ClientMessage::Disconnect {})
        .expect("Client failed to send disconnect server message");
    close_connection_events.send(CloseClientConnectionEvent);
}

fn close_server_connection(
    mut close_connection_events: EventReader<CloseClientConnectionEvent>,
    mut commands: Commands,
    mut client: ResMut<Client>,
    connection_id: Res<ClientConnectionId>,
) {
    if !close_connection_events.is_empty() {
        client
            .close_connection(connection_id.0)
            .expect("Error closing client connection to server");
        commands.remove_resource::<ClientConnectionId>();
        close_connection_events.clear();
    }
}

fn handle_server_messages(
    mut users: ResMut<Users>,
    mut client: ResMut<Client>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut close_connection_events: EventWriter<CloseClientConnectionEvent>,
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
                close_connection_events.send(CloseClientConnectionEvent);
                client_state.set(ClientState::Menu);
            }
        }
    }
}

fn handle_terminal_messages(
    mut commands: Commands,
    mut terminal_messages: ResMut<TerminalReceiver>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    client_state: Res<State<ClientState>>,
    client: Res<Client>,
    users: Res<Users>,
) {
    if let Ok(message) = terminal_messages.try_recv() {
        match client_state.get() {
            ClientState::Menu => {
                if message.is_empty() {
                    next_client_state.set(ClientState::HostingServer);
                    commands.insert_resource(ClientConnectionConfig(
                        ConnectionConfiguration::from_strings("127.0.0.1:6006", "0.0.0.0:0")
                            .expect("Failed to make a server connection configuration"),
                    ));
                } else {
                    match ConnectionConfiguration::from_strings(&message, "0.0.0.0:0") {
                        Ok(connection) => {
                            next_client_state.set(ClientState::ClientToServer);
                            commands.insert_resource(ClientConnectionConfig(connection));
                        }
                        Err(e) => {
                            error!("{} not a valid server address: {}", message, e);
                        }
                    }
                }
            }
            _ => {
                if message == "/quit" {
                    next_client_state.set(ClientState::Menu);
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
                    // todo change this to some .expect
                }
            }
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
            .expect("Could not send join message to server");

        connection_events.clear();
    }
}

pub(crate) struct ClientPlugin;
// All of this stuff should be changed to be as in the server file as possible
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // quinnet library plugins
        app.add_plugins((
            QuinnetClientPlugin::default(),
            QuinnetServerPlugin::default(),
        ));

        // add states and events
        app.add_event::<CloseClientConnectionEvent>();
        app.add_state::<ClientState>();
        app.add_event::<server::SetHostEvent>();

        // stuff that should happen for hosting?
        app.init_resource::<Users>();
        app.init_resource::<server::Users>(); // this sucks

        // input systems
        app.add_systems(Startup, start_terminal_listener);
        app.add_systems(Update, handle_terminal_messages);

        // menu systems
        app.add_systems(OnEnter(ClientState::Menu), prompt);

        // start as client systems
        app.add_systems(
            OnEnter(ClientState::ClientToServer),
            (start_connection,).in_set(GameSystems::ClientSystems),
        );

        // close client connection handler
        app.add_systems(
            Update,
            close_server_connection.run_if(resource_exists::<ClientConnectionId>()),
        );

        // start as host systems
        // -- client stuff
        app.edit_schedule(OnEnter(ClientState::HostingServer), |schedule| {
            schedule.configure_sets(GameSystems::ClientSystems.after(GameSystems::ServerSystems));
            schedule.add_systems((start_connection,).in_set(GameSystems::ClientSystems));
        });
        // -- server stuff
        app.add_systems(
            OnEnter(ClientState::HostingServer),
            (server::start_listening, server::handle_set_host).in_set(GameSystems::ServerSystems),
        );

        // client update systems
        app.edit_schedule(Update, |schedule| {
            schedule.configure_sets(GameSystems::ClientSystems.run_if(
                in_state(ClientState::ClientToServer).or_else(in_state(ClientState::HostingServer)),
            ));
            schedule.add_systems(
                (
                    handle_terminal_messages,
                    handle_server_messages,
                    handle_client_events,
                )
                    .in_set(GameSystems::ClientSystems),
            );
        });
        // client exit systems
        app.add_systems(
            OnExit(ClientState::ClientToServer),
            (announce_leave_server).in_set(GameSystems::ClientSystems),
        );

        // server update systems
        app.edit_schedule(Update, |schedule| {
            schedule.configure_sets(
                GameSystems::ServerSystems.run_if(in_state(ClientState::HostingServer)),
            );
            schedule.add_systems(
                (server::handle_client_messages, server::handle_server_events)
                    .in_set(GameSystems::ServerSystems),
            );
        });
        // server exit systems
        app.add_systems(
            OnExit(ClientState::HostingServer),
            (server::end_server, close_server_connection).in_set(GameSystems::ServerSystems),
        );
    }
}

pub fn client_main() {
    info!("This is the client main function");

    let mut app = App::new();
    app.add_plugins(ClientPlugin);
    app.run();
}
