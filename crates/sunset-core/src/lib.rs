#![forbid(unsafe_code)]

pub mod arithmetic;
pub mod block;
pub mod block_state;
pub mod coords;
pub mod mesh;
pub mod palette;
pub mod terrain;

pub use arithmetic::{floor_div, floor_mod};
pub use block_state::BlockState;
pub use coords::{BlockPos, ChunkPos, SectionPos};
pub use mesh::{
    mesh_section, mesh_section_greedy, Face, Mesh, SectionNeighbors, Vertex, VERTEX_SIZE_BYTES,
};
pub use palette::{PalettedSection, SECTION_SIZE};
pub use terrain::{TerrainGenerator, TerrainParams};
