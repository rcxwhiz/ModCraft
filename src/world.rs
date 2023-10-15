use crate::chunk::Chunk;
use bevy::{prelude::*, utils::HashMap};

#[derive(Resource)]
pub struct World {
    pub chunks: HashMap<(i64, i64), Chunk>,
}

impl Default for World {
    fn default() -> Self {
        let mut chunks = HashMap::new();
        chunks.insert((0, 0), Chunk::default());
        Self { chunks }
    }
}
