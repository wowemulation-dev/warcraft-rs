//! DBD (Database Definition) file parser and converter
//!
//! This module provides functionality for parsing WoW DBD definition files
//! and converting them to YAML schemas compatible with the wow-cdbc parser.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a column definition in the DBD file
#[derive(Debug, Clone)]
pub struct DbdColumn {
    pub name: String,
    pub base_type: String,
    pub foreign_key: Option<ForeignKey>,
    pub comment: Option<String>,
    pub is_optional: bool,
}

/// Represents a foreign key reference
#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub table: String,
    pub field: String,
}

/// Represents a field in a BUILD or LAYOUT section
#[derive(Debug, Clone)]
pub struct DbdField {
    pub name: String,
    pub type_size: TypeSize,
    pub is_array: bool,
    pub array_size: Option<usize>,
    pub is_key: bool,
    pub is_relation: bool,
    pub is_noninline: bool,
}

/// Type size specification
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSize {
    Unspecified,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Float,
}

impl TypeSize {
    pub fn parse_type_size(s: &str) -> Self {
        match s {
            "8" => TypeSize::Int8,
            "u8" => TypeSize::UInt8,
            "16" => TypeSize::Int16,
            "u16" => TypeSize::UInt16,
            "32" => TypeSize::Int32,
            "u32" => TypeSize::UInt32,
            _ => TypeSize::Unspecified,
        }
    }

    pub fn to_type_name(&self, base_type: &str) -> &'static str {
        match self {
            TypeSize::Int8 => "Int8",
            TypeSize::UInt8 => "UInt8",
            TypeSize::Int16 => "Int16",
            TypeSize::UInt16 => "UInt16",
            TypeSize::Int32 => "Int32",
            TypeSize::UInt32 => "UInt32",
            TypeSize::Float => "Float32",
            TypeSize::Unspecified => match base_type {
                "float" => "Float32",
                "string" | "locstring" => "String",
                _ => "UInt32",
            },
        }
    }
}

/// Represents a BUILD section with version info and fields
#[derive(Debug)]
pub struct DbdBuild {
    pub versions: Vec<String>,
    pub fields: Vec<DbdField>,
}

/// Represents a LAYOUT section with hash and associated builds
#[derive(Debug)]
pub struct DbdLayout {
    pub hash: String,
    pub builds: Vec<String>,
    pub fields: Vec<DbdField>,
}

/// Represents a complete DBD file
#[derive(Debug)]
pub struct DbdFile {
    pub columns: Vec<DbdColumn>,
    pub builds: Vec<DbdBuild>,
    pub layouts: Vec<DbdLayout>,
}

/// Parse a DBD file from the given path
pub fn parse_dbd_file(path: &Path) -> Result<DbdFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    parse_dbd_content(&content)
}

/// Parse DBD content from a string
pub fn parse_dbd_content(content: &str) -> Result<DbdFile, Box<dyn std::error::Error>> {
    let mut columns = Vec::new();
    let mut builds = Vec::new();
    let mut layouts = Vec::new();

    let mut current_section = None;
    let mut current_build_versions = Vec::new();
    let mut current_build_fields = Vec::new();
    let mut current_layout_hash = String::new();
    let mut current_layout_builds = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and human-readable COMMENT lines (per DBD spec).
        if line.is_empty() || line.starts_with("COMMENT") {
            continue;
        }

        // Check for section headers
        if line == "COLUMNS" {
            // Save any pending build/layout
            save_pending_build(
                &mut builds,
                &mut current_build_versions,
                &mut current_build_fields,
            );
            save_pending_layout(
                &mut layouts,
                &mut current_layout_hash,
                &mut current_layout_builds,
                &mut current_build_fields,
            );

            current_section = Some("COLUMNS");
            continue;
        } else if let Some(stripped) = line.strip_prefix("BUILD ") {
            let versions: Vec<String> = stripped.split(", ").map(|s| s.to_string()).collect();
            // Stacked BUILD lines (consecutive BUILDs with no fields between them)
            // share the same following field block. Accumulate versions instead
            // of starting a new build with an empty body.
            if current_section == Some("BUILD") && current_build_fields.is_empty() {
                current_build_versions.extend(versions);
            } else {
                save_pending_build(
                    &mut builds,
                    &mut current_build_versions,
                    &mut current_build_fields,
                );
                current_build_versions = versions;
            }
            current_section = Some("BUILD");
            continue;
        } else if let Some(stripped) = line.strip_prefix("LAYOUT ") {
            // Save previous build/layout if any
            save_pending_build(
                &mut builds,
                &mut current_build_versions,
                &mut current_build_fields,
            );
            save_pending_layout(
                &mut layouts,
                &mut current_layout_hash,
                &mut current_layout_builds,
                &mut current_build_fields,
            );

            current_section = Some("LAYOUT");
            let parts: Vec<&str> = stripped.split(", ").collect();
            current_layout_hash = parts[0].to_string();
            current_layout_builds.clear();
            continue;
        }

        // Parse content based on current section
        match current_section {
            Some("COLUMNS") => {
                if let Some(column) = parse_column_line(line) {
                    columns.push(column);
                }
            }
            Some("BUILD") => {
                let field = parse_field_line(line);
                current_build_fields.push(field);
            }
            Some("LAYOUT") => {
                // Check if this is another BUILD line for LAYOUT section
                if let Some(stripped) = line.strip_prefix("BUILD ") {
                    let build_versions: Vec<String> =
                        stripped.split(", ").map(|s| s.to_string()).collect();
                    current_layout_builds.extend(build_versions);
                } else {
                    // Parse field definition
                    let field = parse_field_line(line);
                    current_build_fields.push(field);
                }
            }
            _ => {}
        }
    }

    // Save any remaining build/layout
    save_pending_build(
        &mut builds,
        &mut current_build_versions,
        &mut current_build_fields,
    );
    save_pending_layout(
        &mut layouts,
        &mut current_layout_hash,
        &mut current_layout_builds,
        &mut current_build_fields,
    );

    Ok(DbdFile {
        columns,
        builds,
        layouts,
    })
}

fn save_pending_build(
    builds: &mut Vec<DbdBuild>,
    versions: &mut Vec<String>,
    fields: &mut Vec<DbdField>,
) {
    if !versions.is_empty() && !fields.is_empty() {
        builds.push(DbdBuild {
            versions: versions.clone(),
            fields: fields.clone(),
        });
        versions.clear();
        fields.clear();
    }
}

fn save_pending_layout(
    layouts: &mut Vec<DbdLayout>,
    hash: &mut String,
    builds: &mut Vec<String>,
    fields: &mut Vec<DbdField>,
) {
    if !hash.is_empty() && !fields.is_empty() {
        layouts.push(DbdLayout {
            hash: hash.clone(),
            builds: builds.clone(),
            fields: fields.clone(),
        });
        hash.clear();
        builds.clear();
        fields.clear();
    }
}

fn parse_column_line(line: &str) -> Option<DbdColumn> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return None;
    }

    let type_and_rest = parts[0];
    let rest = parts[1..].join(" ");

    // Extract base type and check for foreign key in the type specification
    let (base_type, type_foreign_key) = if let Some(angle_start) = type_and_rest.find('<') {
        let base = &type_and_rest[..angle_start];
        if let Some(angle_end) = type_and_rest.find('>') {
            let fk_str = &type_and_rest[angle_start + 1..angle_end];
            let foreign_key = fk_str.find("::").map(|sep_pos| ForeignKey {
                table: fk_str[..sep_pos].to_string(),
                field: fk_str[sep_pos + 2..].to_string(),
            });
            (base, foreign_key)
        } else {
            (type_and_rest, None)
        }
    } else {
        (type_and_rest, None)
    };

    // Check if optional (ends with ?)
    let is_optional = rest.trim_end().ends_with('?');
    let rest = if is_optional {
        rest.trim_end().trim_end_matches('?')
    } else {
        rest.trim_end()
    };

    // Parse name and remaining content (no more foreign key parsing needed since we handled it above)
    let (name, remaining) = {
        let comment_pos = rest.find("//");
        if let Some(pos) = comment_pos {
            (rest[..pos].trim().to_string(), &rest[pos..])
        } else {
            (rest.trim().to_string(), "")
        }
    };

    // Extract comment
    let comment = if remaining.trim().starts_with("//") {
        Some(remaining.trim()[2..].trim().to_string())
    } else {
        None
    };

    Some(DbdColumn {
        name,
        base_type: base_type.to_string(),
        foreign_key: type_foreign_key,
        comment,
        is_optional,
    })
}

fn parse_field_line(line: &str) -> DbdField {
    let name: String;
    let mut type_size = TypeSize::Unspecified;
    let mut is_array = false;
    let mut array_size = None;
    let mut is_key = false;
    let mut is_relation = false;
    let mut is_noninline = false;

    // Strip trailing `// human comment` (per WoWDBDefs DBD spec).
    let line = line.split("//").next().unwrap_or(line).trim();

    // Check for special markers
    let line = if let Some(stripped) = line.strip_prefix("$id$") {
        is_key = true;
        stripped
    } else if let Some(stripped) = line.strip_prefix("$noninline,id$") {
        is_key = true;
        is_noninline = true;
        stripped
    } else if let Some(stripped) = line.strip_prefix("$relation$") {
        is_relation = true;
        stripped
    } else {
        line
    };

    // Handle array notation first (can be combined with type size)
    let (base_part, array_info) = if let Some(bracket_start) = line.find('[') {
        if let Some(bracket_end) = line.find(']') {
            let array_str = &line[bracket_start + 1..bracket_end];
            is_array = true;
            array_size = array_str.parse().ok();

            // Check if there's a type spec before the array
            let before_bracket = &line[..bracket_start];
            let after_bracket = &line[bracket_end + 1..];
            (
                before_bracket.to_string() + after_bracket,
                Some((is_array, array_size)),
            )
        } else {
            (line.to_string(), None)
        }
    } else {
        (line.to_string(), None)
    };

    // Apply array info if found
    if let Some((arr, size)) = array_info {
        is_array = arr;
        array_size = size;
    }

    // Parse type size notation
    if let Some(angle_start) = base_part.find('<') {
        name = base_part[..angle_start].to_string();
        if let Some(angle_end) = base_part.find('>') {
            let size_str = &base_part[angle_start + 1..angle_end];
            type_size = TypeSize::parse_type_size(size_str);
        }
    } else {
        name = base_part.trim().to_string();
    }

    DbdField {
        name,
        type_size,
        is_array,
        array_size,
        is_key,
        is_relation,
        is_noninline,
    }
}

/// Convert a DBD file to YAML schemas
pub fn convert_to_yaml_schemas(
    dbd_file: &DbdFile,
    base_name: &str,
    version_filter: Option<&str>,
    generate_all: bool,
) -> Vec<(String, String, String)> {
    let mut results = Vec::new();

    // Create a map of column info for quick lookup
    let column_map: HashMap<String, &DbdColumn> = dbd_file
        .columns
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    // Generate schemas for builds
    for build in &dbd_file.builds {
        if should_generate_version(&build.versions, version_filter, generate_all) {
            let version_suffix = determine_version_suffix(&build.versions);
            let yaml_content = generate_yaml_schema(
                &column_map,
                build,
                base_name,
                &version_suffix,
                version_filter,
            );
            let filename = generate_filename(base_name, &build.versions[0]);
            results.push((filename, yaml_content, version_suffix));
        }
    }

    // Generate schemas for layouts
    for layout in &dbd_file.layouts {
        let pseudo_build = DbdBuild {
            versions: layout.builds.clone(),
            fields: layout.fields.clone(),
        };

        if should_generate_version(&pseudo_build.versions, version_filter, generate_all) {
            let version_suffix = determine_version_suffix(&pseudo_build.versions);
            let yaml_content = generate_yaml_schema(
                &column_map,
                &pseudo_build,
                base_name,
                &version_suffix,
                version_filter,
            );
            let filename = if layout.builds.is_empty() {
                format!("{}_layout_{}.yaml", base_name, &layout.hash[..8])
            } else {
                generate_filename(base_name, &layout.builds[0])
            };
            results.push((filename, yaml_content, version_suffix));
        }
    }

    // If no schemas generated and no specific filters, generate at least one for the latest
    if results.is_empty()
        && version_filter.is_none()
        && !generate_all
        && !dbd_file.layouts.is_empty()
    {
        let layout = &dbd_file.layouts.last().unwrap();
        let pseudo_build = DbdBuild {
            versions: layout.builds.clone(),
            fields: layout.fields.clone(),
        };
        let version_suffix = "Latest".to_string();
        let yaml_content = generate_yaml_schema(
            &column_map,
            &pseudo_build,
            base_name,
            &version_suffix,
            version_filter,
        );
        let filename = format!("{base_name}_latest.yaml");
        results.push((filename, yaml_content, version_suffix));
    }

    results
}

fn should_generate_version(versions: &[String], filter: Option<&str>, generate_all: bool) -> bool {
    if generate_all {
        return true;
    }

    if let Some(target) = filter {
        versions.iter().any(|v| v.contains(target))
    } else {
        false
    }
}

fn determine_version_suffix(versions: &[String]) -> String {
    if versions.is_empty() {
        return "Unknown".to_string();
    }

    let first = &versions[0];
    let last = versions.last().unwrap();

    // Check for specific version patterns
    for v in versions {
        if v.contains("1.12") {
            return "1.12.x (Vanilla)".to_string();
        } else if v.contains("2.4.3") {
            return "2.4.3 (TBC)".to_string();
        } else if v.contains("3.3.5") {
            return "3.3.5a (WotLK)".to_string();
        } else if v.contains("4.3.4") {
            return "4.3.4 (Cataclysm)".to_string();
        } else if v.contains("5.4.8") {
            return "5.4.8 (MoP)".to_string();
        } else if v.contains("6.2.4") {
            return "6.2.4 (WoD)".to_string();
        } else if v.contains("7.3.5") {
            return "7.3.5 (Legion)".to_string();
        } else if v.contains("8.3") {
            return "8.3.x (BfA)".to_string();
        } else if v.contains("9.2") {
            return "9.2.x (Shadowlands)".to_string();
        } else if v.contains("10.") {
            return "10.x (Dragonflight)".to_string();
        }
    }

    // Default: show range
    if first == last {
        first.clone()
    } else {
        format!("{first} - {last}")
    }
}

fn generate_filename(base_name: &str, version: &str) -> String {
    let sanitized_version = version
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect::<String>()
        .replace('.', "_");

    format!("{base_name}_{sanitized_version}.yaml")
}

/// Locale ordering for the WotLK-era (3.x and 4.0 pre-collapse) locstring layout.
/// 16 stringref slots are stored in this order, followed by a single u32 flags field.
const LOCSTRING_LOCALES_WOTLK: &[&str] = &[
    "enUS", "koKR", "frFR", "deDE", "zhCN", "zhTW", "esES", "esMX", "ruRU", "unk9", "unk10",
    "unk11", "unk12", "unk13", "unk14", "unk15",
];

/// Locale ordering for the Vanilla/TBC-era (1.x, 2.x) locstring layout.
/// 8 stringref slots are stored in this order, followed by a single u32 flags field.
const LOCSTRING_LOCALES_CLASSIC: &[&str] = &[
    "enUS", "koKR", "frFR", "deDE", "zhCN", "zhTW", "esES", "esMX",
];

/// Determine the per-locale stringref slots that make up a `locstring`.
///
/// When `target_version` is provided (e.g. the `--version` CLI filter), the
/// era is resolved from that string. Otherwise it's resolved from the build's
/// first listed version. This matters because a single field block in a DBD
/// is often shared across many builds spanning multiple expansions; the file
/// on disk only matches one of them, so the user's target wins.
///
/// Returns an empty slice for 4.1+ builds where `locstring` collapsed back to
/// a single string field.
fn locstring_locales_for_build(
    build: &DbdBuild,
    target_version: Option<&str>,
) -> &'static [&'static str] {
    let pick = target_version
        .or_else(|| build.versions.first().map(|s| s.as_str()))
        .unwrap_or("");

    let mut parts = pick.split('.');
    let major: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    match major {
        0..=2 => LOCSTRING_LOCALES_CLASSIC,
        3 => LOCSTRING_LOCALES_WOTLK,
        // The 4.0 pre-Cataclysm client kept the WotLK locstring layout; from
        // 4.1 onward it collapsed back to a single string field.
        4 if minor == 0 => LOCSTRING_LOCALES_WOTLK,
        _ => &[],
    }
}

fn generate_yaml_schema(
    column_map: &HashMap<String, &DbdColumn>,
    build: &DbdBuild,
    base_name: &str,
    version_suffix: &str,
    target_version: Option<&str>,
) -> String {
    let mut yaml = String::new();

    // Header
    yaml.push_str(&format!("# {base_name} schema for WoW {version_suffix}\n"));
    yaml.push_str(&format!("# Generated from {base_name}.dbd\n"));
    yaml.push_str(&format!("# Build range: {}\n\n", build.versions.join(", ")));

    yaml.push_str(&format!("name: {base_name}\n"));

    // Only emit key_field when the DBD explicitly marks one with $id$
    if let Some(key_field) = build.fields.iter().find(|f| f.is_key).map(|f| &f.name) {
        yaml.push_str(&format!("key_field: {key_field}\n"));
    }
    yaml.push_str("fields:\n");

    let locstring_locales = locstring_locales_for_build(build, target_version);

    // Generate fields
    for field in &build.fields {
        let column = column_map.get(&field.name);

        let is_locstring = column.is_some_and(|c| c.base_type == "locstring");

        if is_locstring && !locstring_locales.is_empty() {
            // Pre-4.1 layout: emit one StringRef per locale plus a flags field.
            for locale in locstring_locales {
                yaml.push_str(&format!("  - name: {}_{}\n", field.name, locale));
                yaml.push_str("    type_name: String\n");
                yaml.push_str(&format!(
                    "    description: Localized {} text ({})\n",
                    field.name.to_lowercase(),
                    locale
                ));
            }
            yaml.push_str(&format!("  - name: {}_flags\n", field.name));
            yaml.push_str("    type_name: UInt32\n");
            yaml.push_str(&format!(
                "    description: Locale flags bitmask for {}\n",
                field.name
            ));
            continue;
        }

        // Determine field type
        let field_type = if let Some(col) = column {
            if col.base_type == "locstring" || col.base_type == "string" {
                "String"
            } else if col.base_type == "float" {
                "Float32"
            } else {
                field.type_size.to_type_name(&col.base_type)
            }
        } else {
            // No column definition, use field's type size
            field.type_size.to_type_name("int")
        };

        yaml.push_str(&format!("  - name: {}\n", field.name));
        yaml.push_str(&format!("    type_name: {field_type}\n"));

        if field.is_array {
            yaml.push_str("    is_array: true\n");
            if let Some(size) = field.array_size {
                yaml.push_str(&format!("    array_size: {size}\n"));
            }
        }

        // Add description
        let description = generate_field_description(field, column);
        // Quote description if it contains special YAML characters
        let description = if description.contains('&')
            || description.contains(':')
            || description.contains('#')
        {
            format!("\"{}\"", description.replace('"', "\\\""))
        } else {
            description
        };
        yaml.push_str(&format!("    description: {description}\n"));
    }

    yaml
}

fn generate_field_description(field: &DbdField, column: Option<&&DbdColumn>) -> String {
    if let Some(col) = column {
        // Use comment if available
        if let Some(ref comment) = col.comment {
            return comment.clone();
        }

        // Use foreign key info
        if let Some(ref fk) = col.foreign_key {
            return format!("Reference to {}::{}", fk.table, fk.field);
        }
    }

    // Generate based on field properties
    if field.is_key {
        "Unique identifier".to_string()
    } else if field.is_relation {
        format!("Reference to related {}", field.name.replace("ID", ""))
    } else if let Some(col) = column {
        match col.base_type.as_str() {
            "locstring" => format!("Localized {} text", field.name.to_lowercase()),
            "string" => format!("{} text", field.name),
            "float" => format!("{} value", field.name),
            _ => format!("{} field", field.name),
        }
    } else {
        field.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_line() {
        let field = parse_field_line("$id$ID<32>");
        assert_eq!(field.name, "ID");
        assert!(field.is_key);
        assert_eq!(field.type_size, TypeSize::Int32);

        let field = parse_field_line("Pos[3]");
        assert_eq!(field.name, "Pos");
        assert!(field.is_array);
        assert_eq!(field.array_size, Some(3));

        let field = parse_field_line("Icon<u16>[9]");
        assert_eq!(field.name, "Icon");
        assert!(field.is_array);
        assert_eq!(field.array_size, Some(9));
        assert_eq!(field.type_size, TypeSize::UInt16);
    }

    #[test]
    fn test_parse_skips_comment_lines() {
        let dbd = "\
COLUMNS
int ID

BUILD 3.3.5.12340
COMMENT unknown_flag=1
$id$ID<32>
";
        let dbd_file = parse_dbd_content(dbd).unwrap();
        // The COMMENT line must not be turned into a phantom field.
        assert_eq!(dbd_file.builds[0].fields.len(), 1);
        assert_eq!(dbd_file.builds[0].fields[0].name, "ID");
    }

    #[test]
    fn test_parse_field_line_strips_inline_comment() {
        // DanceMoves.dbd has lines like `Name_lang // pre-8622 ...` — the comment
        // must be stripped from the field name so column lookups still work.
        let field =
            parse_field_line("Name_lang // pre-8622 this uses the old 9 locale locstring format");
        assert_eq!(field.name, "Name_lang");

        let field = parse_field_line("$id$ID<32> // primary key");
        assert_eq!(field.name, "ID");
        assert!(field.is_key);
        assert_eq!(field.type_size, TypeSize::Int32);
    }

    #[test]
    fn test_parse_stacked_builds_share_fields() {
        // AreaTrigger.dbd-style: several consecutive BUILD lines share a single
        // field block. All builds should be saved with the same fields.
        let dbd = "\
COLUMNS
int ID

BUILD 3.0.8.9328
BUILD 3.0.1.8303-3.3.5.12340
BUILD 2.0.0.5610-2.4.3.8606
$id$ID<32>
";
        let dbd_file = parse_dbd_content(dbd).unwrap();
        // One stacked BUILD group becomes one DbdBuild whose `versions` lists all entries.
        assert_eq!(dbd_file.builds.len(), 1);
        assert_eq!(
            dbd_file.builds[0].versions,
            vec![
                "3.0.8.9328",
                "3.0.1.8303-3.3.5.12340",
                "2.0.0.5610-2.4.3.8606",
            ]
        );
        assert_eq!(dbd_file.builds[0].fields.len(), 1);
    }

    #[test]
    fn test_locstring_expansion_wotlk() {
        let dbd = "\
COLUMNS
int ID
locstring Name_lang
int Flags

BUILD 3.3.5.12340
$id$ID<32>
Name_lang
Flags<32>
";
        let dbd_file = parse_dbd_content(dbd).unwrap();
        let column_map: HashMap<String, &DbdColumn> = dbd_file
            .columns
            .iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        let yaml = generate_yaml_schema(
            &column_map,
            &dbd_file.builds[0],
            "Test",
            "3.3.5a",
            Some("3.3.5"),
        );
        let name_lines: Vec<&str> = yaml
            .lines()
            .filter(|l| l.trim_start().starts_with("- name:"))
            .collect();
        // ID + 16 locale stringrefs + Name_lang_flags + Flags = 19
        assert_eq!(name_lines.len(), 19);
        assert!(yaml.contains("- name: Name_lang_enUS"));
        assert!(yaml.contains("- name: Name_lang_ruRU"));
        assert!(yaml.contains("- name: Name_lang_unk15"));
        assert!(yaml.contains("- name: Name_lang_flags"));
        assert!(!yaml.contains("- name: Name_lang\n"));
    }

    #[test]
    fn test_locstring_expansion_classic() {
        let dbd = "\
COLUMNS
int ID
locstring Name_lang

BUILD 1.12.1.5875
$id$ID<32>
Name_lang
";
        let dbd_file = parse_dbd_content(dbd).unwrap();
        let column_map: HashMap<String, &DbdColumn> = dbd_file
            .columns
            .iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        let yaml = generate_yaml_schema(
            &column_map,
            &dbd_file.builds[0],
            "Test",
            "1.12.x",
            Some("1.12"),
        );
        let name_lines: Vec<&str> = yaml
            .lines()
            .filter(|l| l.trim_start().starts_with("- name:"))
            .collect();
        // ID + 8 locale stringrefs + flags = 10
        assert_eq!(name_lines.len(), 10);
        assert!(yaml.contains("- name: Name_lang_enUS"));
        assert!(!yaml.contains("- name: Name_lang_ruRU"));
        assert!(yaml.contains("- name: Name_lang_flags"));
    }

    #[test]
    fn test_locstring_no_expansion_modern() {
        let dbd = "\
COLUMNS
int ID
locstring Name_lang

BUILD 5.4.8.18414
$id$ID<32>
Name_lang
";
        let dbd_file = parse_dbd_content(dbd).unwrap();
        let column_map: HashMap<String, &DbdColumn> = dbd_file
            .columns
            .iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        let yaml = generate_yaml_schema(
            &column_map,
            &dbd_file.builds[0],
            "Test",
            "5.4.8",
            Some("5.4.8"),
        );
        // ID + Name_lang (single string)
        assert!(yaml.contains("- name: Name_lang\n"));
        assert!(!yaml.contains("Name_lang_enUS"));
    }

    #[test]
    fn test_locstring_expansion_uses_target_version_not_first_build() {
        // Achievement_Category-style: a stacked BUILD group spans 6.0 down to
        // 3.3.5; the listed *first* version is 6.0, but a 3.3.5 client needs
        // the WotLK 16-locale layout. The target_version filter must win.
        let dbd = "\
COLUMNS
int ID
locstring Name_lang
int Ui_order

BUILD 6.0.1.18179
BUILD 4.0.0.11792
BUILD 3.0.1.8622-3.3.5.12340
$id$ID<32>
Name_lang
Ui_order<32>
";
        let dbd_file = parse_dbd_content(dbd).unwrap();
        let column_map: HashMap<String, &DbdColumn> = dbd_file
            .columns
            .iter()
            .map(|c| (c.name.clone(), c))
            .collect();
        // Filtering for 3.3.5 should expand the locstring to 16 locales + flags.
        let yaml = generate_yaml_schema(
            &column_map,
            &dbd_file.builds[0],
            "Test",
            "3.3.5a",
            Some("3.3.5"),
        );
        let name_lines: Vec<&str> = yaml
            .lines()
            .filter(|l| l.trim_start().starts_with("- name:"))
            .collect();
        // ID + 16 locale stringrefs + Name_lang_flags + Ui_order = 19
        assert_eq!(name_lines.len(), 19);
        assert!(yaml.contains("- name: Name_lang_enUS"));
        assert!(yaml.contains("- name: Name_lang_flags"));
    }

    #[test]
    fn test_parse_column_line() {
        let col = parse_column_line("int ID").unwrap();
        assert_eq!(col.name, "ID");
        assert_eq!(col.base_type, "int");
        assert!(col.foreign_key.is_none());

        let col = parse_column_line(
            "int<SpellCastTimes::ID> CastingTimeIndex // todo: rename CastingTimeID",
        )
        .unwrap();
        assert_eq!(col.name, "CastingTimeIndex");
        assert_eq!(col.base_type, "int");
        assert!(col.foreign_key.is_some());
        let fk = col.foreign_key.unwrap();
        assert_eq!(fk.table, "SpellCastTimes");
        assert_eq!(fk.field, "ID");
        assert_eq!(col.comment.unwrap(), "todo: rename CastingTimeID");

        let col = parse_column_line("locstring Name_lang?").unwrap();
        assert_eq!(col.name, "Name_lang");
        assert!(col.is_optional);
    }
}
