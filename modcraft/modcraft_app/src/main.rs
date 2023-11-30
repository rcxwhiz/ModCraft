use bevy::prelude::*;

mod server;

#[cfg(not(feature = "dedicated-server"))]
mod client;

use modcraft_lib::add;

fn main() {
    println!("Welcome to modcraft (now different)");

    #[cfg(not(feature = "dedicated-server"))]
    println!("Compiled as client");

    #[cfg(feature = "dedicated-server")]
    println!("Compiled as dedicated server");

    println!("Modcraft lib is working: {} + {} = {}", 2, 2, add(2, 2));

    let mut app = App::new();

    #[cfg(feature = "dedicated-server")]
    server::setup_app(&mut app);

    #[cfg(not(feature = "dedicated-server"))]
    client::setup_app(&mut app);

    app.run();
}
