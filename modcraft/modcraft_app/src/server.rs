use std::collections::HashMap;

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfiguration,
    },
    shared::{channel::ChannelId, ClientId},
};

use crate::protocol::{ClientMessage, ServerMessage};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub(crate) enum InternalServerState {
    #[default]
    Off,
    Launching,
    Running,
}

#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct Users {
    names: HashMap<ClientId, String>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum ServerSystems {
    Startup,
    FixedUpdate,
    OnExit,
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
                    endpoint
                        .send_group_message_on(
                            users.names.keys().into_iter(),
                            ChannelId::UnorderedReliable,
                            ServerMessage::ChatMessage { client_id, message },
                        )
                        .expect("Failed to send group message with chat");
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
    info!("Starting endpoint!");

    server
        .start_endpoint(
            ServerConfiguration::from_string("0.0.0.0:6006").unwrap(),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "127.0.0.1".to_string(),
            },
        )
        .expect("Server failed to start endpoint");
}

fn handle_app_exit(
    app_exit_events: EventReader<AppExit>,
    server: ResMut<Server>,
    users: Res<Users>,
) {
    if !app_exit_events.is_empty() {
        on_server_exit(server, users);
    }
}

fn on_server_exit(mut server: ResMut<Server>, users: Res<Users>) {
    info!("Server exiting!");

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

fn set_internal_server_ready(
    mut next_internal_server_state: ResMut<NextState<InternalServerState>>,
) {
    next_internal_server_state.set(InternalServerState::Running);
}

fn clear_users(mut users: ResMut<Users>) {
    (*users).names.clear();
}

pub(crate) struct InternalServerPlugin;
impl Plugin for InternalServerPlugin {
    fn build(&self, app: &mut App) {
        // TODO these need to be centralized
        let startup_systems = (start_listening,);
        let fixed_update_systems = (handle_client_messages, handle_server_events);
        let exit_systems = (on_server_exit, clear_users);

        app.add_plugins(QuinnetServerPlugin::default())
            .add_state::<InternalServerState>()
            .init_resource::<Users>()
            .add_systems(
                OnEnter(InternalServerState::Launching),
                startup_systems.in_set(ServerSystems::Startup),
            )
            .add_systems(
                OnEnter(InternalServerState::Launching),
                set_internal_server_ready.after(ServerSystems::Startup),
            )
            .add_systems(
                FixedUpdate,
                fixed_update_systems
                    .run_if(in_state(InternalServerState::Running))
                    .in_set(ServerSystems::FixedUpdate),
            )
            .add_systems(
                OnExit(InternalServerState::Running),
                exit_systems.in_set(ServerSystems::OnExit),
            ); // how does this work?
    }
}

pub(crate) struct DedicatedServerPlugin;
impl Plugin for DedicatedServerPlugin {
    fn build(&self, app: &mut App) {
        let startup_systems = (start_listening,);
        let fixed_update_systems = (handle_client_messages, handle_server_events);
        let post_update_systems = (handle_app_exit,);

        app.add_plugins((MinimalPlugins, QuinnetServerPlugin::default()))
            .init_resource::<Users>()
            .add_systems(Startup, startup_systems)
            .add_systems(FixedUpdate, fixed_update_systems)
            .add_systems(PostUpdate, post_update_systems);
    }
}
