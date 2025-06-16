//! # Comprehensive Simple Example
//!
//! This example demonstrates the core features of the wow-cdbc parser library:
//! - Parsing DBC files with schemas defined in code
//! - Using lazy loading for efficient memory usage with large files
//! - Basic DBC file operations
//!
//! ## Usage
//! ```bash
//! cargo run --example comprehensive_simple
//! ```

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use wow_cdbc::{DbcParser, FieldType, LazyDbcParser, Schema, SchemaField, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== WoW DBC Parser - Core Features Demo ===");
    println!("This example demonstrates the core features of the DBC parser\n");

    // 1. Parse a DBC file using a schema defined in code
    parse_with_code_schema()?;

    // 2. Use lazy loading for large files
    parse_with_lazy_loading()?;

    // 3. Parse without schema
    parse_without_schema()?;

    Ok(())
}

fn parse_with_code_schema() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Parsing with code-defined schema");
    println!("   --------------------------------");

    // Use a test DBC file
    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/ItemClass.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data file not found");
        println!("   Run this example from the project root directory\n");
        return Ok(());
    }

    // Open the DBC file
    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);

    // Parse the header
    let parser = DbcParser::parse(&mut reader)?;
    println!("   Record count: {}", parser.header().record_count);
    println!("   Field count: {}", parser.header().field_count);

    // Define the schema for ItemClass.dbc
    let mut schema = Schema::new("ItemClass");
    schema.add_field(SchemaField::new("ClassID", FieldType::Int32));
    schema.add_field(SchemaField::new("SubClassID", FieldType::Int32));
    schema.add_field(SchemaField::new("Flags", FieldType::Int32));
    schema.add_field(SchemaField::new("ClassName", FieldType::String));
    schema.set_key_field("ClassID");

    // Apply the schema and parse records
    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;

    println!("   Parsed {} records", record_set.len());

    // Access records by index
    if let Some(record) = record_set.get_record(0) {
        if let Some(Value::Int32(id)) = record.get_value_by_name("ClassID") {
            println!("   First record ClassID: {}", id);
        }
    }

    // Access records by key
    if let Some(record) = record_set.get_record_by_key(2) {
        if let Some(Value::StringRef(name_ref)) = record.get_value_by_name("ClassName") {
            let name = record_set.get_string(*name_ref)?;
            println!("   Class 2 name: {}", name);
        }
    }

    // Display all records
    println!("   All item classes:");
    for i in 0..record_set.len() {
        if let Some(record) = record_set.get_record(i) {
            if let (Some(Value::Int32(id)), Some(Value::StringRef(name_ref))) = (
                record.get_value_by_name("ClassID"),
                record.get_value_by_name("ClassName"),
            ) {
                let name = record_set.get_string(*name_ref)?;
                println!("     - Class {}: {}", id, name);
            }
        }
    }

    println!();
    Ok(())
}

fn parse_with_lazy_loading() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Lazy loading example");
    println!("   -------------------");

    // Use a larger test DBC file
    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/Talent.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data file not found\n");
        return Ok(());
    }

    // Open the DBC file
    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);

    // Parse the header
    let parser = DbcParser::parse(&mut reader)?;

    // Define a simple schema for demonstration
    let mut schema = Schema::new("Talent");
    // We'll define a minimal schema - just enough fields to demonstrate
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("TabID", FieldType::UInt32));
    schema.add_field(SchemaField::new("Row", FieldType::UInt32));
    schema.add_field(SchemaField::new("Column", FieldType::UInt32));
    // Add remaining fields as Int32 for simplicity
    let field_count = parser.header().field_count as usize;
    for i in 4..field_count {
        schema.add_field(SchemaField::new(format!("Field{}", i), FieldType::Int32));
    }
    schema.set_key_field("ID");

    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;

    // Create lazy parser
    let string_block = std::sync::Arc::new(record_set.string_block().clone());
    let start = Instant::now();

    let lazy_parser = LazyDbcParser::new(
        parser.data(),
        parser.header(),
        parser.schema(),
        string_block,
    );

    // Iterate over records on-demand
    println!("   Processing records lazily...");
    let mut count = 0;
    for record in lazy_parser.record_iterator() {
        let record = record?;
        count += 1;

        // Only process first few records for demo
        if count <= 3 {
            if let Some(Value::UInt32(id)) = record.get_value_by_name("ID") {
                println!("   - Record {}: ID = {}", count, id);
            }
        }
    }

    let elapsed = start.elapsed();
    println!("   Processed {} records in {:?}", count, elapsed);

    // Compare with direct access
    println!("   Direct access to specific records:");
    let indices = vec![0, 10, 20, 30];
    for idx in indices {
        if idx < parser.header().record_count {
            let record = lazy_parser.get_record(idx)?;
            if let Some(Value::UInt32(id)) = record.get_value_by_name("ID") {
                println!("   - Record {}: ID = {}", idx, id);
            }
        }
    }

    println!();
    Ok(())
}

fn parse_without_schema() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Parsing without schema");
    println!("   ----------------------");

    // Use a test DBC file
    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/StableSlotPrices.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data file not found\n");
        return Ok(());
    }

    // Open the DBC file
    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);

    // Parse the header
    let parser = DbcParser::parse(&mut reader)?;
    let header = parser.header();

    println!("   File information:");
    println!("   - Records: {}", header.record_count);
    println!("   - Fields: {}", header.field_count);
    println!("   - Record size: {} bytes", header.record_size);
    println!("   - String block size: {} bytes", header.string_block_size);

    // Parse records without schema
    let record_set = parser.parse_records()?;

    println!("   Parsed {} records without schema", record_set.len());

    // Display raw field data for first few records
    println!("   Raw data from first 3 records:");
    for i in 0..3.min(record_set.len()) {
        if let Some(record) = record_set.get_record(i) {
            print!("   Record {}: ", i);
            for (j, value) in record.values().iter().enumerate() {
                if j > 0 {
                    print!(", ");
                }
                print!("{}", value);
            }
            println!();
        }
    }

    println!();
    Ok(())
}
