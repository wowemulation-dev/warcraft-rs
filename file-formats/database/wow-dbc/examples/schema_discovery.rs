//! # Schema Discovery Example
//!
//! This example demonstrates how to use the schema discovery functionality to automatically
//! detect field types in DBC files when you don't have a predefined schema.
//!
//! ## Usage
//! ```bash
//! cargo run --example schema_discovery
//! ```
//!
//! The schema discovery feature analyzes the data patterns in the DBC file to make
//! educated guesses about field types (integers, floats, strings, etc.).

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_dbc::{Confidence, DbcParser, SchemaDiscoverer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a test DBC file from the test data
    let dbc_path = Path::new("test-data/3.3.5a/DBFilesClient/Spell.dbc");

    // Check if file exists, if not, use a simpler example
    let dbc_path = if dbc_path.exists() {
        dbc_path
    } else {
        Path::new("test-data/1.12.1/DBFilesClient/ItemClass.dbc")
    };

    // Open the DBC file
    println!("\n=== Schema Discovery Example ===");
    println!("Opening DBC file: {}", dbc_path.display());
    let file = File::open(dbc_path).inspect_err(|_| {
        eprintln!("Error: Could not open file. Make sure you run this from the project root.");
    })?;
    let mut reader = BufReader::new(file);

    // Parse the DBC file
    println!("Parsing DBC file...");
    let parser = DbcParser::parse(&mut reader)?;

    // Print basic header information
    let header = parser.header();
    println!("DBC header information:");
    println!("  Record count: {}", header.record_count);
    println!("  Field count: {}", header.field_count);
    println!("  Record size: {} bytes", header.record_size);
    println!("  String block size: {} bytes", header.string_block_size);

    // Parse records to get the string block
    println!("Parsing records...");
    let record_set = parser.parse_records()?;

    // Create schema discoverer
    println!("Setting up schema discoverer...");
    let discoverer = SchemaDiscoverer::new(header, parser.data(), record_set.string_block())
        .with_max_records(100) // Analyze up to 100 records
        .with_validate_strings(true) // Validate string references
        .with_detect_arrays(true) // Try to detect arrays
        .with_detect_key(true); // Try to identify key field

    // Discover the schema
    println!("Discovering schema...");
    let discovered_schema = discoverer.discover()?;

    // Print discovery results
    println!("\nDiscovered schema:");
    println!("  Schema is valid: {}", discovered_schema.is_valid);

    if let Some(msg) = &discovered_schema.validation_message {
        println!("  Validation message: {}", msg);
    }

    // Print discovered fields
    println!("\nDiscovered fields:");
    for (i, field) in discovered_schema.fields.iter().enumerate() {
        let confidence_str = match field.confidence {
            Confidence::Low => "Low",
            Confidence::Medium => "Medium",
            Confidence::High => "High",
        };

        println!(
            "  Field {}: {:?} (Confidence: {})",
            i, field.field_type, confidence_str
        );

        if field.is_array {
            println!("    Is array with size: {}", field.array_size.unwrap_or(0));
        }

        if field.is_key_candidate {
            println!("    Is key candidate");
        }

        println!(
            "    Sample values: {:?}",
            field.sample_values.iter().take(5).collect::<Vec<_>>()
        );
    }

    // Print key field if detected
    if let Some(key_index) = discovered_schema.key_field_index {
        println!("\nDetected key field: {}", key_index);
    } else {
        println!("\nNo key field detected");
    }

    // Generate a schema with automatic field naming
    println!("\nGenerating schema with field names...");
    let file_stem = dbc_path.file_stem().unwrap_or_default().to_string_lossy();
    let schema = discoverer.generate_schema(&file_stem)?;

    // Print the generated schema
    println!("Generated schema:");
    println!("  Name: {}", schema.name);
    println!("  Fields:");

    for (i, field) in schema.fields.iter().enumerate() {
        if field.is_array {
            println!(
                "    {}: {:?}[{}]",
                field.name,
                field.field_type,
                field.array_size.unwrap_or(0)
            );
        } else {
            println!("    {}: {:?}", field.name, field.field_type);
        }

        if Some(i) == schema.key_field_index {
            println!("      (Key field)");
        }
    }

    // Demonstrate using the discovered schema to parse records
    println!("\nParsing records with the discovered schema...");
    let parser_with_schema = parser.with_schema(schema)?;
    let record_set_with_schema = parser_with_schema.parse_records()?;

    // Print the first record using the schema
    if let Some(record) = record_set_with_schema.get_record(0) {
        println!("First record values:");

        for (i, value) in record.values().iter().enumerate() {
            println!("  Field {}: {:?}", i, value);
        }
    }

    println!("\nSchema discovery completed successfully!");
    Ok(())
}
