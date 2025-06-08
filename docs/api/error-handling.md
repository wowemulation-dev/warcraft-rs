# ⚠️ Error Handling

Comprehensive error handling in `warcraft-rs`.

## Error Types

### Core Error Enum

```rust
use std::fmt;
use std::error::Error as StdError;

/// Main error type for warcraft-rs
#[derive(Debug)]
pub enum Error {
    /// I/O related errors
    Io(std::io::Error),

    /// File not found
    FileNotFound(String),

    /// Invalid file format
    InvalidFormat(String),

    /// Unsupported version
    UnsupportedVersion {
        format: String,
        version: u32,
        supported: Vec<u32>,
    },

    /// Invalid magic number
    InvalidMagic {
        expected: [u8; 4],
        found: [u8; 4],
    },

    /// Data corruption
    CorruptedData(String),

    /// Missing required data
    MissingData(String),

    /// Index out of bounds
    IndexOutOfBounds {
        index: usize,
        max: usize,
    },

    /// Invalid chunk
    InvalidChunk {
        fourcc: [u8; 4],
        reason: String,
    },

    /// String decoding error
    StringDecoding(std::string::FromUtf8Error),

    /// Compression error
    Compression(String),

    /// Encryption error
    Encryption(String),

    /// Validation failed
    ValidationFailed(Vec<ValidationError>),

    /// Resource not found
    ResourceNotFound {
        resource_type: String,
        identifier: String,
    },

    /// Operation not supported
    NotSupported(String),

    /// Custom error
    Custom(String),
}
```

### Error Implementation

```rust
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::FileNotFound(path) => write!(f, "File not found: {}", path),
            Error::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Error::UnsupportedVersion { format, version, supported } => {
                write!(f, "Unsupported {} version {}, supported: {:?}",
                    format, version, supported)
            }
            Error::InvalidMagic { expected, found } => {
                write!(f, "Invalid magic: expected {:?}, found {:?}",
                    expected, found)
            }
            Error::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
            Error::MissingData(msg) => write!(f, "Missing data: {}", msg),
            Error::IndexOutOfBounds { index, max } => {
                write!(f, "Index {} out of bounds (max: {})", index, max)
            }
            Error::InvalidChunk { fourcc, reason } => {
                write!(f, "Invalid chunk '{}': {}",
                    String::from_utf8_lossy(fourcc), reason)
            }
            Error::StringDecoding(e) => write!(f, "String decoding error: {}", e),
            Error::Compression(msg) => write!(f, "Compression error: {}", msg),
            Error::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            Error::ValidationFailed(errors) => {
                write!(f, "Validation failed: {} errors", errors.len())
            }
            Error::ResourceNotFound { resource_type, identifier } => {
                write!(f, "{} not found: {}", resource_type, identifier)
            }
            Error::NotSupported(feature) => {
                write!(f, "Feature not supported: {}", feature)
            }
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::StringDecoding(e) => Some(e),
            _ => None,
        }
    }
}
```

### Conversions

```rust
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::StringDecoding(err)
    }
}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Io(e) => e,
            e => std::io::Error::new(std::io::ErrorKind::Other, e),
        }
    }
}
```

## Validation Errors

```rust
/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Validation result
pub struct ValidationResult {
    errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            severity: ValidationSeverity::Error,
        });
    }

    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            severity: ValidationSeverity::Warning,
        });
    }

    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.severity == ValidationSeverity::Error)
    }

    pub fn into_result(self) -> Result<(), Error> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(Error::ValidationFailed(self.errors))
        }
    }
}
```

## Result Type Extensions

```rust
/// Extension trait for Result types
pub trait ResultExt<T> {
    /// Add context to errors
    fn context(self, msg: &str) -> Result<T, Error>;

    /// Convert to custom error with message
    fn or_error(self, msg: &str) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn context(self, msg: &str) -> Result<T, Error> {
        self.map_err(|e| {
            let err: Error = e.into();
            Error::Custom(format!("{}: {}", msg, err))
        })
    }

    fn or_error(self, msg: &str) -> Result<T, Error> {
        self.map_err(|_| Error::Custom(msg.to_string()))
    }
}

/// Extension for Option types
pub trait OptionExt<T> {
    /// Convert None to error
    fn ok_or_error(self, msg: &str) -> Result<T, Error>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_error(self, msg: &str) -> Result<T, Error> {
        self.ok_or_else(|| Error::Custom(msg.to_string()))
    }
}
```

## Error Context

```rust
/// Provides context for errors
pub struct ErrorContext {
    file_path: Option<String>,
    operation: Option<String>,
    details: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            file_path: None,
            operation: None,
            details: Vec::new(),
        }
    }

    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    pub fn with_operation(mut self, op: impl Into<String>) -> Self {
        self.operation = Some(op.into());
        self
    }

    pub fn add_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    pub fn wrap_error(self, error: Error) -> Error {
        let mut msg = String::new();

        if let Some(op) = self.operation {
            msg.push_str(&format!("During {}: ", op));
        }

        msg.push_str(&error.to_string());

        if let Some(path) = self.file_path {
            msg.push_str(&format!(" (file: {})", path));
        }

        for detail in self.details {
            msg.push_str(&format!("\n  - {}", detail));
        }

        Error::Custom(msg)
    }
}
```

## Common Error Patterns

### Early Return with Context

```rust
use crate::{Error, ResultExt};

fn load_model(path: &str) -> Result<Model, Error> {
    let data = std::fs::read(path)
        .context("Failed to read model file")?;

    let header = ModelHeader::parse(&data)
        .context("Failed to parse model header")?;

    if header.version > MAX_SUPPORTED_VERSION {
        return Err(Error::UnsupportedVersion {
            format: "M2".to_string(),
            version: header.version,
            supported: vec![18, 20, 21],
        });
    }

    let model = Model::from_bytes(&data)
        .context("Failed to parse model data")?;

    Ok(model)
}
```

### Validation with Collection

```rust
fn validate_terrain(terrain: &Terrain) -> Result<(), Error> {
    let mut validation = ValidationResult::new();

    // Check bounds
    if terrain.chunks.is_empty() {
        validation.add_error("chunks", "No chunks in terrain");
    }

    // Check heights
    for (i, chunk) in terrain.chunks.iter().enumerate() {
        if chunk.heights.len() != 145 {
            validation.add_error(
                format!("chunks[{}].heights", i),
                format!("Expected 145 heights, found {}", chunk.heights.len())
            );
        }

        // Check for NaN values
        if chunk.heights.iter().any(|&h| h.is_nan()) {
            validation.add_warning(
                format!("chunks[{}].heights", i),
                "Contains NaN values"
            );
        }
    }

    validation.into_result()
}
```

### Graceful Degradation

```rust
fn load_textures(paths: &[String]) -> Vec<Result<Texture, Error>> {
    paths.iter()
        .map(|path| {
            Texture::load(path)
                .or_else(|e| {
                    eprintln!("Warning: Failed to load texture {}: {}", path, e);
                    // Try fallback texture
                    Texture::load("textures/default.blp")
                })
        })
        .collect()
}
```

### Error Recovery

```rust
fn read_chunks(reader: &mut impl Read) -> Result<Vec<Chunk>, Error> {
    let mut chunks = Vec::new();
    let mut errors = Vec::new();

    loop {
        match Chunk::read(reader) {
            Ok(chunk) => chunks.push(chunk),
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // End of file reached
                break;
            }
            Err(Error::InvalidChunk { fourcc, reason }) => {
                // Skip invalid chunk
                eprintln!("Skipping invalid chunk {:?}: {}", fourcc, reason);
                errors.push(format!("Invalid chunk: {:?}", fourcc));

                // Try to recover by seeking to next chunk
                if let Err(e) = skip_to_next_chunk(reader) {
                    return Err(e);
                }
            }
            Err(e) => return Err(e),
        }
    }

    if !errors.is_empty() {
        eprintln!("Loaded with {} errors", errors.len());
    }

    Ok(chunks)
}
```

## Custom Error Types

### Format-Specific Errors

```rust
/// M2-specific errors
#[derive(Debug)]
pub enum M2Error {
    InvalidBoneIndex(u16),
    AnimationNotFound(u16),
    SkinNotFound(u8),
    TextureNotFound(u16),
}

impl From<M2Error> for Error {
    fn from(err: M2Error) -> Self {
        match err {
            M2Error::InvalidBoneIndex(idx) => {
                Error::Custom(format!("Invalid bone index: {}", idx))
            }
            M2Error::AnimationNotFound(id) => {
                Error::ResourceNotFound {
                    resource_type: "Animation".to_string(),
                    identifier: id.to_string(),
                }
            }
            M2Error::SkinNotFound(id) => {
                Error::ResourceNotFound {
                    resource_type: "Skin".to_string(),
                    identifier: id.to_string(),
                }
            }
            M2Error::TextureNotFound(id) => {
                Error::ResourceNotFound {
                    resource_type: "Texture".to_string(),
                    identifier: id.to_string(),
                }
            }
        }
    }
}
```

## Error Reporting

### User-Friendly Messages

```rust
/// Convert errors to user-friendly messages
pub trait UserError {
    fn user_message(&self) -> String;
    fn suggestion(&self) -> Option<String>;
}

impl UserError for Error {
    fn user_message(&self) -> String {
        match self {
            Error::FileNotFound(path) => {
                format!("Could not find file: {}", path)
            }
            Error::UnsupportedVersion { format, version, .. } => {
                format!("This {} file (version {}) is not supported", format, version)
            }
            Error::CorruptedData(_) => {
                "The file appears to be corrupted".to_string()
            }
            _ => "An error occurred while processing the file".to_string(),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Error::FileNotFound(_) => {
                Some("Check that the file path is correct".to_string())
            }
            Error::UnsupportedVersion { supported, .. } => {
                Some(format!("Supported versions: {:?}", supported))
            }
            Error::CorruptedData(_) => {
                Some("Try extracting the file again from the MPQ".to_string())
            }
            _ => None,
        }
    }
}
```

## Best Practices

1. **Use specific error variants** instead of generic `Custom` when possible
2. **Add context** to errors when propagating them up
3. **Validate early** to catch errors before processing
4. **Provide recovery options** when appropriate
5. **Log warnings** for non-fatal issues
6. **Include helpful information** in error messages

## See Also

- [Core Types](core-types.md)
- [Validation Guide](../guides/validation.md)
- [Logging Guide](../guides/logging.md)
