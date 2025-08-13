// use std::collections::HashMap;
//
// use crate::error::{M2Error, Result};
// use crate::model::M2Model;
// use crate::version::M2Version;
//
// /// Functions for converting models between different versions
// pub struct M2Converter {
//     /// Conversion paths between versions
//     conversion_paths: HashMap<(M2Version, M2Version), Vec<M2Version>>,
// }
//
// impl M2Converter {
//     /// Create a new converter
//     pub fn new() -> Self {
//         let mut converter = Self {
//             conversion_paths: HashMap::new(),
//         };
//
//         // Build the conversion paths
//         converter.build_conversion_paths();
//
//         converter
//     }
//
//     /// Convert a model from one version to another
//     pub fn convert(&self, model: &M2Model, target_version: M2Version) -> Result<M2Model> {
//         let source_version = model.header.version().ok_or(M2Error::ConversionError {
//             from: model.header.version,
//             to: target_version.to_header_version(),
//             reason: "Unknown source version".to_string(),
//         })?;
//
//         if source_version == target_version {
//             return Ok(model.clone());
//         }
//
//         // Check if we have a direct conversion path
//         if source_version.has_direct_conversion_path(&target_version) {
//             return model.convert(target_version);
//         }
//
//         // Look for a conversion path
//         if let Some(path) = self.get_conversion_path(source_version, target_version) {
//             // Apply conversions in sequence
//             let mut current_model = model.clone();
//
//             for step_version in path {
//                 current_model = current_model.convert(*step_version)?;
//             }
//
//             Ok(current_model)
//         } else {
//             Err(M2Error::ConversionError {
//                 from: source_version.to_header_version(),
//                 to: target_version.to_header_version(),
//                 reason: "No conversion path found".to_string(),
//             })
//         }
//     }
//
//     /// Build the conversion paths between versions
//     fn build_conversion_paths(&mut self) {
//         // Define a list of all versions in order
//         let versions = [
//             M2Version::Classic,
//             M2Version::TBC,
//             M2Version::WotLK,
//             M2Version::Cataclysm,
//             M2Version::MoP,
//             M2Version::WoD,
//             M2Version::Legion,
//             M2Version::BfA,
//             M2Version::Shadowlands,
//             M2Version::Dragonflight,
//             M2Version::TheWarWithin,
//         ];
//
//         // For each pair of versions, build a conversion path
//         for &from_version in &versions {
//             for &to_version in &versions {
//                 if from_version == to_version {
//                     continue;
//                 }
//
//                 // Direct conversion for adjacent versions
//                 if from_version.has_direct_conversion_path(&to_version) {
//                     self.conversion_paths
//                         .insert((from_version, to_version), vec![to_version]);
//                     continue;
//                 }
//
//                 // Build a path through intermediate versions
//                 let mut path = Vec::new();
//                 let direction = if to_version > from_version { 1 } else { -1 };
//
//                 let start_idx = versions.iter().position(|&v| v == from_version).unwrap();
//                 let end_idx = versions.iter().position(|&v| v == to_version).unwrap();
//
//                 if direction > 0 {
//                     // Upgrade path
//                     for &version in &versions[(start_idx + 1)..=end_idx] {
//                         path.push(version);
//                     }
//                 } else {
//                     // Downgrade path
//                     for &version in versions[end_idx..start_idx].iter().rev() {
//                         path.push(version);
//                     }
//                 }
//
//                 self.conversion_paths
//                     .insert((from_version, to_version), path);
//             }
//         }
//     }
//
//     /// Get a conversion path between two versions
//     fn get_conversion_path(&self, from: M2Version, to: M2Version) -> Option<&Vec<M2Version>> {
//         self.conversion_paths.get(&(from, to))
//     }
// }
//
// /// Default implementation
// impl Default for M2Converter {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_conversion_paths() {
//         let converter = M2Converter::new();
//
//         // Path from Classic to Cataclysm (goes through TBC and WotLK)
//         let path = converter
//             .get_conversion_path(M2Version::Classic, M2Version::Cataclysm)
//             .unwrap();
//         assert_eq!(
//             path,
//             &vec![M2Version::TBC, M2Version::WotLK, M2Version::Cataclysm]
//         );
//
//         // Path from Classic to MoP
//         let path = converter
//             .get_conversion_path(M2Version::Classic, M2Version::MoP)
//             .unwrap();
//         assert_eq!(
//             path,
//             &vec![
//                 M2Version::TBC,
//                 M2Version::WotLK,
//                 M2Version::Cataclysm,
//                 M2Version::MoP
//             ]
//         );
//
//         // Path from TheWarWithin to Classic
//         let path = converter
//             .get_conversion_path(M2Version::TheWarWithin, M2Version::Classic)
//             .unwrap();
//         assert!(path.len() > 1); // Multiple steps to go back
//
//         // Ensure we have a path for every version pair
//         let versions = [
//             M2Version::Classic,
//             M2Version::TBC,
//             M2Version::WotLK,
//             M2Version::Cataclysm,
//             M2Version::MoP,
//             M2Version::WoD,
//             M2Version::Legion,
//             M2Version::BfA,
//             M2Version::Shadowlands,
//             M2Version::Dragonflight,
//             M2Version::TheWarWithin,
//         ];
//
//         for &from_version in &versions {
//             for &to_version in &versions {
//                 if from_version == to_version {
//                     continue;
//                 }
//
//                 assert!(
//                     converter
//                         .get_conversion_path(from_version, to_version)
//                         .is_some()
//                 );
//             }
//         }
//     }
// }
