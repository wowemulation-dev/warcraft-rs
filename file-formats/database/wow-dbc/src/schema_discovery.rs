//! Schema discovery functionality to auto-detect field types in DBC files.

use crate::{DbcHeader, Error, FieldType, Result, Schema, SchemaField, StringBlock, StringRef};
use std::collections::HashSet;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Confidence level for a field type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Low confidence (50-70%)
    Low,
    /// Medium confidence (70-90%)
    Medium,
    /// High confidence (90-100%)
    High,
}

/// Represents a discovered field type with confidence level
#[derive(Debug, Clone)]
pub struct DiscoveredField {
    /// The field type
    pub field_type: FieldType,
    /// Confidence level in the detection
    pub confidence: Confidence,
    /// Whether the field is potentially a key field
    pub is_key_candidate: bool,
    /// Whether the field is an array
    pub is_array: bool,
    /// Size of the array, if the field is an array
    pub array_size: Option<usize>,
    /// Sample values (for validation and debugging)
    pub sample_values: Vec<u32>,
}

/// Represents a complete discovered schema
#[derive(Debug, Clone)]
pub struct DiscoveredSchema {
    /// The discovered fields
    pub fields: Vec<DiscoveredField>,
    /// Key field index, if detected
    pub key_field_index: Option<usize>,
    /// Validation status of the schema
    pub is_valid: bool,
    /// Validation message, if any
    pub validation_message: Option<String>,
}

impl DiscoveredSchema {
    /// Convert a discovered schema to a regular schema
    pub fn to_schema(&self, name: &str) -> Schema {
        let mut schema = Schema::new(name);

        for (i, field) in self.fields.iter().enumerate() {
            let field_name = format!("field_{}", i);

            if field.is_array {
                schema.add_field(SchemaField::new_array(
                    field_name,
                    field.field_type,
                    field.array_size.unwrap_or(0),
                ));
            } else {
                schema.add_field(SchemaField::new(field_name, field.field_type));
            }
        }

        if let Some(key_index) = self.key_field_index {
            schema.set_key_field_index(key_index);
        }

        schema
    }
}

/// Schema discoverer for DBC files
#[derive(Debug)]
pub struct SchemaDiscoverer<'a> {
    /// The DBC header
    header: &'a DbcHeader,
    /// The raw data of the DBC file
    data: &'a [u8],
    /// The string block
    string_block: &'a StringBlock,
    /// Maximum number of records to analyze (0 = all)
    max_records: u32,
    /// Whether to validate string references
    validate_strings: bool,
    /// Whether to detect arrays
    detect_arrays: bool,
    /// Whether to detect the key field
    detect_key: bool,
}

impl<'a> SchemaDiscoverer<'a> {
    /// Create a new schema discoverer
    pub fn new(header: &'a DbcHeader, data: &'a [u8], string_block: &'a StringBlock) -> Self {
        Self {
            header,
            data,
            string_block,
            max_records: 100, // Default sample size
            validate_strings: true,
            detect_arrays: true,
            detect_key: true,
        }
    }

    /// Set the maximum number of records to analyze
    pub fn with_max_records(mut self, max_records: u32) -> Self {
        self.max_records = max_records;
        self
    }

    /// Set whether to validate string references
    pub fn with_validate_strings(mut self, validate_strings: bool) -> Self {
        self.validate_strings = validate_strings;
        self
    }

    /// Set whether to detect arrays
    pub fn with_detect_arrays(mut self, detect_arrays: bool) -> Self {
        self.detect_arrays = detect_arrays;
        self
    }

    /// Set whether to detect the key field
    pub fn with_detect_key(mut self, detect_key: bool) -> Self {
        self.detect_key = detect_key;
        self
    }

    /// Discover the schema of the DBC file
    pub fn discover(&self) -> Result<DiscoveredSchema> {
        // Determine how many records to analyze
        let records_to_analyze =
            if self.max_records == 0 || self.max_records > self.header.record_count {
                self.header.record_count
            } else {
                self.max_records
            };

        // Skip the header
        let mut cursor = Cursor::new(self.data);
        cursor.seek(SeekFrom::Start(DbcHeader::SIZE as u64))?;

        // Fetch raw record data for analysis
        let mut record_data = Vec::with_capacity(records_to_analyze as usize);
        for _ in 0..records_to_analyze {
            let mut record = Vec::with_capacity(self.header.record_size as usize);
            let mut buffer = vec![0u8; self.header.record_size as usize];
            cursor.read_exact(&mut buffer)?;

            // Parse into u32 values (most DBC fields are 4 bytes)
            let mut record_cursor = Cursor::new(&buffer);
            for _ in 0..self.header.field_count {
                let mut buf = [0u8; 4];
                record_cursor.read_exact(&mut buf)?;
                let value = u32::from_le_bytes(buf);
                record.push(value);
            }

            record_data.push(record);
        }

        // Analyze the record data to discover field types
        let discovered_fields = self.analyze_fields(&record_data)?;

        // Detect the key field
        let key_field_index = if self.detect_key {
            self.detect_key_field(&record_data, &discovered_fields)
        } else {
            None
        };

        // Validate the discovered schema
        let (is_valid, validation_message) = self.validate_schema(&discovered_fields)?;

        Ok(DiscoveredSchema {
            fields: discovered_fields,
            key_field_index,
            is_valid,
            validation_message,
        })
    }

    /// Analyze all fields to determine their types
    fn analyze_fields(&self, record_data: &[Vec<u32>]) -> Result<Vec<DiscoveredField>> {
        let mut discovered_fields = Vec::with_capacity(self.header.field_count as usize);

        // If no records to analyze, return empty fields
        if record_data.is_empty() {
            return Ok(discovered_fields);
        }

        // Analyze each field
        for field_index in 0..self.header.field_count as usize {
            // Extract values for this field from all analyzed records
            let field_values: Vec<u32> = record_data
                .iter()
                .map(|record| record[field_index])
                .collect();

            // Analyze field values to determine type
            let discovered_field = self.analyze_field(field_index, &field_values)?;
            discovered_fields.push(discovered_field);
        }

        // Detect arrays if configured
        if self.detect_arrays {
            self.detect_array_fields(&mut discovered_fields);
        }

        Ok(discovered_fields)
    }

    /// Analyze a single field to determine its type
    fn analyze_field(&self, _field_index: usize, values: &[u32]) -> Result<DiscoveredField> {
        // Check if all values are 0 or 1 (boolean)
        let is_bool = values.iter().all(|&value| value == 0 || value == 1);

        // Check if any values are in the string block range
        let possible_string_refs = values
            .iter()
            .filter(|&&value| value > 0 && value < self.string_block.size() as u32)
            .count();

        let is_string_ref = possible_string_refs > 0 && possible_string_refs >= values.len() / 2; // At least half of values should be potential strings

        // Validate string references if configured
        let is_valid_string_ref = if self.validate_strings && is_string_ref {
            // Check if the string references point to valid strings
            let valid_strings = values
                .iter()
                .filter(|&&value| {
                    if value == 0 {
                        // Empty string is valid
                        return true;
                    }

                    // Check if the value points to a valid string
                    self.string_block.get_string(StringRef::new(value)).is_ok()
                })
                .count();

            valid_strings >= values.len() * 3 / 4 // At least 75% of values should be valid strings
        } else {
            false
        };

        // Check for potential key field
        let is_key_candidate = self.is_potential_key(values);

        // Check if the values fit in different integer ranges
        let min_value = values.iter().copied().min().unwrap_or(0);
        let max_value = values.iter().copied().max().unwrap_or(0);

        let fits_uint8 = max_value <= 0xFF;
        let fits_int8 = min_value >= 0x80 && max_value <= 0x7F;
        let fits_uint16 = max_value <= 0xFFFF;
        let fits_int16 = min_value >= 0x8000 && max_value <= 0x7FFF;

        // Check if the values could be floating point
        let could_be_float = values.iter().any(|&value| {
            // Check if the bit pattern could represent a reasonable float
            let float_val = f32::from_bits(value);
            float_val.is_finite()
                && !float_val.is_subnormal()
                && (float_val.abs() < 0.00001 || float_val.abs() > 0.00001)
        });

        // Determine the most likely field type
        let (field_type, confidence) = if is_valid_string_ref {
            (FieldType::String, Confidence::High)
        } else if is_string_ref {
            (FieldType::String, Confidence::Medium)
        } else if is_bool {
            (FieldType::Bool, Confidence::High)
        } else if fits_uint8 {
            (FieldType::UInt8, Confidence::Medium)
        } else if fits_int8 {
            (FieldType::Int8, Confidence::Medium)
        } else if fits_uint16 {
            (FieldType::UInt16, Confidence::Medium)
        } else if fits_int16 {
            (FieldType::Int16, Confidence::Medium)
        } else if could_be_float {
            (FieldType::Float32, Confidence::Medium)
        } else if values.iter().any(|&v| v > 0x7FFFFFFF) {
            // If any value is larger than i32::MAX, it's probably unsigned
            (FieldType::UInt32, Confidence::High)
        } else {
            // Default to Int32
            (FieldType::Int32, Confidence::Low)
        };

        // Collect sample values for validation and debugging
        let sample_values = values.iter().take(10).copied().collect();

        Ok(DiscoveredField {
            field_type,
            confidence,
            is_key_candidate,
            is_array: false,  // Will be set later if detected
            array_size: None, // Will be set later if detected
            sample_values,
        })
    }

    /// Check if a field could be a key field
    fn is_potential_key(&self, values: &[u32]) -> bool {
        // A key field should have unique, non-zero values
        if values.is_empty() {
            return false;
        }

        // Check if all values are unique
        let unique_values: HashSet<u32> = values.iter().copied().collect();
        if unique_values.len() != values.len() {
            return false;
        }

        // Check if all values are non-zero
        if values.iter().any(|&value| value == 0) {
            return false;
        }

        // Check if values are sequential or mostly sequential
        let min_value = *values.iter().min().unwrap();
        let max_value = *values.iter().max().unwrap();

        // Sequential or nearly sequential values are good candidates
        let range = max_value - min_value + 1;
        if range as usize <= values.len() * 2 {
            return true;
        }

        // Check if values are reasonably dense in their range
        let density = values.len() as f32 / range as f32;
        density > 0.2 // At least 20% of the range is filled
    }

    /// Detect array fields based on patterns in field types
    fn detect_array_fields(&self, fields: &mut Vec<DiscoveredField>) {
        if fields.len() <= 1 {
            return; // No arrays possible with one or zero fields
        }

        // Look for repeating patterns of field types
        for array_size in 2..=10 {
            // Try different array sizes
            if fields.len() % array_size != 0 {
                continue; // Fields must divide evenly by array size
            }

            let potential_arrays = fields.len() / array_size;
            let mut is_array_pattern = true;

            for a in 0..potential_arrays {
                let base_type = fields[a * array_size].field_type;

                // Check if all fields in the potential array have the same type
                for i in 1..array_size {
                    if fields[a * array_size + i].field_type != base_type {
                        is_array_pattern = false;
                        break;
                    }
                }

                if !is_array_pattern {
                    break;
                }
            }

            if is_array_pattern {
                // Mark fields as array elements
                let mut new_fields = Vec::with_capacity(potential_arrays);

                for a in 0..potential_arrays {
                    let mut base_field = fields[a * array_size].clone();
                    base_field.is_array = true;
                    base_field.array_size = Some(array_size);
                    new_fields.push(base_field);
                }

                *fields = new_fields;
                return; // Successfully detected arrays
            }
        }
    }

    /// Detect the key field
    fn detect_key_field(
        &self,
        record_data: &[Vec<u32>],
        fields: &[DiscoveredField],
    ) -> Option<usize> {
        // Find candidates based on field analysis
        let mut candidates: Vec<usize> = fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.is_key_candidate)
            .map(|(i, _)| i)
            .collect();

        // If no candidates, check for fields with ascending values
        if candidates.is_empty() {
            for (field_index, field) in fields.iter().enumerate() {
                if field.field_type != FieldType::UInt32 && field.field_type != FieldType::Int32 {
                    continue;
                }

                // Get values for this field
                let values: Vec<u32> = record_data
                    .iter()
                    .map(|record| record[field_index])
                    .collect();

                // Check if values are always increasing
                let mut is_increasing = true;
                for i in 1..values.len() {
                    if values[i] <= values[i - 1] {
                        is_increasing = false;
                        break;
                    }
                }

                if is_increasing {
                    candidates.push(field_index);
                }
            }
        }

        // If still no candidates, pick the first UInt32 field
        if candidates.is_empty() {
            for (field_index, field) in fields.iter().enumerate() {
                if field.field_type == FieldType::UInt32 {
                    candidates.push(field_index);
                    break;
                }
            }
        }

        // If only one candidate, return it
        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        // If multiple candidates, prefer the first field
        candidates.sort();
        candidates.first().copied()
    }

    /// Validate the discovered schema
    fn validate_schema(&self, fields: &[DiscoveredField]) -> Result<(bool, Option<String>)> {
        // Check if the field count matches
        let field_count = if fields.iter().any(|f| f.is_array) {
            fields
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
            fields.len() as u32
        };

        if field_count != self.header.field_count {
            return Ok((
                false,
                Some(format!(
                    "Field count mismatch: schema has {} fields, but DBC has {} fields",
                    field_count, self.header.field_count
                )),
            ));
        }

        // Calculate the record size based on field types
        let record_size = fields
            .iter()
            .map(|f| {
                if f.is_array {
                    f.field_type.size() * f.array_size.unwrap_or(0)
                } else {
                    f.field_type.size()
                }
            })
            .sum::<usize>() as u32;

        // Check if the record size matches
        if record_size != self.header.record_size {
            return Ok((
                false,
                Some(format!(
                    "Record size mismatch: schema defines {} bytes, but DBC has {} bytes per record",
                    record_size, self.header.record_size
                )),
            ));
        }

        Ok((true, None))
    }

    /// Generate a schema from the discovered fields with automatic field naming
    pub fn generate_schema(&self, name: &str) -> Result<Schema> {
        let discovered = self.discover()?;
        if !discovered.is_valid {
            return Err(Error::SchemaValidation(
                discovered
                    .validation_message
                    .unwrap_or_else(|| "Invalid discovered schema".to_string()),
            ));
        }

        let mut schema = Schema::new(name);

        // Add fields with meaningful names based on type and position
        for (i, field) in discovered.fields.iter().enumerate() {
            // Use field index as a base for field names
            let field_name = if field.is_key_candidate {
                "ID".to_string()
            } else {
                match field.field_type {
                    FieldType::String => format!("String_{}", i),
                    FieldType::Float32 => format!("Float_{}", i),
                    FieldType::Bool => format!("Flag_{}", i),
                    FieldType::UInt32 | FieldType::Int32 => format!("Value_{}", i),
                    FieldType::UInt8 | FieldType::Int8 => format!("Byte_{}", i),
                    FieldType::UInt16 | FieldType::Int16 => format!("Short_{}", i),
                }
            };

            if field.is_array {
                schema.add_field(SchemaField::new_array(
                    field_name,
                    field.field_type,
                    field.array_size.unwrap_or(0),
                ));
            } else {
                schema.add_field(SchemaField::new(field_name, field.field_type));
            }
        }

        // Set the key field if detected
        if let Some(key_index) = discovered.key_field_index {
            schema.set_key_field_index(key_index);
        }

        Ok(schema)
    }
}
