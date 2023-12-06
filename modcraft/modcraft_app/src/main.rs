use bevy::prelude::App;

mod protocol;
mod server;

#[cfg(not(feature = "dedicated-server"))]
mod client;
#[cfg(not(feature = "dedicated-server"))]
mod internal_server;

use modcraft_lib::add;

fn main() {
    #[cfg(feature = "dedicated-server")]
    println!("Compiled as dedicated server");

    #[cfg(not(feature = "dedicated-server"))]
    println!("Compiled as client");

    println!("Modcraft lib is working: {} + {} = {}", 2, 2, add(2, 2));

    let mut app = App::new();

    #[cfg(feature = "dedicated-server")]
    app.add_plugins(server::DedicatedServerPlugin);

    #[cfg(not(feature = "dedicated-server"))]
    app.add_plugins(client::ClientPlugin);

    app.run();
}
