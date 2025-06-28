// parallel.rs - Parallel parsing of ADT files

#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crate::Adt;
use crate::AdtVersion;
use crate::error::Result;

/// Callback function for parallel processing
#[allow(dead_code)]
type ProcessCallback<T> = fn(&Adt, &Path) -> Result<T>;

/// Options for parallel processing
#[derive(Debug, Clone, Default)]
pub struct ParallelOptions {
    /// Maximum number of threads to use (0 = auto)
    pub max_threads: usize,
    /// Whether to continue on errors
    pub continue_on_error: bool,
    /// Whether to use memory mapping
    pub use_mmap: bool,
}

/// Process multiple ADT files in parallel
#[cfg(feature = "parallel")]
pub fn process_parallel<T, F>(
    files: &[PathBuf],
    processor: F,
    options: &ParallelOptions,
) -> Result<Vec<(PathBuf, Result<T>)>>
where
    F: Fn(&Adt, &Path) -> Result<T> + Send + Sync,
    T: Send,
{
    // Set thread pool size if specified
    if options.max_threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(options.max_threads)
            .build_global()
            .map_err(|e| {
                crate::error::AdtError::ParseError(format!(
                    "Failed to configure thread pool: {e}"
                ))
            })?;
    }

    // Process files in parallel
    let results: Vec<(PathBuf, Result<T>)> = files
        .par_iter()
        .map(|file_path| {
            let result = process_single_file(file_path, &processor, options);
            (file_path.clone(), result)
        })
        .collect();

    Ok(results)
}

/// Process multiple ADT files sequentially
#[cfg(not(feature = "parallel"))]
pub fn process_parallel<T, F>(
    files: &[PathBuf],
    processor: F,
    options: &ParallelOptions,
) -> Result<Vec<(PathBuf, Result<T>)>>
where
    F: Fn(&Adt, &Path) -> Result<T>,
{
    let mut results = Vec::with_capacity(files.len());

    for file_path in files {
        let result = process_single_file(file_path, &processor, options);
        results.push((file_path.clone(), result));
    }

    Ok(results)
}

/// Process a single ADT file
fn process_single_file<T, F>(
    file_path: &Path,
    processor: &F,
    options: &ParallelOptions,
) -> Result<T>
where
    F: Fn(&Adt, &Path) -> Result<T>,
{
    // Parse the ADT file
    let adt = if options.use_mmap {
        #[cfg(feature = "mmap")]
        {
            load_adt_mmap(file_path)?
        }
        #[cfg(not(feature = "mmap"))]
        {
            Adt::from_path(file_path)?
        }
    } else {
        Adt::from_path(file_path)?
    };

    // Process the ADT
    processor(&adt, file_path)
}

/// Load an ADT file using memory mapping
#[cfg(feature = "mmap")]
fn load_adt_mmap(path: &Path) -> Result<Adt> {
    use memmap2::Mmap;
    use std::io::Cursor;

    // Open the file
    let file = File::open(path)?;

    // Create a memory map
    let mmap = unsafe { Mmap::map(&file)? };

    // Create a cursor over the memory map
    let mut cursor = Cursor::new(&mmap[..]);

    // Parse the ADT
    Adt::from_reader(&mut cursor)
}

/// Batch convert ADT files from one version to another
pub fn batch_convert(
    input_files: &[PathBuf],
    output_dir: &Path,
    target_version: AdtVersion,
    options: &ParallelOptions,
) -> Result<Vec<(PathBuf, Result<()>)>> {
    // Define processor function
    let processor = move |adt: &Adt, input_path: &Path| {
        // Convert to target version
        let converted = adt.to_version(target_version)?;

        // Determine output path
        let file_name = input_path.file_name().unwrap();
        let output_path = output_dir.join(file_name);

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Write converted ADT
        let output_file = File::create(&output_path)?;
        let mut writer = BufWriter::new(output_file);
        converted.write(&mut writer)?;

        Ok(())
    };

    // Process files
    process_parallel(input_files, processor, options)
}

/// Batch validate ADT files
pub fn batch_validate(
    input_files: &[PathBuf],
    output_dir: &Path,
    validation_level: crate::validator::ValidationLevel,
    options: &ParallelOptions,
) -> Result<Vec<(PathBuf, Result<crate::validator::ValidationReport>)>> {
    // Define processor function
    let processor = move |adt: &Adt, input_path: &Path| {
        // Validate the ADT
        let report = adt.validate_with_report(validation_level)?;

        // Write report to file if output_dir is provided
        if !output_dir.as_os_str().is_empty() {
            // Determine output path
            let file_name = input_path.file_name().unwrap();
            let mut output_path = output_dir.join(file_name);
            output_path.set_extension("txt");

            // Create output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            // Write report
            let mut file = File::create(&output_path)?;
            use std::io::Write;
            writeln!(file, "Validation Report for {}", input_path.display())?;
            writeln!(file, "{}", report.format())?;
        }

        Ok(report)
    };

    // Process files
    process_parallel(input_files, processor, options)
}
