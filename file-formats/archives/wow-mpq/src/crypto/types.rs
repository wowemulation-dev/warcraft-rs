//! Cryptographic types and constants

/// Hash types for MPQ operations
pub mod hash_type {
    /// Hash for table offset calculation
    pub const TABLE_OFFSET: u32 = 0;
    /// First part of filename hash
    pub const NAME_A: u32 = 1;
    /// Second part of filename hash
    pub const NAME_B: u32 = 2;
    /// File encryption key generation
    pub const FILE_KEY: u32 = 3;
    /// Secondary key mixing
    pub const KEY2_MIX: u32 = 4;
}
