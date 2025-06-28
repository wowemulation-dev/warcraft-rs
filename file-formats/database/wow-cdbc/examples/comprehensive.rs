//! # Comprehensive Example
//!
//! This example demonstrates all features of the wow-cdbc parser library, including:
//! - Core parsing functionality
//! - Schema loading from YAML files
//! - Memory-mapped file access
//! - Parallel processing
//! - Export to JSON and CSV formats
//! - Schema discovery
//!
//! ## Usage
//! ```bash
//! # Run with all features enabled
//! cargo run --example comprehensive --all-features
//!
//! # Run with specific features
//! cargo run --example comprehensive --features "yaml csv_export"
//! ```

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use wow_cdbc::{DbcParser, FieldType, LazyDbcParser, Schema, SchemaField};

#[cfg(feature = "serde")]
use wow_cdbc::export_to_json;

#[cfg(feature = "mmap")]
use wow_cdbc::MmapDbcFile;

#[cfg(feature = "yaml")]
use wow_cdbc::SchemaDefinition;

#[cfg(feature = "csv_export")]
use wow_cdbc::export_to_csv;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== WoW DBC Parser Comprehensive Example ===");
    println!("This example demonstrates various features of the DBC parser");
    println!("============================================\n");

    // Core features (always available)
    parse_with_code_schema()?;
    parse_with_lazy_loading()?;

    // Feature-gated functionality
    #[cfg(feature = "yaml")]
    parse_with_yaml_schema()?;

    #[cfg(feature = "mmap")]
    parse_with_memory_mapping()?;

    #[cfg(feature = "parallel")]
    parse_with_parallel_processing()?;

    #[cfg(feature = "serde")]
    export_to_json_example()?;

    #[cfg(feature = "csv_export")]
    export_to_csv_example()?;

    // Schema discovery
    schema_discovery_example()?;

    println!("All examples completed successfully!");
    Ok(())
}

fn parse_with_code_schema() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Parsing with code-defined schema");
    println!("   ================================");

    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/ItemClass.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data not found\n");
        return Ok(());
    }

    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);

    let start = Instant::now();
    let parser = DbcParser::parse(&mut reader)?;
    println!("   Header parsed in {:?}", start.elapsed());
    println!("   - Records: {}", parser.header().record_count);
    println!("   - Fields: {}", parser.header().field_count);
    println!("   - Record size: {} bytes", parser.header().record_size);

    // Define schema
    let mut schema = Schema::new("ItemClass");
    schema.add_field(SchemaField::new("ClassID", FieldType::Int32));
    schema.add_field(SchemaField::new("SubClassID", FieldType::Int32));
    schema.add_field(SchemaField::new("Flags", FieldType::Int32));
    schema.add_field(SchemaField::new("ClassName", FieldType::String));
    schema.set_key_field("ClassID");

    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;
    println!("   Records parsed in {:?}", start.elapsed());

    // Display some records
    println!("   Sample records:");
    for i in 0..3.min(record_set.len()) {
        let record = record_set.get_record(i).unwrap();
        let class_id = record.get_value_by_name("ClassID").unwrap();
        let class_name = record.get_value_by_name("ClassName").unwrap();
        println!("   - {class_id}: {class_name}");
    }

    println!();
    Ok(())
}

fn parse_with_lazy_loading() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Lazy loading for large files");
    println!("   =============================");

    let dbc_path = Path::new("test-data/2.4.3/DBFilesClient/Spell.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data not found\n");
        return Ok(());
    }

    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);

    let parser = DbcParser::parse(&mut reader)?;
    let record_count = parser.header().record_count;
    let field_count = parser.header().field_count;
    println!("   Processing {record_count} records lazily");

    // Create minimal schema for demonstration
    let mut schema = Schema::new("Spell");
    for i in 0..field_count as usize {
        schema.add_field(SchemaField::new(format!("Field{i}"), FieldType::Int32));
    }

    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;

    let string_block = std::sync::Arc::new(record_set.string_block().clone());
    let lazy_parser = LazyDbcParser::new(
        parser.data(),
        parser.header(),
        parser.schema(),
        string_block,
    );

    // Process specific records without loading all into memory
    let indices = vec![0, 100, 500, 1000];
    println!("   Accessing specific records:");
    for idx in indices {
        if idx < record_count {
            let record = lazy_parser.get_record(idx)?;
            println!("   - Record {}: {} fields", idx, record.len());
        }
    }

    println!();
    Ok(())
}

#[cfg(feature = "yaml")]
fn parse_with_yaml_schema() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Parsing with YAML schema");
    println!("   ========================");

    let schema_path = Path::new("schemas/Spell.yaml");
    let dbc_path = Path::new("test-data/2.4.3/DBFilesClient/Spell.dbc");

    if !schema_path.exists() || !dbc_path.exists() {
        println!("   Skipping: schema or test data not found\n");
        return Ok(());
    }

    // Load schema from YAML
    let schema_def = SchemaDefinition::from_yaml(schema_path)?;
    let schema = schema_def.to_schema()?;
    println!("   Loaded schema: {}", schema.name);
    println!("   - Fields: {}", schema.fields.len());
    if let Some(key_index) = schema.key_field_index {
        println!("   - Key field index: {key_index}");
    }

    // Parse DBC with schema
    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);
    let parser = DbcParser::parse(&mut reader)?;
    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;

    println!("   Parsed {} records successfully", record_set.len());

    // Show a sample spell
    if let Some(record) = record_set.get_record(0) {
        if let Some(wow_cdbc::Value::UInt32(id)) = record.get_value_by_name("ID") {
            println!("   First spell ID: {id}");
        }
    }

    println!();
    Ok(())
}

#[cfg(feature = "mmap")]
fn parse_with_memory_mapping() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Memory-mapped file access");
    println!("   =========================");

    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/WorldMapArea.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data not found\n");
        return Ok(());
    }

    let start = Instant::now();
    let mmap_file = MmapDbcFile::open(dbc_path)?;
    let file_size = mmap_file.as_slice().len();
    println!(
        "   Memory-mapped {} bytes in {:?}",
        file_size,
        start.elapsed()
    );

    let parser = DbcParser::parse_bytes(mmap_file.as_slice())?;
    println!("   - Records: {}", parser.header().record_count);
    println!("   - Memory mapping provides zero-copy access");

    // Create simple schema
    let mut schema = Schema::new("WorldMapArea");
    let field_count = parser.header().field_count as usize;
    for i in 0..field_count {
        schema.add_field(SchemaField::new(format!("Field{i}"), FieldType::Int32));
    }

    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;
    println!("   Parsed {} records via memory mapping", record_set.len());

    println!();
    Ok(())
}

#[cfg(feature = "parallel")]
fn parse_with_parallel_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. Parallel record processing");
    println!("   ==========================");

    let dbc_path = Path::new("test-data/3.3.5a/DBFilesClient/Spell.dbc");
    let dbc_path = if dbc_path.exists() {
        dbc_path
    } else {
        // Try alternative file
        let alt_path = Path::new("test-data/1.12.1/DBFilesClient/Talent.dbc");
        if !alt_path.exists() {
            println!("   Skipping: test data not found\n");
            return Ok(());
        }
        alt_path
    };

    let file = std::fs::read(dbc_path)?;
    let parser = DbcParser::parse_bytes(&file)?;

    // Create schema
    let mut schema = Schema::new("Records");
    let field_count = parser.header().field_count as usize;
    for i in 0..field_count {
        schema.add_field(SchemaField::new(format!("Field{i}"), FieldType::Int32));
    }

    let parser = parser.with_schema(schema)?;

    // Parse normally for comparison
    let start = Instant::now();
    let _record_set = parser.parse_records()?;
    let sequential_time = start.elapsed();

    // Parse in parallel
    let start = Instant::now();
    let header = parser.header();
    let string_block = std::sync::Arc::new(parser.parse_records()?.string_block().clone());

    let records = wow_cdbc::parse_records_parallel(&file, header, parser.schema(), string_block)?;
    let parallel_time = start.elapsed();

    println!("   Sequential parsing: {sequential_time:?}");
    println!("   Parallel parsing: {parallel_time:?}");
    if parallel_time.as_secs_f64() > 0.0 {
        println!(
            "   Speedup: {:.2}x",
            sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
        );
    }
    println!("   Processed {} records", records.len());

    println!();
    Ok(())
}

#[cfg(feature = "serde")]
fn export_to_json_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. Export to JSON");
    println!("   ===============");

    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/ItemBagFamily.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data not found\n");
        return Ok(());
    }

    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);
    let parser = DbcParser::parse(&mut reader)?;

    // Create schema
    let mut schema = Schema::new("ItemBagFamily");
    schema.add_field(SchemaField::new("ID", FieldType::Int32));
    schema.add_field(SchemaField::new("Name", FieldType::String));
    schema.set_key_field("ID");

    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;

    // Export to JSON
    let json_path = "item_bag_family.json";
    let json_file = File::create(json_path)?;
    export_to_json(&record_set, json_file)?;
    println!("   Exported {} records to {}", record_set.len(), json_path);

    // Show sample of JSON content
    let json_content = std::fs::read_to_string(json_path)?;
    let lines: Vec<&str> = json_content.lines().take(5).collect();
    println!("   Sample JSON output:");
    for line in lines {
        println!("   {line}");
    }
    if json_content.lines().count() > 5 {
        println!("   ...");
    }

    // Clean up
    std::fs::remove_file(json_path)?;

    println!();
    Ok(())
}

#[cfg(feature = "csv_export")]
fn export_to_csv_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("7. Export to CSV");
    println!("   ==============");

    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/SkillLineCategory.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data not found\n");
        return Ok(());
    }

    let file = File::open(dbc_path)?;
    let mut reader = BufReader::new(file);
    let parser = DbcParser::parse(&mut reader)?;

    // Create schema
    let mut schema = Schema::new("SkillLineCategory");
    schema.add_field(SchemaField::new("ID", FieldType::Int32));
    schema.add_field(SchemaField::new("Name", FieldType::String));
    schema.add_field(SchemaField::new("SortOrder", FieldType::Int32));
    schema.set_key_field("ID");

    let parser = parser.with_schema(schema)?;
    let record_set = parser.parse_records()?;

    // Export to CSV
    let csv_path = "skill_categories.csv";
    let csv_file = File::create(csv_path)?;
    export_to_csv(&record_set, csv_file)?;
    println!("   Exported {} records to {}", record_set.len(), csv_path);

    // Show sample of CSV content
    let csv_content = std::fs::read_to_string(csv_path)?;
    let lines: Vec<&str> = csv_content.lines().take(5).collect();
    println!("   Sample CSV output:");
    for line in lines {
        println!("   {line}");
    }

    // Clean up
    std::fs::remove_file(csv_path)?;

    println!();
    Ok(())
}

fn schema_discovery_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("8. Schema discovery");
    println!("   =================");

    let dbc_path = Path::new("test-data/1.12.1/DBFilesClient/AreaPOI.dbc");
    if !dbc_path.exists() {
        println!("   Skipping: test data not found\n");
        return Ok(());
    }

    let file = std::fs::read(dbc_path)?;
    let parser = DbcParser::parse_bytes(&file)?;
    let record_set = parser.parse_records()?;

    // Skip the header in the data slice
    let data_start = wow_cdbc::DbcHeader::SIZE;
    let data_end =
        data_start + (parser.header().record_count * parser.header().record_size) as usize;
    let record_data = &file[data_start..data_end];

    // Discover schema
    let discoverer =
        wow_cdbc::SchemaDiscoverer::new(parser.header(), record_data, record_set.string_block());

    let discovered_schema = discoverer.discover()?;
    println!("   Discovered {} fields:", discovered_schema.fields.len());

    for (idx, field) in discovered_schema.fields.iter().enumerate().take(5) {
        let confidence_str = match field.confidence {
            wow_cdbc::Confidence::High => "high",
            wow_cdbc::Confidence::Medium => "medium",
            wow_cdbc::Confidence::Low => "low",
        };
        println!(
            "   - Field {}: {:?} (confidence: {})",
            idx, field.field_type, confidence_str
        );
    }

    if discovered_schema.fields.len() > 5 {
        println!(
            "   ... and {} more fields",
            discovered_schema.fields.len() - 5
        );
    }

    println!();
    Ok(())
}
