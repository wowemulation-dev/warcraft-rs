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

// Re-export utilities when needed
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
pub use format::*;
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
pub use io::*;
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
pub use progress::*;
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
pub use table::*;
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
pub use tree::*;
