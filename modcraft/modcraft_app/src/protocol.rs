use std::collections::HashMap;

use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

// messages from clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { name: String },
    Disconnect {},
    ChatMessage { message: String },
}

// messages from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    ClientConnected {
        client_id: ClientId,
        username: String,
    },
    ClientDisconnected {
        client_id: ClientId,
    },
    ChatMessage {
        client_id: ClientId,
        message: String,
    },
    InitClient {
        client_id: ClientId,
        usernames: HashMap<ClientId, String>,
    },
}
