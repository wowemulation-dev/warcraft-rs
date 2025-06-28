//! BLP texture command implementations

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use image::{ImageFormat, ImageReader, imageops::FilterType};
use std::path::{Path, PathBuf};
use wow_blp::{
    convert::{
        AlphaBits, Blp2Format, BlpOldFormat, BlpTarget, DxtAlgorithm, blp_to_image, image_to_blp,
    },
    encode::save_blp,
    parser::load_blp,
    types::BlpContent,
};

#[derive(Subcommand)]
pub enum BlpCommands {
    /// Display information about a BLP file
    Info {
        /// Path to the BLP file
        file: PathBuf,

        /// Show detailed mipmap information
        #[arg(long)]
        mipmaps: bool,

        /// Show raw header data
        #[arg(long)]
        raw: bool,

        /// Show compression statistics and ratios
        #[arg(long)]
        compression: bool,

        /// Show file size breakdown
        #[arg(long)]
        size: bool,

        /// Find best mipmap level for target size
        #[arg(long)]
        best_mipmap_for: Option<u32>,

        /// Show all information (equivalent to --mipmaps --compression --size)
        #[arg(long)]
        all: bool,
    },

    /// Validate BLP file integrity
    Validate {
        /// Path to the BLP file
        file: PathBuf,

        /// Strict validation mode
        #[arg(long)]
        strict: bool,
    },

    /// Convert BLP files to/from other image formats
    Convert {
        /// Input file path (BLP or other image format)
        input: PathBuf,

        /// Output file path
        output: PathBuf,

        /// Input format (auto-detected from extension if not specified)
        #[arg(short = 'i', long)]
        input_format: Option<InputFormat>,

        /// Output format (auto-detected from extension if not specified)
        #[arg(short = 'o', long)]
        output_format: Option<OutputFormat>,

        /// BLP version to use when encoding to BLP
        #[arg(long, default_value = "blp1")]
        blp_version: BlpVersionCli,

        /// BLP encoding format to use
        #[arg(long, default_value = "jpeg")]
        blp_format: BlpFormat,

        /// Alpha bits (0, 1, 4, or 8)
        #[arg(long, default_value = "8")]
        alpha_bits: u8,

        /// Mipmap level to extract when converting from BLP
        #[arg(long, default_value = "0")]
        mipmap_level: usize,

        /// Skip mipmap generation when encoding to BLP
        #[arg(long)]
        no_mipmaps: bool,

        /// Mipmap filtering algorithm
        #[arg(long, default_value = "lanczos3")]
        mipmap_filter: MipmapFilter,

        /// DXT compression quality
        #[arg(long, default_value = "medium")]
        dxt_compression: DxtCompression,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum InputFormat {
    Blp,
    Png,
    Jpeg,
    Gif,
    Bmp,
    Ico,
    Tiff,
    Webp,
    Pnm,
    Dds,
    Tga,
    #[value(name = "openexr")]
    OpenExr,
    Farbfeld,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Blp,
    Png,
    Jpeg,
    Gif,
    Bmp,
    Ico,
    Tiff,
    Pnm,
    Tga,
    #[value(name = "openexr")]
    OpenExr,
    Farbfeld,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum BlpVersionCli {
    Blp0,
    Blp1,
    Blp2,
}

impl From<BlpVersionCli> for wow_blp::types::BlpVersion {
    fn from(value: BlpVersionCli) -> Self {
        match value {
            BlpVersionCli::Blp0 => wow_blp::types::BlpVersion::Blp0,
            BlpVersionCli::Blp1 => wow_blp::types::BlpVersion::Blp1,
            BlpVersionCli::Blp2 => wow_blp::types::BlpVersion::Blp2,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum BlpFormat {
    Raw1,
    Raw3,
    Jpeg,
    Dxt1,
    Dxt3,
    Dxt5,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum MipmapFilter {
    /// Nearest Neighbor
    Nearest,
    /// Linear Filter
    Triangle,
    /// Cubic Filter
    CatmullRom,
    /// Gaussian Filter
    Gaussian,
    /// Lanczos with window 3
    Lanczos3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum DxtCompression {
    /// Range fit, fast, poor quality
    Fastest,
    /// Cluster algorithm, slow, good quality
    Medium,
    /// Iterative cluster algorithm, very slow, great quality
    Finest,
}

// Conversion implementations

impl From<MipmapFilter> for FilterType {
    fn from(value: MipmapFilter) -> FilterType {
        match value {
            MipmapFilter::Nearest => FilterType::Nearest,
            MipmapFilter::Triangle => FilterType::Triangle,
            MipmapFilter::CatmullRom => FilterType::CatmullRom,
            MipmapFilter::Gaussian => FilterType::Gaussian,
            MipmapFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

impl From<DxtCompression> for DxtAlgorithm {
    fn from(value: DxtCompression) -> DxtAlgorithm {
        match value {
            DxtCompression::Fastest => DxtAlgorithm::RangeFit,
            DxtCompression::Medium => DxtAlgorithm::ClusterFit,
            DxtCompression::Finest => DxtAlgorithm::IterativeClusterFit,
        }
    }
}

impl TryFrom<OutputFormat> for ImageFormat {
    type Error = anyhow::Error;

    fn try_from(val: OutputFormat) -> Result<ImageFormat> {
        match val {
            OutputFormat::Blp => anyhow::bail!("BLP format handled separately"),
            OutputFormat::Png => Ok(ImageFormat::Png),
            OutputFormat::Jpeg => Ok(ImageFormat::Jpeg),
            OutputFormat::Gif => Ok(ImageFormat::Gif),
            OutputFormat::Bmp => Ok(ImageFormat::Bmp),
            OutputFormat::Ico => Ok(ImageFormat::Ico),
            OutputFormat::Tiff => Ok(ImageFormat::Tiff),
            OutputFormat::Pnm => Ok(ImageFormat::Pnm),
            OutputFormat::Tga => Ok(ImageFormat::Tga),
            OutputFormat::OpenExr => Ok(ImageFormat::OpenExr),
            OutputFormat::Farbfeld => Ok(ImageFormat::Farbfeld),
        }
    }
}

fn guess_input_format(path: &Path) -> Option<InputFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "blp" => Some(InputFormat::Blp),
        "png" => Some(InputFormat::Png),
        "jpg" | "jpeg" => Some(InputFormat::Jpeg),
        "gif" => Some(InputFormat::Gif),
        "bmp" => Some(InputFormat::Bmp),
        "ico" => Some(InputFormat::Ico),
        "tiff" | "tif" => Some(InputFormat::Tiff),
        "webp" => Some(InputFormat::Webp),
        "pnm" | "pbm" | "pgm" | "ppm" | "pam" => Some(InputFormat::Pnm),
        "dds" => Some(InputFormat::Dds),
        "tga" => Some(InputFormat::Tga),
        "exr" => Some(InputFormat::OpenExr),
        "ff" | "farbfeld" => Some(InputFormat::Farbfeld),
        _ => None,
    }
}

fn guess_output_format(path: &Path) -> Option<OutputFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "blp" => Some(OutputFormat::Blp),
        "png" => Some(OutputFormat::Png),
        "jpg" | "jpeg" => Some(OutputFormat::Jpeg),
        "gif" => Some(OutputFormat::Gif),
        "bmp" => Some(OutputFormat::Bmp),
        "ico" => Some(OutputFormat::Ico),
        "tiff" | "tif" => Some(OutputFormat::Tiff),
        "pnm" | "pbm" | "pgm" | "ppm" | "pam" => Some(OutputFormat::Pnm),
        "tga" => Some(OutputFormat::Tga),
        "exr" => Some(OutputFormat::OpenExr),
        "ff" | "farbfeld" => Some(OutputFormat::Farbfeld),
        _ => None,
    }
}

fn make_blp_target(
    version: BlpVersionCli,
    format: BlpFormat,
    alpha_bits: u8,
    dxt_algo: DxtCompression,
) -> Result<BlpTarget> {
    use wow_blp::types::BlpVersion;
    let version: BlpVersion = version.into();
    match version {
        BlpVersion::Blp0 => match format {
            BlpFormat::Raw1 => {
                let alpha_bits = match alpha_bits {
                    0 => AlphaBits::NoAlpha,
                    1 => AlphaBits::Bit1,
                    4 => AlphaBits::Bit4,
                    8 => AlphaBits::Bit8,
                    _ => anyhow::bail!("Invalid alpha bits {} for BLP0 Raw1 format", alpha_bits),
                };
                Ok(BlpTarget::Blp0(BlpOldFormat::Raw1 { alpha_bits }))
            }
            BlpFormat::Jpeg => {
                let has_alpha = match alpha_bits {
                    0 => false,
                    8 => true,
                    _ => anyhow::bail!(
                        "Invalid alpha bits {} for BLP0 JPEG format (only 0 or 8 supported)",
                        alpha_bits
                    ),
                };
                Ok(BlpTarget::Blp0(BlpOldFormat::Jpeg { has_alpha }))
            }
            _ => anyhow::bail!("BLP0 only supports Raw1 and JPEG formats"),
        },
        BlpVersion::Blp1 => match format {
            BlpFormat::Raw1 => {
                let alpha_bits = match alpha_bits {
                    0 => AlphaBits::NoAlpha,
                    1 => AlphaBits::Bit1,
                    4 => AlphaBits::Bit4,
                    8 => AlphaBits::Bit8,
                    _ => anyhow::bail!("Invalid alpha bits {} for BLP1 Raw1 format", alpha_bits),
                };
                Ok(BlpTarget::Blp1(BlpOldFormat::Raw1 { alpha_bits }))
            }
            BlpFormat::Jpeg => {
                let has_alpha = match alpha_bits {
                    0 => false,
                    8 => true,
                    _ => anyhow::bail!(
                        "Invalid alpha bits {} for BLP1 JPEG format (only 0 or 8 supported)",
                        alpha_bits
                    ),
                };
                Ok(BlpTarget::Blp1(BlpOldFormat::Jpeg { has_alpha }))
            }
            _ => anyhow::bail!("BLP1 only supports Raw1 and JPEG formats"),
        },
        BlpVersion::Blp2 => match format {
            BlpFormat::Raw1 => {
                let alpha_bits = match alpha_bits {
                    0 => AlphaBits::NoAlpha,
                    1 => AlphaBits::Bit1,
                    4 => AlphaBits::Bit4,
                    8 => AlphaBits::Bit8,
                    _ => anyhow::bail!("Invalid alpha bits {} for BLP2 Raw1 format", alpha_bits),
                };
                Ok(BlpTarget::Blp2(Blp2Format::Raw1 { alpha_bits }))
            }
            BlpFormat::Raw3 => Ok(BlpTarget::Blp2(Blp2Format::Raw3)),
            BlpFormat::Jpeg => {
                let has_alpha = match alpha_bits {
                    0 => false,
                    8 => true,
                    _ => anyhow::bail!(
                        "Invalid alpha bits {} for BLP2 JPEG format (only 0 or 8 supported)",
                        alpha_bits
                    ),
                };
                Ok(BlpTarget::Blp2(Blp2Format::Jpeg { has_alpha }))
            }
            BlpFormat::Dxt1 => {
                let has_alpha = match alpha_bits {
                    0 => false,
                    1 => true,
                    _ => anyhow::bail!(
                        "Invalid alpha bits {} for BLP2 DXT1 format (only 0 or 1 supported)",
                        alpha_bits
                    ),
                };
                Ok(BlpTarget::Blp2(Blp2Format::Dxt1 {
                    has_alpha,
                    compress_algorithm: dxt_algo.into(),
                }))
            }
            BlpFormat::Dxt3 => {
                let has_alpha = match alpha_bits {
                    0 => false,
                    8 => true,
                    _ => anyhow::bail!(
                        "Invalid alpha bits {} for BLP2 DXT3 format (only 0 or 8 supported)",
                        alpha_bits
                    ),
                };
                Ok(BlpTarget::Blp2(Blp2Format::Dxt3 {
                    has_alpha,
                    compress_algorithm: dxt_algo.into(),
                }))
            }
            BlpFormat::Dxt5 => {
                let has_alpha = match alpha_bits {
                    0 => false,
                    8 => true,
                    _ => anyhow::bail!(
                        "Invalid alpha bits {} for BLP2 DXT5 format (only 0 or 8 supported)",
                        alpha_bits
                    ),
                };
                Ok(BlpTarget::Blp2(Blp2Format::Dxt5 {
                    has_alpha,
                    compress_algorithm: dxt_algo.into(),
                }))
            }
        },
    }
}

fn convert_blp(args: ConvertArgs) -> Result<()> {
    // Determine input format
    let input_format = args
        .input_format
        .or_else(|| guess_input_format(&args.input))
        .context("Failed to determine input format. Please specify with --input-format")?;

    // Determine output format
    let output_format = args
        .output_format
        .or_else(|| guess_output_format(&args.output))
        .context("Failed to determine output format. Please specify with --output-format")?;

    log::info!("Converting from {input_format:?} to {output_format:?}");

    // Load input image
    let input_image = if input_format == InputFormat::Blp {
        let blp_image = load_blp(&args.input)
            .with_context(|| format!("Failed to load BLP file: {}", args.input.display()))?;

        blp_to_image(&blp_image, args.mipmap_level)
            .with_context(|| format!("Failed to convert BLP mipmap level {}", args.mipmap_level))?
    } else {
        ImageReader::open(&args.input)
            .with_context(|| format!("Failed to open image file: {}", args.input.display()))?
            .decode()
            .with_context(|| format!("Failed to decode image: {}", args.input.display()))?
    };

    // Save output
    match output_format {
        OutputFormat::Blp => {
            let target = make_blp_target(
                args.blp_version,
                args.blp_format,
                args.alpha_bits,
                args.dxt_compression,
            )?;
            let blp = image_to_blp(
                input_image,
                !args.no_mipmaps,
                target,
                args.mipmap_filter.into(),
            )
            .context("Failed to convert image to BLP")?;

            save_blp(&blp, &args.output)
                .with_context(|| format!("Failed to save BLP file: {}", args.output.display()))?;
        }
        _ => {
            let img_format = output_format.try_into()?;
            input_image
                .save_with_format(&args.output, img_format)
                .with_context(|| format!("Failed to save image: {}", args.output.display()))?;
        }
    }

    println!(
        "✓ Converted {} to {}",
        args.input.display(),
        args.output.display()
    );
    Ok(())
}

fn show_blp_info(
    file: PathBuf,
    show_mipmaps: bool,
    show_raw: bool,
    show_compression: bool,
    show_size: bool,
    best_mipmap_for: Option<u32>,
    show_all: bool,
) -> Result<()> {
    let blp =
        load_blp(&file).with_context(|| format!("Failed to load BLP file: {}", file.display()))?;

    // Apply --all flag
    let show_mipmaps = show_mipmaps || show_all;
    let show_compression = show_compression || show_all;
    let show_size = show_size || show_all;

    println!("BLP File Information: {}", file.display());
    println!("=====================================");

    // Basic info
    println!("Version: {:?}", blp.header.version);
    println!("Dimensions: {}x{}", blp.header.width, blp.header.height);
    println!("Content Type: {:?}", blp.header.content);
    println!("Compression: {:?}", blp.compression_type());
    println!("Alpha Bits: {}", blp.alpha_bit_depth());
    println!("Has Mipmaps: {}", blp.header.has_mipmaps());
    println!("Image Count: {}", blp.image_count());

    // Content-specific info
    match &blp.content {
        BlpContent::Jpeg(jpeg) => {
            println!("JPEG Header Size: {} bytes", jpeg.header.len());
        }
        BlpContent::Raw1(raw) => {
            println!("Palette Colors: {}", raw.cmap.len());
        }
        BlpContent::Raw3(_) => {
            println!("Format Details: Uncompressed BGRA");
        }
        BlpContent::Dxt1(_) => {
            println!("Format Details: S3TC BC1 (4 bpp)");
        }
        BlpContent::Dxt3(_) => {
            println!("Format Details: S3TC BC2 (8 bpp, explicit alpha)");
        }
        BlpContent::Dxt5(_) => {
            println!("Format Details: S3TC BC3 (8 bpp, interpolated alpha)");
        }
    }

    // Compression statistics
    if show_compression {
        println!("\nCompression Statistics:");
        println!("----------------------");
        let compression_ratio = blp.compression_ratio();
        println!("Compression Ratio: {compression_ratio:.2}:1");
        println!(
            "Compression Efficiency: {:.1}%",
            (1.0 - 1.0 / compression_ratio) * 100.0
        );
    }

    // File size breakdown
    if show_size {
        println!("\nFile Size Information:");
        println!("---------------------");
        let estimated_size = blp.estimated_file_size();
        println!(
            "Estimated File Size: {} bytes ({:.2} KB)",
            estimated_size,
            estimated_size as f32 / 1024.0
        );

        let mipmap_info = blp.mipmap_info();
        let total_uncompressed = mipmap_info
            .iter()
            .map(|info| info.width * info.height * 4)
            .sum::<u32>();
        println!(
            "Uncompressed Size: {} bytes ({:.2} KB)",
            total_uncompressed,
            total_uncompressed as f32 / 1024.0
        );
    }

    // Best mipmap for target size
    if let Some(target_size) = best_mipmap_for {
        println!("\nBest Mipmap for {target_size}x{target_size} target:");
        println!("-------------------------------");
        let best_level = blp.best_mipmap_for_size(target_size);
        let (width, height) = blp.header.mipmap_size(best_level);
        println!("Best Level: {best_level} ({width}x{height})");
    }

    // Mipmap info (using new convenience method)
    if show_mipmaps {
        println!("\nMipmap Information:");
        println!("-------------------");
        let mipmap_info = blp.mipmap_info();
        for info in &mipmap_info {
            println!(
                "  Level {}: {}x{} ({} bytes, {} pixels)",
                info.level, info.width, info.height, info.data_size, info.pixel_count
            );
        }
    }

    // Raw header data
    if show_raw {
        println!("\nRaw Header Data:");
        println!("----------------");
        println!("  Version: {:?}", blp.header.version);
        println!("  Content Tag: {:?}", blp.header.content);
        println!("  Flags: {:?}", blp.header.flags);
        println!("  Width: {}", blp.header.width);
        println!("  Height: {}", blp.header.height);
        println!("  Mipmap Locator: {:?}", blp.header.mipmap_locator);
    }

    Ok(())
}

fn validate_blp(file: PathBuf, strict: bool) -> Result<()> {
    println!("Validating BLP file: {}", file.display());

    // Try to load the file
    let blp = match load_blp(&file) {
        Ok(blp) => blp,
        Err(e) => {
            println!("✗ Failed to load BLP file: {e}");
            return Err(e.into());
        }
    };

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check version
    log::debug!("BLP version: {:?}", blp.header.version);

    // Check dimensions
    if blp.header.width == 0 || blp.header.height == 0 {
        errors.push("Invalid dimensions (0 width or height)".to_string());
    }

    if !blp.header.width.is_power_of_two() || !blp.header.height.is_power_of_two() {
        if strict {
            errors.push("Dimensions are not powers of 2".to_string());
        } else {
            warnings.push("Dimensions are not powers of 2 (non-standard but may work)".to_string());
        }
    }

    // Check mipmap consistency
    if blp.header.has_mipmaps() {
        let expected_levels = (blp.header.width.max(blp.header.height) as f32).log2() as usize + 1;
        let actual_levels = blp.image_count();

        if actual_levels < expected_levels {
            warnings.push(format!(
                "Incomplete mipmap chain: expected {expected_levels} levels, got {actual_levels}"
            ));
        }
    }

    // Format-specific validation
    match &blp.content {
        BlpContent::Jpeg(jpeg) => {
            // JPEG-specific validations
            if jpeg.header.is_empty() {
                errors.push("JPEG header is empty".to_string());
            }
        }
        BlpContent::Dxt1(_) | BlpContent::Dxt3(_) | BlpContent::Dxt5(_) => {
            // DXT requires dimensions to be multiples of 4
            if blp.header.width % 4 != 0 || blp.header.height % 4 != 0 {
                errors.push("DXT format requires dimensions to be multiples of 4".to_string());
            }
        }
        _ => {}
    }

    // Print results
    if errors.is_empty() && warnings.is_empty() {
        println!("✓ BLP file is valid");
        Ok(())
    } else {
        if !errors.is_empty() {
            println!("\nErrors:");
            for error in &errors {
                println!("  ✗ {error}");
            }
        }

        if !warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &warnings {
                println!("  ⚠ {warning}");
            }
        }

        if errors.is_empty() {
            println!("\n✓ BLP file is valid with warnings");
            Ok(())
        } else {
            anyhow::bail!("BLP file validation failed with {} error(s)", errors.len())
        }
    }
}

// Helper struct for convert arguments
struct ConvertArgs {
    input: PathBuf,
    output: PathBuf,
    input_format: Option<InputFormat>,
    output_format: Option<OutputFormat>,
    blp_version: BlpVersionCli,
    blp_format: BlpFormat,
    alpha_bits: u8,
    mipmap_level: usize,
    no_mipmaps: bool,
    mipmap_filter: MipmapFilter,
    dxt_compression: DxtCompression,
}

pub fn execute(command: BlpCommands) -> Result<()> {
    match command {
        BlpCommands::Convert {
            input,
            output,
            input_format,
            output_format,
            blp_version,
            blp_format,
            alpha_bits,
            mipmap_level,
            no_mipmaps,
            mipmap_filter,
            dxt_compression,
        } => convert_blp(ConvertArgs {
            input,
            output,
            input_format,
            output_format,
            blp_version,
            blp_format,
            alpha_bits,
            mipmap_level,
            no_mipmaps,
            mipmap_filter,
            dxt_compression,
        }),
        BlpCommands::Info {
            file,
            mipmaps,
            raw,
            compression,
            size,
            best_mipmap_for,
            all,
        } => show_blp_info(file, mipmaps, raw, compression, size, best_mipmap_for, all),
        BlpCommands::Validate { file, strict } => validate_blp(file, strict),
    }
}
