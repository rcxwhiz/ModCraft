use std::collections::HashMap;

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
        endpoint
            .send_group_message(
                users.names.keys().into_iter(),
                ServerMessage::ClientDisconnected { client_id },
            )
            .unwrap();
        info!("{} disconnected", username);
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

pub(crate) fn setup_app(app: &mut App) {
    app.add_plugins(QuinnetServerPlugin::default())
        .init_resource::<Users>()
        .add_systems(Startup, start_listening)
        .add_systems(Update, (handle_client_messages, handle_server_events));
}
