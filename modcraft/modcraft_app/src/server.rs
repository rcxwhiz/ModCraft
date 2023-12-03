use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfiguration,
    },
    shared::{channel::ChannelId, ClientId},
};

use crate::{
    internal_server::{ClientLeftFlagPlugin, ServerReadyFlagPlugin},
    protocol::{ClientMessage, ServerMessage},
};

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    names: HashMap<ClientId, String>,
}

pub(crate) fn handle_client_messages(mut server: ResMut<Server>, mut users: ResMut<Users>) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some(message) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            match message {
                ClientMessage::Join { name } => {
                    if users.names.contains_key(&client_id) {
                        warn!(
                            "Received a Join from an already connected client: {}",
                            client_id
                        );
                    } else {
                        info!("{} connected", name);
                        users.names.insert(client_id, name.clone());
                        endpoint
                            .send_message(
                                client_id,
                                ServerMessage::InitClient {
                                    client_id,
                                    usernames: users.names.clone(),
                                },
                            )
                            .expect("Failed to send init client message to new client");
                        endpoint
                            .send_group_message(
                                users.names.keys().into_iter(),
                                ServerMessage::ClientConnected {
                                    client_id,
                                    username: name,
                                },
                            )
                            .expect("Failed to send client connected message to clients");
                    }
                }
                ClientMessage::Disconnect {} => {
                    // add something to disconnect clients if host quits
                    endpoint.disconnect_client(client_id).unwrap();
                    handle_disconnect(endpoint, &mut users, client_id);
                }
                ClientMessage::ChatMessage { message } => {
                    info!(
                        "Chat message | {:?}: {}",
                        users.names.get(&client_id),
                        message
                    );
                    endpoint.send_group_message_on(
                        users.names.keys().into_iter(),
                        ChannelId::UnorderedReliable,
                        ServerMessage::ChatMessage { client_id, message },
                    ).expect("Failed to send group message with chat");
                }
            }
        }
    }
}

fn handle_server_events(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut users: ResMut<Users>,
) {
    for client in connection_lost_events.read() {
        handle_disconnect(server.endpoint_mut(), &mut users, client.id);
    }
}

fn handle_disconnect(endpoint: &mut Endpoint, users: &mut ResMut<Users>, client_id: ClientId) {
    if let Some(username) = users.names.remove(&client_id) {
        endpoint
            .send_group_message(
                users.names.keys().into_iter(),
                ServerMessage::ClientDisconnected { client_id },
            )
            .expect("Failed to send user disconnected group message");
        info!("{} disconnected", username);
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        );
    }
}

fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfiguration::from_string("0.0.0.0:6006").unwrap(),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "127.0.0.1".to_string(),
            },
        )
        .expect("Server failed to start endpoint");
}

fn on_server_exit(
    app_exit_events: EventReader<AppExit>,
    mut server: ResMut<Server>,
    users: Res<Users>,
) {
    if !app_exit_events.is_empty() {
        let endpoint = server.endpoint();
        endpoint
            .send_group_message(
                users.names.keys().into_iter(),
                ServerMessage::ServerStopping {},
            )
            .expect("Server failed to send group message that it is stopping");
        server
            .stop_endpoint()
            .expect("Server failed to stop its endpoint");
    }
}

struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuinnetServerPlugin::default())
            .init_resource::<Users>()
            .add_systems(Startup, start_listening)
            .add_systems(FixedUpdate, (handle_client_messages, handle_server_events))
            .add_systems(PostUpdate, on_server_exit);
    }
}

pub fn server_main() {
    info!("This is the server main function");

    start_server(None, None);
}

pub fn internal_server_main(
    client_left_flag: Arc<Mutex<bool>>,
    server_ready_flag: Arc<Mutex<bool>>,
) {
    info!("This is the internal server main function");

    start_server(Some(client_left_flag), Some(server_ready_flag));
}

fn start_server(
    client_left_flag: Option<Arc<Mutex<bool>>>,
    server_ready_flag: Option<Arc<Mutex<bool>>>,
) {
    info!("This is the function that starts the server");

    let mut app = App::new();
    app.add_plugins(ServerPlugin);

    if let Some(client_left_flag) = client_left_flag {
        app.add_plugins(ClientLeftFlagPlugin::new(client_left_flag));
    }

    if let Some(server_ready_flag) = server_ready_flag {
        app.add_plugins(ServerReadyFlagPlugin::new(server_ready_flag));
    }

    // TODO there will probably be another flag later for integrated servers allowing people to join

    app.run();
}
