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

pub mod data_generator;
pub mod mpq_builder;

pub use data_generator::{TestDataConfig, generate_test_data};
pub use mpq_builder::{TestArchiveConfig, create_test_archive};
