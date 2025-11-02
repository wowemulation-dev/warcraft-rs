//! ADT chunk structure definitions.
//!
//! This module contains binrw-derived structures for all ADT chunk types across
//! all supported WoW versions (Vanilla 1.x through MoP 5.x).
//!
//! ## Organization
//!
//! - [`simple`] - Basic chunks with simple binrw derives (MVER, MHDR, MCIN, etc.)
//! - [`strings`] - String table chunks (MTEX, MMDX, MWMO, MMID, MWID)
//! - [`placement`] - Object placement chunks (MDDF, MODF)
//! - [`mcnk`] - Terrain chunk with nested subchunks (MCVT, MCNR, MCLY, MCAL, etc.)
//! - [`mh2o`] - Multi-level water structure (WotLK+)
//! - [`blend_mesh`] - Blend mesh system chunks (MBMH, MBBB, MBNV, MBMI) for MoP 5.x+

pub mod blend_mesh;
pub mod mcnk;
pub mod mh2o;
pub mod placement;
pub mod simple;
pub mod strings;

// Re-export simple chunk structures
pub use simple::{
    MampChunk, McinChunk, McinEntry, MfboChunk, MhdrChunk, MtxfChunk, MtxpChunk, MverChunk,
};

// Re-export string chunk structures
pub use strings::{MmdxChunk, MmidChunk, MtexChunk, MwidChunk, MwmoChunk};

// Re-export placement chunk structures
pub use placement::{DoodadPlacement, MddfChunk, ModfChunk, WmoPlacement};

// Re-export MCNK chunk structures
pub use mcnk::{
    AlphaFormat, AlphaMap, LiquidType, McalChunk, MccvChunk, MclqChunk, MclyChunk, MclyFlags,
    MclyLayer, McnkChunk, McnkFlags, McnkHeader, McnrChunk, McrfChunk, McseChunk, McshChunk,
    McvtChunk, SoundEmitter, VertexColor, VertexNormal,
};

// Re-export MH2O chunk structures
pub use mh2o::{
    DepthOnlyVertex, HeightDepthVertex, HeightUvDepthVertex, HeightUvVertex, LiquidVertexFormat,
    Mh2oAttributes, Mh2oChunk, Mh2oEntry, Mh2oHeader, Mh2oInstance, UvMapEntry,
};

// Re-export blend mesh chunk structures (MoP 5.x+)
pub use blend_mesh::{
    MbbbChunk, MbbbEntry, MbmhChunk, MbmhEntry, MbmiChunk, MbnvChunk, MbnvVertex,
};
