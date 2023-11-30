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
    server::QuinnetServerPlugin,
    shared::ClientId,
};
use rand::{distributions::Alphanumeric, Rng};
use tokio::sync::mpsc;

use crate::{
    protocol::{ClientMessage, ServerMessage},
    server::{self, SetHostEvent},
};

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    names: HashMap<ClientId, String>,
}

#[derive(Resource, Deref, DerefMut)]
struct TerminalReceiver(mpsc::Receiver<String>);

#[derive(Resource)]
struct ServerAddress(Option<String>);

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, client: Res<Client>) {
    if !app_exit_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        sleep(Duration::from_secs_f32(0.1));
    }
}

fn handle_server_messages(
    mut users: ResMut<Users>,
    mut client: ResMut<Client>,
    mut server_users: ResMut<server::Users>,
    mut app_exit_events: EventWriter<AppExit>,
    server_address: Res<ServerAddress>,
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

                if server_address.0.is_none() {
                    server_users.host = Some(client_id);
                }
            }
            ServerMessage::ServerStopping => {
                info!("Server shutdown!");
                app_exit_events.send(AppExit);
            }
        }
    }
}

fn handle_terminal_messages(
    mut terminal_messages: ResMut<TerminalReceiver>,
    mut app_exit_events: EventWriter<AppExit>,
    client: Res<Client>,
    users: Res<Users>,
) {
    while let Ok(message) = terminal_messages.try_recv() {
        if message == "/quit" {
            app_exit_events.send(AppExit);
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
    let addr = server_address
        .0
        .clone()
        .unwrap_or(String::from("127.0.0.1:6006"));

    client
        .open_connection(
            ConnectionConfiguration::from_strings(&addr, "0.0.0.0:0").unwrap(),
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

pub(crate) fn setup_app(app: &mut App) {
    // client stuff
    app.add_plugins(QuinnetClientPlugin::default());
    app.init_resource::<Users>();
    app.add_event::<SetHostEvent>();

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

    let args: Vec<String> = env::args().collect();
    let server_address = args.get(1);
    app.insert_resource(ServerAddress(server_address.cloned()));

    if server_address.is_none() {
        // there should be a more organized way to do this (plugins?)
        app.add_plugins(QuinnetServerPlugin::default())
            .init_resource::<server::Users>()
            .add_systems(Startup, server::start_listening.before(start_connection))
            .add_systems(PostStartup, server::handle_set_host)
            .add_systems(
                Update,
                (server::handle_client_messages, server::handle_server_events),
            );
    }
}
