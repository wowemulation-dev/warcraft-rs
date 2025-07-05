//! Cryptographic types and constants

/// Hash types for MPQ operations
pub mod hash_type {
    /// Hash for table offset calculation
    pub const TABLE_OFFSET: u32 = 0x000;
    /// First part of filename hash
    pub const NAME_A: u32 = 0x100;
    /// Second part of filename hash
    pub const NAME_B: u32 = 0x200;
    /// File encryption key generation
    pub const FILE_KEY: u32 = 0x300;
    /// Secondary key mixing (used internally)
    pub const KEY2_MIX: u32 = 0x400;
}
