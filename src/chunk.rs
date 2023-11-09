use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use crate::block::Block;

const CHUNK_WIDTH: u32 = 16;
const CHUNK_HEIGHT: u32 = 256;
const CHUNK_AREA: u32 = CHUNK_WIDTH * CHUNK_WIDTH;
const CHUNK_VOLUME: u32 = CHUNK_AREA * CHUNK_HEIGHT;

#[derive(Resource)]
struct Chunk {
    blocks: [Block; CHUNK_VOLUME as usize],
}

fn spawn_blocks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk: Res<Chunk>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let cube_mesh = meshes.add(shape::Cube::default().into());

    for x in 0..CHUNK_WIDTH {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_WIDTH {
                match chunk.blocks[x as usize + (y * CHUNK_AREA) as usize + (z * CHUNK_WIDTH) as usize] {
                    Block::Air => println!("hello :("),
                    _ => {
                        commands.spawn(PbrBundle {
                            mesh: cube_mesh.clone(),
                            transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                            material: debug_material.clone(),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}

fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

impl Default for Chunk {
    fn default() -> Self {
        let mut chunk = Chunk {
            blocks: [Block::Air; CHUNK_VOLUME as usize],
        };

        for x in 0..CHUNK_WIDTH {
            for y in 0..32 {
                for z in 0..CHUNK_WIDTH {
                    // let &mut block = chunk.block(x as usize, y as usize, z as usize);
                    chunk.blocks[x as usize + (y * CHUNK_AREA) as usize + (z * CHUNK_WIDTH) as usize] = Block::Dirt;
                }
            }
        }

        chunk
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        // add a chunk resource
        app.init_resource::<Chunk>();
        app.add_systems(Startup, spawn_blocks);
    }
}
