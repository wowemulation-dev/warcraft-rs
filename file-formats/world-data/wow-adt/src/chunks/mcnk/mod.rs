//! MCNK terrain chunk and subchunks.
//!
//! MCNK chunks define individual 33.333×33.333 yard terrain tiles within ADT files.
//! Each MCNK contains multiple subchunks for vertex data, normals, textures, etc.

pub mod chunk;
pub mod header;
pub mod mcal;
pub mod mcbb;
pub mod mccv;
pub mod mcdd;
pub mod mclq;
pub mod mclv;
pub mod mcly;
pub mod mcmt;
pub mod mcnr;
pub mod mcrd;
pub mod mcrf;
pub mod mcrw;
pub mod mcse;
pub mod mcsh;
pub mod mcvt;

pub use chunk::McnkChunk;
pub use header::{McnkFlags, McnkHeader};
pub use mcal::{AlphaFormat, AlphaMap, McalChunk};
pub use mcbb::{BlendBatch, McbbChunk};
pub use mccv::{MccvChunk, VertexColor};
pub use mcdd::McddChunk;
pub use mclq::{LiquidType, LiquidVertex, MclqChunk};
pub use mclv::MclvChunk;
pub use mcly::{MclyChunk, MclyFlags, MclyLayer};
pub use mcmt::McmtChunk;
pub use mcnr::{McnrChunk, VertexNormal};
pub use mcrd::McrdChunk;
pub use mcrf::McrfChunk;
pub use mcrw::McrwChunk;
pub use mcse::{McseChunk, SoundEmitter};
pub use mcsh::McshChunk;
pub use mcvt::McvtChunk;
