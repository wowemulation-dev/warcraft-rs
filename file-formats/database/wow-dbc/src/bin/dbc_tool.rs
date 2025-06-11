use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use wow_dbc::{DbcParser, FieldType, Schema, SchemaField, export_to_csv, export_to_json};

#[derive(Parser)]
#[command(name = "dbc_tool")]
#[command(about = "A tool for working with World of Warcraft DBC files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display information about a DBC file
    Info {
        /// The DBC file to analyze
        file: PathBuf,
    },
    /// Export DBC data to JSON
    ExportJson {
        /// The DBC file to export
        file: PathBuf,
        /// Output JSON file
        output: PathBuf,
        /// Schema file (YAML)
        #[arg(short, long)]
        schema: Option<PathBuf>,
    },
    /// Export DBC data to CSV
    ExportCsv {
        /// The DBC file to export
        file: PathBuf,
        /// Output CSV file
        output: PathBuf,
        /// Schema file (YAML)
        #[arg(short, long)]
        schema: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file } => {
            let mut reader = BufReader::new(File::open(&file)?);
            let parser = DbcParser::parse(&mut reader)?;
            let header = parser.header();

            println!("DBC File Information:");
            println!("  Magic: {}", String::from_utf8_lossy(&header.magic));
            println!("  Records: {}", header.record_count);
            println!("  Fields: {}", header.field_count);
            println!("  Record Size: {} bytes", header.record_size);
            println!("  String Block Size: {} bytes", header.string_block_size);
        }
        Commands::ExportJson {
            file,
            output,
            schema: _schema,
        } => {
            // For now, use a simple schema
            let data = std::fs::read(&file)?;
            let parser = DbcParser::parse_bytes(&data)?;

            // Create a basic schema (in real use, this would be loaded from the schema file)
            let mut schema = Schema::new("DBC");
            // Add fields based on field count
            let field_count = parser.header().field_count;
            for i in 0..field_count {
                schema.add_field(SchemaField::new(format!("Field{}", i), FieldType::UInt32));
            }

            let parser = parser.with_schema(schema)?;
            let record_set = parser.parse_records()?;

            let json_output = File::create(output)?;
            export_to_json(&record_set, json_output)?;

            println!("Exported to JSON successfully");
        }
        Commands::ExportCsv {
            file,
            output,
            schema: _schema,
        } => {
            // For now, use a simple schema
            let data = std::fs::read(&file)?;
            let parser = DbcParser::parse_bytes(&data)?;

            // Create a basic schema
            let mut schema = Schema::new("DBC");
            let field_count = parser.header().field_count;
            for i in 0..field_count {
                schema.add_field(SchemaField::new(format!("Field{}", i), FieldType::UInt32));
            }

            let parser = parser.with_schema(schema)?;
            let record_set = parser.parse_records()?;

            let csv_output = File::create(output)?;
            export_to_csv(&record_set, csv_output)?;

            println!("Exported to CSV successfully");
        }
    }

    Ok(())
}
