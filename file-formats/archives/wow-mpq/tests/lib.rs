//! Hierarchical test organization for wow_mpq
//!
//! Tests are organized by level:
//! - Level 1: Core primitives (unit tests in src/)
//! - Level 2: Component integration
//! - Level 3: Feature integration
//! - Level 4: End-to-end scenarios
//! - Level 5: Compatibility & performance

// Common test utilities
mod common;

// Level 2: Component Integration Tests
mod component;

// Level 3: Feature Integration Tests
mod integration;

// Level 4: End-to-End Scenario Tests
mod scenarios;

// Level 5: Compliance Tests
mod compliance;
