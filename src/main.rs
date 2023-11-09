use bevy::prelude::*;

mod block;
mod chunk;
mod world_dimension;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_camera)
        .add_plugins(world_dimension::WorldDimensionPlugin)
        .run();
}

fn setup_camera(
    mut commands: Commands,
) {
    // spawn camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-20.0, 0.0, 10.0).looking_at(Vec3::X, Vec3::Y),
        ..Default::default()
    });
}
