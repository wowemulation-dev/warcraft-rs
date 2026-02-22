//! Tree structure rendering utilities for file format visualization

use console::Style;
use std::collections::HashMap;

/// Represents a node in a tree structure
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub node_type: NodeType,
    pub size: Option<u64>,
    pub children: Vec<TreeNode>,
    pub metadata: HashMap<String, String>,
    pub external_refs: Vec<ExternalRef>,
}

/// Types of nodes in the tree
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Root,
    Header,
    #[allow(dead_code)]
    Chunk,
    #[allow(dead_code)]
    Table,
    File,
    Directory,
    #[allow(dead_code)] // Reserved for future use
    Reference,
    #[allow(dead_code)]
    Property,
    #[allow(dead_code)]
    Data,
}

/// External file reference
#[derive(Debug, Clone)]
pub struct ExternalRef {
    pub path: String,
    pub ref_type: RefType,
    pub exists: Option<bool>,
}

/// Types of external references
#[derive(Debug, Clone, PartialEq)]
pub enum RefType {
    Texture,
    Model,
    Animation,
    Map,
    Database,
    Sound,
    Script,
    Archive,
    Unknown,
}

/// Options for tree rendering
#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub max_depth: Option<usize>,
    pub show_external_refs: bool,
    pub no_color: bool,
    pub show_metadata: bool,
    pub compact: bool,
    #[cfg_attr(not(feature = "wmo"), allow(dead_code))]
    pub verbose: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            show_external_refs: true,
            no_color: false,
            show_metadata: true,
            compact: false,
            verbose: false,
        }
    }
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(name: String, node_type: NodeType) -> Self {
        Self {
            name,
            node_type,
            size: None,
            children: Vec::new(),
            metadata: HashMap::new(),
            external_refs: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Set the size of this node
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Add external reference
    pub fn with_external_ref(mut self, path: &str, ref_type: RefType) -> Self {
        self.external_refs.push(ExternalRef {
            path: path.to_string(),
            ref_type,
            exists: None,
        });
        self
    }
}

impl ExternalRef {
    /// Get emoji icon for reference type
    pub fn icon(&self) -> &'static str {
        match self.ref_type {
            RefType::Texture => "🖼️",
            RefType::Model => "🏗️",
            RefType::Animation => "📽️",
            RefType::Map => "🗺️",
            RefType::Database => "📊",
            RefType::Sound => "🔊",
            RefType::Script => "📄",
            RefType::Archive => "📦",
            RefType::Unknown => "📁",
        }
    }

    /// Get color style based on existence
    pub fn style(&self, no_color: bool) -> Style {
        if no_color {
            Style::new()
        } else {
            match self.exists {
                Some(true) => Style::new().green(),
                Some(false) => Style::new().red(),
                None => Style::new().yellow(),
            }
        }
    }
}

impl NodeType {
    /// Get emoji icon for node type
    pub fn icon(&self) -> &'static str {
        match self {
            NodeType::Root => "📁",
            NodeType::Header => "📋",
            NodeType::Chunk => "📦",
            NodeType::Table => "📊",
            NodeType::File => "📄",
            NodeType::Directory => "📁",
            NodeType::Reference => "🔗",
            NodeType::Property => "🏷️",
            NodeType::Data => "💾",
        }
    }

    /// Get color style for node type
    pub fn style(&self, no_color: bool) -> Style {
        if no_color {
            Style::new()
        } else {
            match self {
                NodeType::Root => Style::new().bold().cyan(),
                NodeType::Header => Style::new().bold().yellow(),
                NodeType::Chunk => Style::new().blue(),
                NodeType::Table => Style::new().magenta(),
                NodeType::File => Style::new().green(),
                NodeType::Directory => Style::new().cyan(),
                NodeType::Reference => Style::new().yellow(),
                NodeType::Property => Style::new().dim(),
                NodeType::Data => Style::new().white(),
            }
        }
    }
}

/// Render a tree structure to string
pub fn render_tree(root: &TreeNode, options: &TreeOptions) -> String {
    let mut output = String::new();
    render_node(root, &mut output, "", true, 0, options);
    output
}

/// Render a single node and its children
fn render_node(
    node: &TreeNode,
    output: &mut String,
    prefix: &str,
    is_last: bool,
    depth: usize,
    options: &TreeOptions,
) {
    if let Some(max_depth) = options.max_depth
        && depth > max_depth
    {
        return;
    }

    let icon = node.node_type.icon();
    let style = node.node_type.style(options.no_color);
    let connector = if depth == 0 {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    let mut line = format!(
        "{}{}{} {}",
        prefix,
        connector,
        icon,
        style.apply_to(&node.name)
    );

    if let Some(size) = node.size {
        line.push_str(&format!(" ({})", format_bytes(size)));
    }

    if options.show_metadata && !node.metadata.is_empty() && options.compact {
        let mut meta_parts = Vec::new();
        for (key, value) in &node.metadata {
            if ["version", "count", "flags", "type"].contains(&key.as_str()) {
                meta_parts.push(format!("{key}:{value}"));
            }
        }
        if !meta_parts.is_empty() {
            line.push_str(&format!(" [{}]", meta_parts.join(", ")));
        }
    }

    output.push_str(&line);
    output.push('\n');

    if options.show_metadata && !options.compact && !node.metadata.is_empty() {
        let child_prefix = if depth == 0 {
            ""
        } else if is_last {
            "    "
        } else {
            "│   "
        };
        let meta_prefix = format!("{prefix}{child_prefix}    ");

        for (key, value) in &node.metadata {
            let meta_style = Style::new().dim();
            output.push_str(&format!(
                "{}🏷️  {}: {}\n",
                meta_prefix,
                meta_style.apply_to(key),
                value
            ));
        }
    }

    if options.show_external_refs && !node.external_refs.is_empty() {
        let child_prefix = if depth == 0 {
            ""
        } else if is_last {
            "    "
        } else {
            "│   "
        };
        let ref_prefix = format!("{prefix}{child_prefix}    ");

        for ext_ref in &node.external_refs {
            let icon = ext_ref.icon();
            let style = ext_ref.style(options.no_color);
            output.push_str(&format!(
                "{}└─→ {} {}\n",
                ref_prefix,
                icon,
                style.apply_to(&ext_ref.path)
            ));
        }
    }

    // Render children
    if !node.children.is_empty() {
        let new_prefix = if depth == 0 {
            String::new()
        } else {
            format!("{}{}", prefix, if is_last { "    " } else { "│   " })
        };

        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            render_node(
                child,
                output,
                &new_prefix,
                is_last_child,
                depth + 1,
                options,
            );
        }
    }
}

/// Format bytes in human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Detect reference type from file extension
pub fn detect_ref_type(path: &str) -> RefType {
    let path_lower = path.to_lowercase();

    if path_lower.ends_with(".blp") {
        RefType::Texture
    } else if path_lower.ends_with(".m2") || path_lower.ends_with(".mdx") {
        RefType::Model
    } else if path_lower.ends_with(".anim") || path_lower.ends_with(".bone") {
        RefType::Animation
    } else if path_lower.ends_with(".wdt")
        || path_lower.ends_with(".adt")
        || path_lower.ends_with(".wdl")
    {
        RefType::Map
    } else if path_lower.ends_with(".dbc") || path_lower.ends_with(".db2") {
        RefType::Database
    } else if path_lower.ends_with(".wav") || path_lower.ends_with(".mp3") {
        RefType::Sound
    } else if path_lower.ends_with(".lua") || path_lower.ends_with(".xml") {
        RefType::Script
    } else if path_lower.ends_with(".mpq") {
        RefType::Archive
    } else {
        RefType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_rendering() {
        let root = TreeNode::new("test.mpq".to_string(), NodeType::Root)
            .with_size(1024)
            .with_metadata("version", "v2")
            .add_child(
                TreeNode::new("Header".to_string(), NodeType::Header)
                    .with_size(32)
                    .with_metadata("format", "MPQ v2"),
            )
            .add_child(
                TreeNode::new("Files".to_string(), NodeType::Directory).add_child(
                    TreeNode::new("texture.blp".to_string(), NodeType::File)
                        .with_size(2048)
                        .with_external_ref("Interface/Icons/texture.blp", RefType::Texture),
                ),
            );

        let options = TreeOptions::default();
        let output = render_tree(&root, &options);

        assert!(output.contains("test.mpq"));
        assert!(output.contains("Header"));
        assert!(output.contains("Files"));
        assert!(output.contains("texture.blp"));
    }

    #[test]
    fn test_ref_type_detection() {
        assert_eq!(detect_ref_type("texture.blp"), RefType::Texture);
        assert_eq!(detect_ref_type("model.m2"), RefType::Model);
        assert_eq!(detect_ref_type("data.dbc"), RefType::Database);
        assert_eq!(detect_ref_type("archive.mpq"), RefType::Archive);
    }
}
