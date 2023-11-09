use bevy::{prelude::*, math::I64Vec3};

#[derive(Copy, Clone)]
pub enum BlockType {
    Air,
    Dirt,
    Stone,
}

#[derive(Copy, Clone)]
pub struct Block {
    pub position: I64Vec3,
    pub b_type: BlockType,
}

impl Block {
    pub fn visible(&self) -> bool {
        match self.b_type {
            BlockType::Air => false,
            _ => true,
        }
    }

    pub fn transparent(&self) -> bool {
        match self.b_type {
            BlockType::Air => true,
            _ => false
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            position: I64Vec3::ZERO,
            b_type: BlockType::Air,
        }
    }
}
