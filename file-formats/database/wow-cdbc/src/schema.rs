//! Schema definitions for DBC files

/// Represents the type of a field in a DBC record
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// 32-bit signed integer
    Int32,
    /// 32-bit unsigned integer
    UInt32,
    /// 32-bit floating point number
    Float32,
    /// String reference (offset into the string block)
    String,
    /// Boolean value (represented as a 32-bit integer)
    Bool,
    /// 8-bit unsigned integer
    UInt8,
    /// 8-bit signed integer
    Int8,
    /// 16-bit unsigned integer
    UInt16,
    /// 16-bit signed integer
    Int16,
}

impl FieldType {
    /// Get the size of the field type in bytes
    pub fn size(&self) -> usize {
        match self {
            FieldType::Int32 => 4,
            FieldType::UInt32 => 4,
            FieldType::Float32 => 4,
            FieldType::String => 4, // String references are 32-bit offsets
            FieldType::Bool => 4,   // Booleans are represented as 32-bit integers
            FieldType::UInt8 => 1,
            FieldType::Int8 => 1,
            FieldType::UInt16 => 2,
            FieldType::Int16 => 2,
        }
    }
}

/// Represents a field in a DBC schema
#[derive(Debug, Clone)]
pub struct SchemaField {
    /// Name of the field
    pub name: String,
    /// Type of the field
    pub field_type: FieldType,
    /// Whether the field is an array
    pub is_array: bool,
    /// Size of the array, if the field is an array
    pub array_size: Option<usize>,
}

impl SchemaField {
    /// Create a new schema field
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            is_array: false,
            array_size: None,
        }
    }

    /// Create a new array schema field
    pub fn new_array(name: impl Into<String>, field_type: FieldType, array_size: usize) -> Self {
        Self {
            name: name.into(),
            field_type,
            is_array: true,
            array_size: Some(array_size),
        }
    }

    /// Get the total size of the field in bytes
    pub fn size(&self) -> usize {
        if self.is_array {
            self.field_type.size() * self.array_size.unwrap_or(0)
        } else {
            self.field_type.size()
        }
    }
}

/// Represents a schema for a DBC file
#[derive(Debug, Clone)]
pub struct Schema {
    /// Name of the schema
    pub name: String,
    /// Fields in the schema
    pub fields: Vec<SchemaField>,
    /// Index of the key field, if any
    pub key_field_index: Option<usize>,
    /// Whether the schema is validated
    pub is_validated: bool,
}

impl Schema {
    /// Create a new schema
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            key_field_index: None,
            is_validated: false,
        }
    }

    /// Add a field to the schema
    pub fn add_field(&mut self, field: SchemaField) -> &mut Self {
        self.fields.push(field);
        self.is_validated = false;
        self
    }

    /// Set the key field by index
    pub fn set_key_field_index(&mut self, index: usize) -> &mut Self {
        self.key_field_index = Some(index);
        self.is_validated = false;
        self
    }

    /// Set the key field by name
    ///
    /// # Panics
    ///
    /// Panics if the field with the given name is not found in the schema.
    /// This is intentional as it indicates a programming error in schema definition.
    pub fn set_key_field(&mut self, name: &str) -> &mut Self {
        let index = self
            .fields
            .iter()
            .position(|f| f.name == name)
            .unwrap_or_else(|| panic!("Field not found: {name}"));
        self.set_key_field_index(index)
    }

    /// Try to set the key field by name, returning an error if the field is not found
    pub fn try_set_key_field(&mut self, name: &str) -> Result<&mut Self, String> {
        let index = self
            .fields
            .iter()
            .position(|f| f.name == name)
            .ok_or_else(|| format!("Field not found: {name}"))?;
        Ok(self.set_key_field_index(index))
    }

    /// Calculate the total size of a record in bytes
    pub fn record_size(&self) -> usize {
        self.fields.iter().map(|f| f.size()).sum()
    }

    /// Validate the schema against a DBC header
    pub fn validate(&mut self, field_count: u32, record_size: u32) -> Result<(), String> {
        let schema_field_count = if self.fields.iter().any(|f| f.is_array) {
            // For arrays, we need to count each element as a separate field
            self.fields
                .iter()
                .map(|f| {
                    if f.is_array {
                        f.array_size.unwrap_or(0)
                    } else {
                        1
                    }
                })
                .sum::<usize>() as u32
        } else {
            self.fields.len() as u32
        };

        if schema_field_count != field_count {
            return Err(format!(
                "Field count mismatch: schema has {schema_field_count} fields, but DBC has {field_count} fields"
            ));
        }

        let schema_record_size = self.record_size() as u32;
        if schema_record_size != record_size {
            return Err(format!(
                "Record size mismatch: schema defines {schema_record_size} bytes, but DBC has {record_size} bytes per record"
            ));
        }

        if let Some(index) = self.key_field_index {
            if index >= self.fields.len() {
                return Err(format!(
                    "Key field index out of bounds: {} (max: {})",
                    index,
                    self.fields.len() - 1
                ));
            }

            let key_field = &self.fields[index];
            match key_field.field_type {
                FieldType::UInt32 | FieldType::Int32 => {}
                _ => {
                    return Err(format!(
                        "Key field must be a 32-bit integer, but is {:?}",
                        key_field.field_type
                    ));
                }
            }

            if key_field.is_array {
                return Err("Key field cannot be an array".to_string());
            }
        }

        self.is_validated = true;
        Ok(())
    }
}
