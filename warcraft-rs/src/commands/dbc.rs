//! DBC database command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;
use wow_cdbc::{
    DbcParser, RecordSet, SchemaDefinition, SchemaDiscoverer, Value, export_to_csv, export_to_json,
};

#[cfg(feature = "yaml")]
use wow_cdbc::FieldType;

#[derive(Subcommand)]
pub enum DbcCommands {
    /// Show information about a DBC file
    Info {
        /// Path to the DBC file
        file: PathBuf,
    },

    /// Validate a DBC file against a schema
    Validate {
        /// Path to the DBC file
        file: PathBuf,

        /// Path to the schema YAML file
        #[arg(short, long)]
        schema: PathBuf,
    },

    /// List records in a DBC file
    List {
        /// Path to the DBC file
        file: PathBuf,

        /// Path to the schema YAML file (optional)
        #[arg(short, long)]
        schema: Option<PathBuf>,

        /// Maximum number of records to display
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    /// Export DBC data to various formats
    Export {
        /// Path to the DBC file
        file: PathBuf,

        /// Path to the schema YAML file
        #[arg(short, long)]
        schema: PathBuf,

        /// Output format (json, csv)
        #[arg(short, long, value_enum, default_value = "json")]
        format: ExportFormat,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze a DBC file for performance and structure
    Analyze {
        /// Path to the DBC file
        file: PathBuf,

        /// Path to the schema YAML file (optional)
        #[arg(short, long)]
        schema: Option<PathBuf>,

        /// Use memory-mapped file (requires mmap feature)
        #[arg(long)]
        mmap: bool,

        /// Use lazy loading (requires mmap feature)
        #[arg(long)]
        lazy: bool,

        /// Enable string caching for performance
        #[arg(long)]
        cache_strings: bool,

        /// Create sorted key map for binary search
        #[arg(long)]
        sorted_keys: bool,
    },

    /// Discover the schema of a DBC file through analysis
    Discover {
        /// Path to the DBC file
        file: PathBuf,

        /// Maximum number of records to analyze (0 = all)
        #[arg(short, long, default_value_t = 100)]
        max_records: u32,

        /// Whether to validate string references
        #[arg(long, default_value_t = true)]
        validate_strings: bool,

        /// Whether to detect array fields
        #[arg(long, default_value_t = true)]
        detect_arrays: bool,

        /// Whether to detect the key field
        #[arg(long, default_value_t = true)]
        detect_key: bool,

        /// Output schema as YAML format
        #[arg(short, long)]
        yaml: bool,

        /// Output file path for the generated schema
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
}

pub fn execute(command: DbcCommands) -> Result<()> {
    match command {
        DbcCommands::Info { file } => info_command(&file),
        DbcCommands::List {
            file,
            schema,
            limit,
        } => list_command(&file, schema.as_deref(), limit),
        DbcCommands::Export {
            file,
            schema,
            format,
            output,
        } => export_command(&file, &schema, format, output.as_deref()),
        DbcCommands::Analyze {
            file,
            schema,
            mmap,
            lazy,
            cache_strings,
            sorted_keys,
        } => analyze_command(
            &file,
            schema.as_deref(),
            mmap,
            lazy,
            cache_strings,
            sorted_keys,
        ),
        DbcCommands::Validate { file, schema } => validate_command(&file, &schema),
        DbcCommands::Discover {
            file,
            max_records,
            validate_strings,
            detect_arrays,
            detect_key,
            yaml,
            output,
        } => discover_command(
            &file,
            max_records,
            validate_strings,
            detect_arrays,
            detect_key,
            yaml,
            output.as_deref(),
        ),
    }
}

/// Display information about a DBC file
fn info_command(file: &Path) -> Result<()> {
    let dbc_file =
        File::open(file).with_context(|| format!("Failed to open DBC file: {}", file.display()))?;
    let mut reader = BufReader::new(dbc_file);

    let parser = DbcParser::parse(&mut reader)
        .with_context(|| format!("Failed to parse DBC file: {}", file.display()))?;
    let header = parser.header();

    println!("DBC File Information");
    println!("===================");
    println!();
    println!("File: {}", file.display());
    println!(
        "Magic: {} ({})",
        std::str::from_utf8(&header.magic).unwrap_or("Invalid"),
        header
            .magic
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!("Version: {:?}", parser.version());
    println!("Record Count: {}", header.record_count);
    println!("Field Count: {}", header.field_count);
    println!("Record Size: {} bytes", header.record_size);
    println!("String Block Size: {} bytes", header.string_block_size);
    println!("Total File Size: {} bytes", header.total_size());

    // Parse raw records (no schema) to show sample data
    let record_set = parser.parse_records().context("Failed to parse records")?;

    println!();
    println!("String Block Stats:");
    println!("  Offset: {}", header.string_block_offset());
    println!("  Size: {}", header.string_block_size);

    // Show sample record
    if let Some(record) = record_set.get_record(0) {
        println!();
        println!("Sample Record (First Record - Raw Values):");
        println!("=========================================");
        for (i, value) in record.values().iter().enumerate() {
            match value {
                Value::UInt32(v) => println!("  Field {i:2}: {v:10} (UInt32)"),
                Value::Int32(v) => println!("  Field {i:2}: {v:10} (Int32)"),
                Value::Float32(v) => println!("  Field {i:2}: {v:10.4} (Float32)"),
                Value::StringRef(v) => match record_set.get_string(*v) {
                    Ok(s) => println!("  Field {i:2}: \"{s}\" (String)"),
                    Err(_) => println!(
                        "  Field {:2}: <Invalid string ref: {}> (String)",
                        i,
                        v.offset()
                    ),
                },
                Value::Bool(v) => println!("  Field {i:2}: {v:10} (Bool)"),
                Value::UInt8(v) => println!("  Field {i:2}: {v:10} (UInt8)"),
                Value::Int8(v) => println!("  Field {i:2}: {v:10} (Int8)"),
                Value::UInt16(v) => println!("  Field {i:2}: {v:10} (UInt16)"),
                Value::Int16(v) => println!("  Field {i:2}: {v:10} (Int16)"),
                Value::Array(vals) => {
                    println!("  Field {:2}: Array[{}]", i, vals.len());
                    for (j, val) in vals.iter().enumerate().take(3) {
                        println!("            [{j}]: {val:?}");
                    }
                    if vals.len() > 3 {
                        println!("            ... {} more items", vals.len() - 3);
                    }
                }
            }
        }
    }

    Ok(())
}

/// List records from a DBC file
fn list_command(file: &Path, schema_path: Option<&Path>, limit: usize) -> Result<()> {
    let dbc_file =
        File::open(file).with_context(|| format!("Failed to open DBC file: {}", file.display()))?;
    let mut reader = BufReader::new(dbc_file);

    let mut parser = DbcParser::parse(&mut reader)
        .with_context(|| format!("Failed to parse DBC file: {}", file.display()))?;

    // Load schema if provided
    if let Some(schema_path) = schema_path {
        let schema_def = SchemaDefinition::from_yaml(schema_path).map_err(|e| {
            anyhow::anyhow!("Failed to load schema {}: {}", schema_path.display(), e)
        })?;
        let schema = schema_def
            .to_schema()
            .map_err(|e| anyhow::anyhow!("Failed to convert schema definition: {}", e))?;
        parser = parser
            .with_schema(schema)
            .context("Failed to apply schema")?;
    }

    let record_set = parser.parse_records().context("Failed to parse records")?;

    println!("DBC Records");
    println!("===========");
    println!("Total records: {}", record_set.len());
    println!("Showing first {} records:", limit.min(record_set.len()));
    println!();

    for i in 0..limit.min(record_set.len()) {
        if let Some(record) = record_set.get_record(i) {
            println!("Record {i}:");
            if let Some(schema) = record.schema() {
                // With schema - show field names
                for (field_idx, field) in schema.fields.iter().enumerate() {
                    if let Some(value) = record.get_value(field_idx) {
                        print!("  {}: ", field.name);
                        print_value(value, &record_set)?;
                    }
                }
            } else {
                // Without schema - show field indices
                for (field_idx, value) in record.values().iter().enumerate() {
                    print!("  Field {field_idx}: ");
                    print_value(value, &record_set)?;
                }
            }
            println!();
        }
    }

    if record_set.len() > limit {
        println!("... {} more records", record_set.len() - limit);
    }

    Ok(())
}

/// Export DBC data to file or stdout
fn export_command(
    file: &Path,
    schema_path: &Path,
    format: ExportFormat,
    output_path: Option<&Path>,
) -> Result<()> {
    let dbc_file =
        File::open(file).with_context(|| format!("Failed to open DBC file: {}", file.display()))?;
    let mut reader = BufReader::new(dbc_file);

    // Load schema
    let schema_def = SchemaDefinition::from_yaml(schema_path)
        .map_err(|e| anyhow::anyhow!("Failed to load schema {}: {}", schema_path.display(), e))?;
    let schema = schema_def
        .to_schema()
        .map_err(|e| anyhow::anyhow!("Failed to convert schema definition: {}", e))?;

    // Parse DBC file with schema
    let parser = DbcParser::parse(&mut reader)
        .with_context(|| format!("Failed to parse DBC file: {}", file.display()))?;
    let parser = parser
        .with_schema(schema)
        .context("Failed to apply schema")?;
    let record_set = parser.parse_records().context("Failed to parse records")?;

    // Export to output
    match output_path {
        Some(path) => {
            let output_file = File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?;
            let writer = BufWriter::new(output_file);

            match format {
                ExportFormat::Json => {
                    export_to_json(&record_set, writer).context("Failed to export to JSON")?;
                }
                ExportFormat::Csv => {
                    export_to_csv(&record_set, writer).context("Failed to export to CSV")?;
                }
            }

            println!(
                "Exported {} records to {}: {}",
                record_set.len(),
                match format {
                    ExportFormat::Json => "JSON",
                    ExportFormat::Csv => "CSV",
                },
                path.display()
            );
        }
        None => {
            // Export to stdout
            let stdout = io::stdout();
            let writer = stdout.lock();

            match format {
                ExportFormat::Json => {
                    export_to_json(&record_set, writer).context("Failed to export to JSON")?;
                }
                ExportFormat::Csv => {
                    export_to_csv(&record_set, writer).context("Failed to export to CSV")?;
                }
            }
        }
    }

    Ok(())
}

/// Analyze DBC file performance and structure
fn analyze_command(
    file: &Path,
    schema_path: Option<&Path>,
    mmap: bool,
    lazy: bool,
    cache_strings: bool,
    sorted_keys: bool,
) -> Result<()> {
    println!("Analyzing DBC file: {}", file.display());
    println!();

    let start = Instant::now();

    if mmap || lazy {
        anyhow::bail!(
            "Memory-mapped file support is not available in the current build.\n\
             The --mmap and --lazy flags require additional build configuration."
        );
    }

    let dbc_file =
        File::open(file).with_context(|| format!("Failed to open DBC file: {}", file.display()))?;
    let mut reader = BufReader::new(dbc_file);

    let parser = DbcParser::parse(&mut reader)
        .with_context(|| format!("Failed to parse DBC file: {}", file.display()))?;
    println!("  Header parsed in: {:?}", start.elapsed());

    let schema_obj = if let Some(schema_path) = schema_path {
        println!("Loading schema: {}", schema_path.display());
        let schema_def = SchemaDefinition::from_yaml(schema_path).map_err(|e| {
            anyhow::anyhow!("Failed to load schema {}: {}", schema_path.display(), e)
        })?;
        Some(
            schema_def
                .to_schema()
                .map_err(|e| anyhow::anyhow!("Failed to convert schema definition: {}", e))?,
        )
    } else {
        None
    };

    if let Some(schema_obj) = schema_obj {
        let parser = parser
            .with_schema(schema_obj)
            .context("Failed to apply schema")?;
        println!("  Schema applied in: {:?}", start.elapsed());

        let mut record_set = parser.parse_records().context("Failed to parse records")?;
        println!("  Records parsed in: {:?}", start.elapsed());

        if cache_strings {
            println!();
            println!("Enabling string caching");
            record_set.enable_string_caching();
            println!("  String cache built in: {:?}", start.elapsed());
        }

        if sorted_keys {
            println!();
            println!("Creating sorted key map");
            record_set
                .create_sorted_key_map()
                .context("Failed to create sorted key map")?;
            println!("  Sorted key map created in: {:?}", start.elapsed());
        }

        println!();
        println!("Total records: {}", record_set.len());
    } else {
        let record_set = parser.parse_records().context("Failed to parse records")?;
        println!("  Records parsed in: {:?}", start.elapsed());
        println!();
        println!("Total records: {}", record_set.len());
    }

    println!();
    println!("Total time: {:?}", start.elapsed());

    Ok(())
}

/// Validate a DBC file against a schema
fn validate_command(file: &Path, schema_path: &Path) -> Result<()> {
    let dbc_file =
        File::open(file).with_context(|| format!("Failed to open DBC file: {}", file.display()))?;
    let mut reader = BufReader::new(dbc_file);

    // Load schema
    let schema_def = SchemaDefinition::from_yaml(schema_path)
        .map_err(|e| anyhow::anyhow!("Failed to load schema {}: {}", schema_path.display(), e))?;
    let schema = schema_def
        .to_schema()
        .map_err(|e| anyhow::anyhow!("Failed to convert schema definition: {}", e))?;

    // Parse DBC file
    let parser = DbcParser::parse(&mut reader)
        .with_context(|| format!("Failed to parse DBC file: {}", file.display()))?;

    println!("Validating DBC file against schema");
    println!("==================================");
    println!();
    println!("File: {}", file.display());
    println!("Schema: {}", schema_path.display());
    println!();

    // Apply schema (this validates it)
    match parser.with_schema(schema) {
        Ok(parser) => {
            println!("✓ Schema validation passed");
            println!();

            // Try to parse records to ensure they can be read
            println!("Parsing records...");
            match parser.parse_records() {
                Ok(record_set) => {
                    println!("✓ Successfully parsed {} records", record_set.len());

                    // Validate string references
                    let mut invalid_strings = 0;
                    for i in 0..record_set.len() {
                        if let Some(record) = record_set.get_record(i) {
                            for value in record.values() {
                                if let Value::StringRef(str_ref) = value {
                                    if record_set.get_string(*str_ref).is_err() {
                                        invalid_strings += 1;
                                    }
                                }
                            }
                        }
                    }

                    if invalid_strings > 0 {
                        println!("⚠ Found {invalid_strings} invalid string references");
                    } else {
                        println!("✓ All string references are valid");
                    }
                }
                Err(e) => {
                    println!("✗ Failed to parse records: {e}");
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            println!("✗ Schema validation failed: {e}");
            return Err(anyhow::anyhow!("Schema validation failed: {}", e));
        }
    }

    println!();
    println!("Validation complete!");

    Ok(())
}

/// Helper function to print a value
fn print_value(value: &Value, record_set: &RecordSet) -> Result<()> {
    match value {
        Value::UInt32(v) => println!("{v}"),
        Value::Int32(v) => println!("{v}"),
        Value::Float32(v) => println!("{v:.4}"),
        Value::StringRef(v) => match record_set.get_string(*v) {
            Ok(s) => println!("\"{s}\""),
            Err(_) => println!("<Invalid string ref: {}>", v.offset()),
        },
        Value::Bool(v) => println!("{v}"),
        Value::UInt8(v) => println!("{v}"),
        Value::Int8(v) => println!("{v}"),
        Value::UInt16(v) => println!("{v}"),
        Value::Int16(v) => println!("{v}"),
        Value::Array(vals) => {
            print!("[");
            for (i, val) in vals.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                match val {
                    Value::UInt32(v) => print!("{v}"),
                    Value::Int32(v) => print!("{v}"),
                    Value::Float32(v) => print!("{v:.4}"),
                    _ => print!("{val:?}"),
                }
            }
            println!("]");
        }
    }
    Ok(())
}

/// Discover the schema of a DBC file through analysis
fn discover_command(
    file: &Path,
    max_records: u32,
    validate_strings: bool,
    detect_arrays: bool,
    detect_key: bool,
    yaml: bool,
    output_path: Option<&Path>,
) -> Result<()> {
    println!("Discovering schema for: {}", file.display());
    println!();

    // Open and parse the DBC file
    let start = Instant::now();
    let dbc_file =
        File::open(file).with_context(|| format!("Failed to open DBC file: {}", file.display()))?;
    let mut reader = BufReader::new(dbc_file);

    let parser = DbcParser::parse(&mut reader)
        .with_context(|| format!("Failed to parse DBC file: {}", file.display()))?;
    println!("File parsed in: {:?}", start.elapsed());

    // Print header information
    let header = parser.header();
    println!("DBC Header Information:");
    println!("======================");
    println!(
        "Magic: {} ({:02X?})",
        std::str::from_utf8(&header.magic).unwrap_or("Invalid"),
        header.magic
    );
    println!("Version: {:?}", parser.version());
    println!("Record Count: {}", header.record_count);
    println!("Field Count: {}", header.field_count);
    println!("Record Size: {} bytes", header.record_size);
    println!("String Block Size: {} bytes", header.string_block_size);
    println!();

    // Parse records to get the string block
    let record_set = parser.parse_records().context("Failed to parse records")?;
    println!("Records parsed in: {:?}", start.elapsed());

    // Create schema discoverer
    let discoverer = SchemaDiscoverer::new(header, parser.data(), record_set.string_block())
        .with_max_records(max_records)
        .with_validate_strings(validate_strings)
        .with_detect_arrays(detect_arrays)
        .with_detect_key(detect_key);

    // Discover schema
    let start = Instant::now();
    let discovered = discoverer.discover().context("Failed to discover schema")?;
    println!("Schema discovered in: {:?}", start.elapsed());
    println!();

    // Check validation status
    println!("Schema Validation:");
    println!("==================");
    if discovered.is_valid {
        println!("✓ Schema is valid!");
    } else {
        println!(
            "✗ Schema validation failed: {}",
            discovered
                .validation_message
                .unwrap_or_else(|| "Unknown validation error".to_string())
        );
    }
    println!();

    // Print discovered fields
    println!("Discovered Fields:");
    println!("==================");
    for (i, field) in discovered.fields.iter().enumerate() {
        println!(
            "Field {:2}: {:8} (confidence: {:?})",
            i,
            format!("{:?}", field.field_type),
            field.confidence
        );

        if field.is_array {
            println!("           Array size: {}", field.array_size.unwrap_or(0));
        }

        if field.is_key_candidate {
            println!("           ⚷ Key candidate");
        }

        // Print sample values
        let samples: Vec<String> = field
            .sample_values
            .iter()
            .take(5)
            .map(|v| v.to_string())
            .collect();
        println!("           Sample values: [{}]", samples.join(", "));
    }
    println!();

    // Print key field
    if let Some(key_index) = discovered.key_field_index {
        println!(
            "Key Field: Field {} ({})",
            key_index,
            if key_index < discovered.fields.len() {
                format!("{:?}", discovered.fields[key_index].field_type)
            } else {
                "Unknown".to_string()
            }
        );
    } else {
        println!("Key Field: None detected");
    }
    println!();

    // Generate schema
    let file_stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let schema = discoverer
        .generate_schema(&file_stem)
        .context("Failed to generate schema")?;

    // Output schema
    if yaml {
        #[cfg(feature = "yaml")]
        {
            use serde::{Deserialize, Serialize};

            // Convert to YAML-compatible structures
            #[derive(Serialize, Deserialize)]
            struct YamlSchemaField {
                name: String,
                type_name: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                is_array: Option<bool>,
                #[serde(skip_serializing_if = "Option::is_none")]
                array_size: Option<usize>,
            }

            #[derive(Serialize, Deserialize)]
            struct YamlSchema {
                name: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                key_field: Option<String>,
                fields: Vec<YamlSchemaField>,
            }

            let mut fields = Vec::new();
            for field in schema.fields.iter() {
                let type_name = match field.field_type {
                    FieldType::Int32 => "Int32",
                    FieldType::UInt32 => "UInt32",
                    FieldType::Float32 => "Float32",
                    FieldType::String => "String",
                    FieldType::Bool => "Bool",
                    FieldType::UInt8 => "UInt8",
                    FieldType::Int8 => "Int8",
                    FieldType::UInt16 => "UInt16",
                    FieldType::Int16 => "Int16",
                };

                fields.push(YamlSchemaField {
                    name: field.name.clone(),
                    type_name: type_name.to_string(),
                    is_array: if field.is_array { Some(true) } else { None },
                    array_size: field.array_size,
                });
            }

            let key_field = if let Some(key_index) = schema.key_field_index {
                if key_index < schema.fields.len() {
                    Some(schema.fields[key_index].name.clone())
                } else {
                    None
                }
            } else {
                None
            };

            let yaml_schema = YamlSchema {
                name: schema.name.clone(),
                key_field,
                fields,
            };

            let yaml_content = serde_yaml_ng::to_string(&yaml_schema)
                .context("Failed to serialize schema to YAML")?;

            // Output to file or stdout
            if let Some(output_path) = output_path {
                std::fs::write(output_path, yaml_content).with_context(|| {
                    format!("Failed to write schema to: {}", output_path.display())
                })?;
                println!("Schema written to: {}", output_path.display());
            } else {
                println!("Generated Schema (YAML):");
                println!("========================");
                println!("{yaml_content}");
            }
        }

        #[cfg(not(feature = "yaml"))]
        {
            anyhow::bail!(
                "YAML output requested but yaml feature is not enabled. Rebuild with --features yaml to enable YAML support."
            );
        }
    } else {
        // Output schema in text format
        println!("Generated Schema:");
        println!("=================");
        println!("Name: {}", schema.name);

        if let Some(key_index) = schema.key_field_index {
            if key_index < schema.fields.len() {
                println!("Key Field: {}", schema.fields[key_index].name);
            }
        }

        println!("Fields:");

        for (i, field) in schema.fields.iter().enumerate() {
            let field_desc = if field.is_array {
                format!(
                    "{}: {:?}[{}]",
                    field.name,
                    field.field_type,
                    field.array_size.unwrap_or(0)
                )
            } else {
                format!("{}: {:?}", field.name, field.field_type)
            };

            if Some(i) == schema.key_field_index {
                println!("  {field_desc} (Key field)");
            } else {
                println!("  {field_desc}");
            }
        }

        // Write schema to file if requested
        if let Some(output_path) = output_path {
            let mut contents = format!("Schema: {}\n", schema.name);

            if let Some(key_index) = schema.key_field_index {
                if key_index < schema.fields.len() {
                    contents.push_str(&format!("Key Field: {}\n", schema.fields[key_index].name));
                }
            }

            contents.push_str("Fields:\n");

            for (i, field) in schema.fields.iter().enumerate() {
                let field_desc = if field.is_array {
                    format!(
                        "  {}: {:?}[{}]\n",
                        field.name,
                        field.field_type,
                        field.array_size.unwrap_or(0)
                    )
                } else {
                    format!("  {}: {:?}\n", field.name, field.field_type)
                };

                contents.push_str(&field_desc);

                if Some(i) == schema.key_field_index {
                    contents.push_str("    (Key field)\n");
                }
            }

            std::fs::write(output_path, contents)
                .with_context(|| format!("Failed to write schema to: {}", output_path.display()))?;
            println!("\nSchema written to: {}", output_path.display());
        }
    }

    Ok(())
}
