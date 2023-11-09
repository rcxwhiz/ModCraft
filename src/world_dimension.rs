use bevy::{prelude::*, utils::HashMap};
use crate::chunk;

#[derive(Bundle)]
struct DimensionBundle {
    marker: Dimension,
    name: Name,
    chunks: Chunks,
}

#[derive(Component)]
struct Dimension;

#[derive(Component)]
struct Name(&'static str);

#[derive(Component)]
struct Chunks(HashMap<(i64, i64), chunk::Chunk>);

impl Chunks {
    fn create_chunk(&mut self, x: i64, z: i64) -> bool {
        let chunk = self.0.get(&(x, z));
        if let Some(_chunk) = chunk {
            return false
        } else {
            self.0.insert((x, z), chunk::Chunk::new(x, z));
            return true
        }
    }
}

static OVERWORLD_NAME: &str = "overworld";

pub struct WorldDimensionPlugin;

impl Plugin for WorldDimensionPlugin {
    fn build(&self, app: &mut App) {
        info!("Running world dimension plugin");
        app.add_systems(Startup, spawn_overworld);
        info!("Added spawn overworld system");
        app.add_systems(Update, render_overworld_chunk);
        info!("Added render overworld chunk system");
    }
}

fn spawn_overworld(
    mut commands: Commands,
) {
    // create a chunks grid with one chunk
    let mut chunks = Chunks(HashMap::new());
    chunks.create_chunk(0, 0);
    info!("Created chunks component with chunk");

    // spawn the dimension
    commands.spawn(DimensionBundle {
        marker: Dimension,
        name: Name(OVERWORLD_NAME),
        chunks,
    });
    info!("Spawned overworld bundle");
}

fn render_overworld_chunk(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    images: ResMut<Assets<Image>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Name, &mut Chunks), With<Dimension>>,
) {
    info!("Looking for overworld dimension to render (in {} results)", query.iter().count());
    for (name, mut chunks) in query.iter_mut() {
        if name.0 == OVERWORLD_NAME {
            info!("Found overworld dimension");
            commands.spawn(chunks.0.get_mut(&(0, 0)).unwrap().get_pbr_bundle(meshes, images, materials));
            info!("Rendered overworld chunk");
            return;
        }
    }
}
