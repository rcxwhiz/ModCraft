use bevy::prelude::*;

mod protocol;
mod server;

#[cfg(not(feature = "dedicated-server"))]
mod client;

use modcraft_lib::add;

fn main() {
    #[cfg(feature = "dedicated-server")]
    info!("Compiled as dedicated server");

    #[cfg(not(feature = "dedicated-server"))]
    info!("Compiled as client");

    info!("Modcraft lib is working: {} + {} = {}", 2, 2, add(2, 2));

    #[cfg(feature = "dedicated-server")]
    server::server_main();

    #[cfg(not(feature = "dedicated-server"))]
    client::client_main();
}
