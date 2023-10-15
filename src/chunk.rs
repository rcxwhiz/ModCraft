use crate::block::Block;
use bevy::prelude::*;

pub struct Chunk {
    pub blocks: [Block; Self::VOLUME],
}

impl Chunk {
    pub const WIDTH: usize = 16;
    pub const HEIGHT: usize = 256;
    pub const VOLUME: usize = Self::WIDTH * Self::WIDTH * Self::HEIGHT;

    fn flat_index(i: &UVec3) -> usize {
        i.x as usize + (i.y as usize * Self::WIDTH) + (i.z as usize * Self::WIDTH * Self::HEIGHT)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        let mut blocks = [Block::Air; Self::VOLUME];
        for y in 0..64 {
            for x in 0..Self::WIDTH {
                for z in 0..Self::WIDTH {
                    blocks[Self::flat_index(&UVec3::new(x as u32, y, z as u32))] = Block::Dirt;
                }
            }
        }

        Self { blocks }
    }
}
