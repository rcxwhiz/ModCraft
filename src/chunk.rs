use bevy::{
    prelude::*,
    math::{I64Vec2, I64Vec3},
};
use crate::{block::{Block, BlockType}, debug::{CubeMesh, DebugMaterial}};

// #[derive(Resource)] // TODO this should be a component now?
pub struct Chunk {
    location: I64Vec2,
    blocks: [Block; Self::VOLUME as usize],
    mesh: Option<Handle<Mesh>>,
    pbr_entity: Option<Entity>,
}

impl Chunk {
    pub const WIDTH: usize = 16;
    pub const HEIGHT: usize = 256;
    pub const AREA: usize = Self::WIDTH * Self::WIDTH;
    pub const VOLUME: usize = Self::AREA * Self::HEIGHT;

    pub fn new(location: I64Vec2) -> Self {
        // create new chunk at location filled with air blocks
        let mut chunk = Chunk {
            location,
            blocks: [Block::default(); Self::VOLUME],
            mesh: None,
            pbr_entity: None,
        };

        // loop through blocks
        for x in 0..Self::WIDTH {
            for y in 0..Self::HEIGHT {
                for z in 0..Self::WIDTH {
                    let block = chunk.blocks.get_mut(x + y * Self::AREA + z * Self::WIDTH).unwrap();
                    // correct the position of the blocks
                    block.position = I64Vec3::new(chunk.location.x * Self::WIDTH as i64 + x as i64, y as i64, chunk.location.y * Self::WIDTH as i64 + z as i64);
                    // set lower blocks to stone and dirt
                    if y < 32 {
                        block.b_type = BlockType::Stone;
                    } else if y < 64 {
                        block.b_type = BlockType::Dirt
                    }
                }
            }
        }

        chunk
    }

    pub fn set_visibility(
        &mut self,
        visible: bool,
        mut commands: Commands,
        cube_mesh: Res<CubeMesh>,
        debug_material: Res<DebugMaterial>,
        mut ev_hide_chunk: EventWriter<HideChunkEvent>,
        mut ev_show_chunk: EventWriter<ShowChunkEvent>,
    ) {
        if visible {
            if let Some(pbr_entity) = self.pbr_entity {
                ev_show_chunk.send(ShowChunkEvent(pbr_entity));
            } else {
                // spawn the entity and save it
                self.pbr_entity = Some(commands.spawn(PbrBundle {
                    mesh: cube_mesh.0.clone(),
                    transform: Transform::from_xyz(0., 0., 10.), // TODO this won't be needed when the mesh from the blocks is correct?
                    material: debug_material.0.clone(),
                    ..Default::default()
                }).id());
            }
        } else if let Some(pbr_entity) = self.pbr_entity {
            ev_hide_chunk.send(HideChunkEvent(pbr_entity));
        }
    }

    fn get_mesh(  // TODO need to get blocks to collaborate to make mesh indicies
        &mut self,
        mut meshes: ResMut<Assets<Mesh>>,
    ) -> Handle<Mesh> {
        if self.mesh.is_none() {
            self.mesh = Some(meshes.add(shape::Cube::default().into()));
        }
        self.mesh.clone().unwrap()
    }
}

#[derive(Event)]
pub struct HideChunkEvent(Entity);

#[derive(Event)]
pub struct ShowChunkEvent(Entity);

fn hide_chunk_system(
    mut ev_hide_chunk: EventReader<HideChunkEvent>,
    mut query: Query<&mut Visibility>,
) {
    for ev in ev_hide_chunk.read() {
        info!("Got a hide chunk event");
        if let Ok(mut visibility) = query.get_mut(ev.0) {
            *visibility = Visibility::Hidden;
            info!("Set visibility to hidden");
        }
    }
}

fn show_chunk_system(
    mut ev_show_chunk: EventReader<ShowChunkEvent>,
    mut query: Query<&mut Visibility>,
) {
    for ev in ev_show_chunk.read() {
        info!("Got a show chunk event");
        if let Ok(mut visibility) = query.get_mut(ev.0) {
            *visibility = Visibility::Visible;
            info!("Set visibility to visibile");
        }
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        info!("Registering hide chunk event");
        app.add_event::<HideChunkEvent>();
        info!("Registering show chunk event");
        app.add_event::<ShowChunkEvent>();
        info!("Adding hide and show chunk systems");
        app.add_systems(Update, (hide_chunk_system, show_chunk_system));
    }
}
