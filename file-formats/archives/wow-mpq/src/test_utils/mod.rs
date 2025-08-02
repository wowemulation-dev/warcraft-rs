//! Test utilities for MPQ archives
//!
//! This module provides utilities for generating test data and creating
//! test MPQ archives, replacing the Python scripts previously used.
//!
//! # Test Data Generation
//!
//! The [`data_generator`] module provides functionality to create directory
//! structures with various file types and sizes for testing archive creation.
//!
//! # Test Archive Creation
//!
//! The [`mpq_builder`] module provides utilities to create test MPQ archives
//! with different configurations, compression methods, encryption settings,
//! and edge cases for comprehensive testing.
//!
//! # WoW Data Location
//!
//! The [`wow_data`] module provides utilities for locating World of Warcraft
//! game data directories across different versions and platforms, making
//! examples portable and independent of hardcoded paths.
pub mod data_generator;
pub mod mpq_builder;
pub mod wow_data;

pub use data_generator::{TestDataConfig, generate_test_data};
pub use mpq_builder::{TestArchiveConfig, create_test_archive};
pub use wow_data::{
    WowVersion, find_any_wow_data, find_wow_data, get_mpq_path, print_setup_instructions,
};
