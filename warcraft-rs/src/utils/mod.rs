//! Shared utilities for the warcraft-rs CLI

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl",
    test
))]
pub mod format;

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl",
    test
))]
pub mod io;

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl",
    test
))]
pub mod progress;

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl",
    test
))]
pub mod table;

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl",
    test
))]
pub mod tree;

// Re-export utilities only when actually used by commands
#[cfg(feature = "mpq")]
pub use format::*;

#[cfg(feature = "mpq")]
pub use io::*;

#[cfg(any(feature = "mpq", feature = "wdl"))]
pub use progress::*;

#[cfg(any(feature = "mpq", feature = "wdl"))]
pub use table::*;

#[cfg(any(
    feature = "mpq",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl"
))]
pub use tree::*;
