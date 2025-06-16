//! Schema loading functionality

#[cfg(feature = "yaml")]
use crate::{FieldType, Schema, SchemaField};
#[cfg(feature = "yaml")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "yaml")]
use std::fs::File;
#[cfg(feature = "yaml")]
use std::io::BufReader;
#[cfg(feature = "yaml")]
use std::path::Path;

#[cfg(feature = "yaml")]
/// Schema definition for YAML files
#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaDefinition {
    /// Name of the schema
    pub name: String,
    /// Fields in the schema
    pub fields: Vec<SchemaFieldDefinition>,
    /// Name of the key field, if any
    pub key_field: Option<String>,
}

#[cfg(feature = "yaml")]
/// Field definition for YAML files
#[derive(Debug, Deserialize, Serialize)]
pub struct SchemaFieldDefinition {
    /// Name of the field
    pub name: String,
    /// Type of the field
    pub type_name: String,
    /// Whether the field is an array
    #[serde(default)]
    pub is_array: bool,
    /// Size of the array, if the field is an array
    pub array_size: Option<usize>,
}

#[cfg(feature = "yaml")]
impl SchemaDefinition {
    /// Load a schema from a YAML file
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let schema_def: SchemaDefinition = serde_yaml_ng::from_reader(reader)?;
        Ok(schema_def)
    }

    /// Load a schema from a YAML string
    pub fn from_yaml_str(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let schema_def: SchemaDefinition = serde_yaml_ng::from_str(yaml)?;
        Ok(schema_def)
    }

    /// Convert a schema definition to a Schema
    pub fn to_schema(&self) -> Result<Schema, Box<dyn std::error::Error>> {
        let mut schema = Schema::new(&self.name);

        for field_def in &self.fields {
            let field_type = match field_def.type_name.to_lowercase().as_str() {
                "int32" => FieldType::Int32,
                "uint32" => FieldType::UInt32,
                "float32" => FieldType::Float32,
                "string" => FieldType::String,
                "bool" => FieldType::Bool,
                "uint8" => FieldType::UInt8,
                "int8" => FieldType::Int8,
                "uint16" => FieldType::UInt16,
                "int16" => FieldType::Int16,
                _ => return Err(format!("Unknown field type: {}", field_def.type_name).into()),
            };

            let field = if field_def.is_array {
                if let Some(size) = field_def.array_size {
                    SchemaField::new_array(&field_def.name, field_type, size)
                } else {
                    return Err("Array fields must specify an array_size".to_string().into());
                }
            } else {
                SchemaField::new(&field_def.name, field_type)
            };

            schema.add_field(field);
        }

        if let Some(key_field) = &self.key_field {
            schema.set_key_field(key_field);
        }

        Ok(schema)
    }
}
