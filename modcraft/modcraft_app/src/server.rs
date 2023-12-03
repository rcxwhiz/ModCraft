use std::{collections::HashMap, sync::{Arc, Mutex}};

use bevy::prelude::*;
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfiguration,
    },
    shared::{channel::ChannelId, ClientId},
};

use crate::protocol::{ClientMessage, ServerMessage};

#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct Users {
    pub host: Option<ClientId>,
    names: HashMap<ClientId, String>,
}

#[derive(Event)]
pub(crate) struct SetHostEvent(ClientId);

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
                            .unwrap();
                        endpoint
                            .send_group_message(
                                users.names.keys().into_iter(),
                                ServerMessage::ClientConnected {
                                    client_id,
                                    username: name,
                                },
                            )
                            .unwrap();
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
                    endpoint.try_send_group_message_on(
                        users.names.keys().into_iter(),
                        ChannelId::UnorderedReliable,
                        ServerMessage::ChatMessage { client_id, message },
                    );
                }
            }
        }
    }
}

pub(crate) fn handle_server_events(
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
        if users.host == Some(client_id) {
            endpoint
                .send_group_message(
                    users.names.keys().into_iter(),
                    ServerMessage::ServerStopping,
                )
                .unwrap();
            endpoint.disconnect_all_clients().unwrap();
            info!("Disconnected all users")
        } else {
            endpoint
                .send_group_message(
                    users.names.keys().into_iter(),
                    ServerMessage::ClientDisconnected { client_id },
                )
                .unwrap();
            info!("{} disconnected", username);
        }
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        );
    }
}

pub(crate) fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfiguration::from_string("0.0.0.0:6006").unwrap(),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "127.0.0.1".to_string(),
            },
        )
        .unwrap();
}

pub(crate) fn handle_set_host(
    mut ev_set_host: EventReader<SetHostEvent>,
    mut users: ResMut<Users>,
) {
    for ev in ev_set_host.read() {
        users.host = Some(ev.0);
    }
}

pub(crate) fn end_server(mut server: ResMut<Server>, users: Res<Users>) {
    let endpoint = server.endpoint();
    endpoint
        .send_group_message(
            users.names.keys().into_iter(),
            ServerMessage::ServerStopping {},
        )
        .unwrap();
    server.stop_endpoint().unwrap();
}

pub(crate) struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuinnetServerPlugin::default())
            .init_resource::<Users>()
            .add_systems(Startup, start_listening)
            .add_systems(Update, (handle_client_messages, handle_server_events));
    }
}

type Flag = Arc<Mutex<bool>>;
#[derive(Resource)]
struct ServerOnlineFlag {
    flag: Flag,
}
impl ServerOnlineFlag {
    fn new(flag: Flag) -> Self {
        Self { flag }
    }

    fn flip_flag(started_flag: Res<ServerOnlineFlag>) {
        *(*started_flag.flag).lock().expect("Server failed to get lock for started flag") = true;
    }
}
impl Plugin for ServerOnlineFlag {
    fn build(&self, app: &mut App) {
        app.insert_resource(Self::new(Arc::clone(&self.flag)));
        app.add_systems(PostStartup, Self::flip_flag);
    }
}

// need to make different dedicated and internal server plugins here
// also probably need to get server on a lower tick rate just to
// demonstrate a divide between client and server updates

pub fn server_main() {
    info!("This is the server main function");

    start_server(None, None);
}

pub fn internal_server_main(stop_flag: Arc<Mutex<bool>>, online_flag: Arc<Mutex<bool>>) {
    info!("This is the internal server main function");

    start_server(Some(stop_flag), Some(online_flag));
}

fn start_server(stop_flag: Option<Arc<Mutex<bool>>>, online_flag: Option<Arc<Mutex<bool>>>) {
    info!("This is the function that starts the server");

    let mut app = App::new();
    app.add_plugins(ServerPlugin);

    if let Some(online_flag) = online_flag {
        app.add_plugins(ServerOnlineFlag::new(online_flag));
    }

    app.run();
}
