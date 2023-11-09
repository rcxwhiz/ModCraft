use bevy::{prelude::*, utils::HashMap, math::I64Vec2};
use crate::{chunk::{self, HideChunkEvent, ShowChunkEvent}, debug::{CubeMesh, DebugMaterial}};

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
struct Chunks(HashMap<I64Vec2, chunk::Chunk>);

impl Chunks {
    fn create_chunk(&mut self, location: &I64Vec2) -> bool {
        let chunk = self.0.get(location);
        if let Some(_chunk) = chunk {
            return false
        } else {
            self.0.insert(*location, chunk::Chunk::new(*location));
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
        app.add_systems(PostStartup, render_overworld_chunk);
        info!("Added render overworld chunk system");
    }
}

fn spawn_overworld(
    mut commands: Commands,
) {
    info!("Spawning overworld dimension");
    // create a chunks grid with one chunk
    let mut chunks = Chunks(HashMap::new());
    chunks.create_chunk(&I64Vec2::ZERO);
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
    commands: Commands,
    cube_mesh: Res<CubeMesh>,
    debug_material: Res<DebugMaterial>,
    mut query: Query<(&Name, &mut Chunks), With<Dimension>>,
    ev_hide_chunk: EventWriter<HideChunkEvent>,
    ev_show_chunk: EventWriter<ShowChunkEvent>,
) {
    info!("Looking for overworld dimension to render (in {} results)", query.iter().count());
    for (name, mut chunks) in query.iter_mut() {
        if name.0 == OVERWORLD_NAME {
            info!("Found overworld dimension");
            let chunk = chunks.0.get_mut(&I64Vec2::ZERO).unwrap();
            info!("Found chunk");
            chunk.set_visibility(
                true,
                commands,
                cube_mesh,
                debug_material,
                ev_hide_chunk,
                ev_show_chunk);
            info!("Set visibility on chunk to true");
            return;
        }
    }
}
