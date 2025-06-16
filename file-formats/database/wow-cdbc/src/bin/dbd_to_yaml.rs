use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dbd_to_yaml")]
#[command(about = "Convert DBD (Database Definition) files to YAML schema format")]
struct Cli {
    /// The DBD file to convert
    input: PathBuf,

    /// Output YAML file
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    println!("Converting DBD file: {}", cli.input.display());

    // Parse DBD using the provided function
    let dbd_definition = wow_cdbc::dbd::parse_dbd_file(&cli.input)?;

    println!("Columns: {}", dbd_definition.columns.len());
    println!("Builds: {}", dbd_definition.builds.len());
    println!("Layouts: {}", dbd_definition.layouts.len());

    // Convert to YAML schema format
    let mut schema = serde_yaml_ng::Mapping::new();

    // Extract name from filename
    let name = cli
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    schema.insert(
        serde_yaml_ng::Value::String("name".to_string()),
        serde_yaml_ng::Value::String(name),
    );

    let mut fields = serde_yaml_ng::Sequence::new();

    for column in &dbd_definition.columns {
        let mut field = serde_yaml_ng::Mapping::new();
        field.insert(
            serde_yaml_ng::Value::String("name".to_string()),
            serde_yaml_ng::Value::String(column.name.clone()),
        );
        field.insert(
            serde_yaml_ng::Value::String("type".to_string()),
            serde_yaml_ng::Value::String(column.base_type.clone()),
        );

        if let Some(ref comment) = column.comment {
            field.insert(
                serde_yaml_ng::Value::String("comment".to_string()),
                serde_yaml_ng::Value::String(comment.to_string()),
            );
        }

        fields.push(serde_yaml_ng::Value::Mapping(field));
    }

    schema.insert(
        serde_yaml_ng::Value::String("fields".to_string()),
        serde_yaml_ng::Value::Sequence(fields),
    );

    // Write YAML
    let yaml_content = serde_yaml_ng::to_string(&serde_yaml_ng::Value::Mapping(schema))?;
    std::fs::write(&cli.output, yaml_content)?;

    println!("Converted to YAML: {}", cli.output.display());

    Ok(())
}
