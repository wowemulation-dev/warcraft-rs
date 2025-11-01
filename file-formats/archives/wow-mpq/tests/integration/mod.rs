//! Level 3: Feature Integration Tests
//!
//! These tests verify complete features work correctly end-to-end.

pub mod archive;
// Note: async_operations test is disabled until async methods are implemented in Archive
// #[cfg(feature = "async")]
// pub mod async_operations;
pub mod compression;
pub mod patch_chain_test;
pub mod path_separator_handling;
pub mod security;
