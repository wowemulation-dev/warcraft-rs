use crate::chunk_discovery::ChunkDiscovery;

/// Type of WMO file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmoFileType {
    /// Root WMO file containing header and references to groups
    Root,
    /// Group file containing geometry and rendering data
    Group,
}

/// Detect the type of WMO file based on chunk patterns
pub fn detect_file_type(discovery: &ChunkDiscovery) -> WmoFileType {
    // Check chunk patterns
    let chunk_ids: Vec<&str> = discovery.chunks.iter().map(|c| c.id.as_str()).collect();

    // Root files have MOHD, MOMT, MOGN and other root-specific chunks
    // Group files have MOGP as their main chunk after MVER
    if chunk_ids.contains(&"MOHD") || chunk_ids.contains(&"MOMT") {
        WmoFileType::Root
    } else if chunk_ids.contains(&"MOGP") || (chunk_ids.len() > 1 && chunk_ids[1] == "MOGP") {
        WmoFileType::Group
    } else {
        // Default to group for unknown patterns
        WmoFileType::Group
    }
}
