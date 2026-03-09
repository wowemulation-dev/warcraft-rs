use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dbd_to_yaml")]
#[command(about = "Convert a DBD (Database Definition) file to a YAML schema for a specific WoW build version")]
#[command(long_about = "Reads a WoWDBDefs .dbd file and outputs a YAML schema that is \
    compatible with the wow-cdbc schema loader. Use --build to target a specific \
    game version (e.g. '3.3.5.12340'). If --build is omitted the first build \
    section in the DBD is used.")]
struct Cli {
    /// The DBD file to convert
    input: PathBuf,

    /// Output YAML file
    output: PathBuf,

    /// Target build version string (e.g. '3.3.5.12340' or '1.12.1.5875').
    /// Must match a version listed in the DBD file.
    #[arg(short, long)]
    build: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();

    let base_name = cli
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let dbd_file = wow_cdbc::dbd::parse_dbd_file(&cli.input)?;

    let build_filter = cli.build.as_deref();
    let generate_all = build_filter.is_none();

    let schemas = wow_cdbc::dbd::convert_to_yaml_schemas(&dbd_file, &base_name, build_filter, generate_all);

    if schemas.is_empty() {
        if let Some(b) = build_filter {
            eprintln!("No build section matched '{}' in {}", b, cli.input.display());
            eprintln!("Available builds:");
            for build in &dbd_file.builds {
                eprintln!("  {}", build.versions.join(", "));
            }
            std::process::exit(1);
        } else {
            eprintln!("No build sections found in {}", cli.input.display());
            std::process::exit(1);
        }
    }

    // When a build filter is given, write the single matching schema to the requested output path.
    // When no filter is given and there are multiple schemas, write only the first one
    // (since the user provided a single output path).
    let (_, yaml_content, version_suffix) = &schemas[0];

    if schemas.len() > 1 && build_filter.is_none() {
        eprintln!(
            "Note: DBD has {} build sections. Writing schema for '{}'. \
            Use --build to select a specific version.",
            schemas.len(),
            version_suffix
        );
    }

    std::fs::write(&cli.output, yaml_content)?;
    println!("Schema for '{}' written to: {}", version_suffix, cli.output.display());

    Ok(())
}
