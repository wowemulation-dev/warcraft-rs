# 🔌 Traits & Interfaces

Core traits and interfaces that define behavior across `warcraft-rs`.

## File I/O Traits

### Loading and Saving

```rust
/// Types that can be loaded from a file path
pub trait LoadFromPath: Sized {
    /// Associated error type
    type Error;

    /// Load from file path
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error>;

    /// Load with custom options
    fn load_with_options<P: AsRef<Path>>(
        path: P,
        options: LoadOptions,
    ) -> Result<Self, Self::Error> {
        // Default implementation ignores options
        Self::load(path)
    }
}

/// Types that can be loaded from bytes
pub trait LoadFromBytes: Sized {
    /// Associated error type
    type Error;

    /// Load from byte slice
    fn from_bytes(data: &[u8]) -> Result<Self, Self::Error>;

    /// Load from reader
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::from_bytes(&data)
    }
}

/// Types that can be saved
pub trait Save {
    /// Associated error type
    type Error;

    /// Save to file path
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Self::Error>;

    /// Save to bytes
    fn to_bytes(&self) -> Result<Vec<u8>, Self::Error>;

    /// Save to writer
    fn to_writer<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        let bytes = self.to_bytes()?;
        writer.write_all(&bytes)?;
        Ok(())
    }
}

/// Load options
#[derive(Debug, Clone, Default)]
pub struct LoadOptions {
    /// Validate data after loading
    pub validate: bool,
    /// Use strict parsing (fail on warnings)
    pub strict: bool,
    /// Maximum file size to load
    pub max_size: Option<usize>,
}
```

### Streaming

```rust
/// Types that support streaming operations
pub trait Streamable {
    /// Stream type
    type Stream: Read + Seek;

    /// Open a stream
    fn open_stream<P: AsRef<Path>>(path: P) -> Result<Self::Stream, Error>;
}

/// Types that can be read in chunks
pub trait ChunkedRead: Sized {
    /// Read next chunk
    fn read_chunk<R: Read>(reader: &mut R) -> Result<Option<Self>, Error>;

    /// Iterator over chunks
    fn chunks<R: Read>(reader: R) -> ChunkIterator<R, Self> {
        ChunkIterator::new(reader)
    }
}
```

## Format Traits

### Versioning

```rust
/// Types with version information
pub trait Versioned {
    /// Get version
    fn version(&self) -> Version;

    /// Check if version is supported
    fn is_version_supported(&self) -> bool {
        Self::supported_versions().contains(&self.version())
    }

    /// Get list of supported versions
    fn supported_versions() -> &'static [Version];
}

/// Version information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}
```

### Validation

```rust
/// Types that can be validated
pub trait Validate {
    /// Validation error type
    type Error;

    /// Validate the data
    fn validate(&self) -> Result<(), Self::Error>;

    /// Quick validation (less thorough)
    fn quick_validate(&self) -> Result<(), Self::Error> {
        self.validate()
    }
}

/// Types that can validate themselves during construction
pub trait ValidateOnLoad: Sized {
    /// Try to create with validation
    fn new_validated(data: Self) -> Result<Self, ValidationError> {
        data.validate()?;
        Ok(data)
    }
}
```

### Chunks

```rust
/// Chunk-based file format
pub trait ChunkedFormat {
    /// Read a chunk by ID
    fn read_chunk(&self, id: ChunkId) -> Option<&Chunk>;

    /// Check if chunk exists
    fn has_chunk(&self, id: ChunkId) -> bool {
        self.read_chunk(id).is_some()
    }

    /// Iterate over chunks
    fn chunks(&self) -> impl Iterator<Item = &Chunk>;

    /// Get required chunks
    fn required_chunks() -> &'static [ChunkId];

    /// Get optional chunks
    fn optional_chunks() -> &'static [ChunkId];
}

/// Individual chunk
pub trait Chunk {
    /// Chunk identifier
    fn id(&self) -> ChunkId;

    /// Chunk size
    fn size(&self) -> usize;

    /// Chunk data
    fn data(&self) -> &[u8];
}
```

## Data Access Traits

### Indexing

```rust
/// Types that can be indexed by ID
pub trait IndexedCollection {
    /// Item type
    type Item;

    /// Get by ID
    fn get(&self, id: u32) -> Option<&Self::Item>;

    /// Get mutable by ID
    fn get_mut(&mut self, id: u32) -> Option<&mut Self::Item>;

    /// Check if ID exists
    fn contains(&self, id: u32) -> bool {
        self.get(id).is_some()
    }

    /// Get all IDs
    fn ids(&self) -> impl Iterator<Item = u32>;
}

/// Types that have an ID
pub trait HasId {
    /// Get the ID
    fn id(&self) -> u32;
}
```

### Named Items

```rust
/// Types with names
pub trait Named {
    /// Get name
    fn name(&self) -> &str;

    /// Get display name (may be localized)
    fn display_name(&self) -> &str {
        self.name()
    }
}

/// Types with localized names
pub trait LocalizedNamed {
    /// Get name for locale
    fn name_for_locale(&self, locale: Locale) -> Option<&str>;

    /// Get name with fallback
    fn name_or_default(&self, locale: Locale) -> &str {
        self.name_for_locale(locale)
            .or_else(|| self.name_for_locale(Locale::EnUS))
            .unwrap_or("Unknown")
    }
}
```

## Geometry Traits

### Bounds

```rust
/// Types with bounding boxes
pub trait Bounded {
    /// Get bounding box
    fn bounds(&self) -> BoundingBox;

    /// Get bounding sphere
    fn bounding_sphere(&self) -> BoundingSphere {
        self.bounds().to_sphere()
    }

    /// Check if point is inside
    fn contains_point(&self, point: Vec3) -> bool {
        self.bounds().contains(point)
    }
}

/// Types with spatial queries
pub trait SpatialQuery {
    /// Find nearest point
    fn nearest_point(&self, point: Vec3) -> Vec3;

    /// Ray intersection test
    fn intersect_ray(&self, ray: &Ray) -> Option<f32>;

    /// Sphere intersection test
    fn intersect_sphere(&self, sphere: &BoundingSphere) -> bool;
}
```

### Transformable

```rust
/// Types that can be transformed
pub trait Transformable {
    /// Apply transformation matrix
    fn transform(&mut self, matrix: &Matrix4);

    /// Translate
    fn translate(&mut self, offset: Vec3) {
        let matrix = Matrix4::translation(offset);
        self.transform(&matrix);
    }

    /// Rotate
    fn rotate(&mut self, rotation: Quat) {
        let matrix = Matrix4::from_quaternion(rotation);
        self.transform(&matrix);
    }

    /// Scale
    fn scale(&mut self, scale: Vec3) {
        let matrix = Matrix4::scale(scale);
        self.transform(&matrix);
    }
}
```

## Rendering Traits

### Drawable

```rust
/// Types that can be rendered
pub trait Drawable {
    /// Vertex type
    type Vertex;

    /// Get vertices
    fn vertices(&self) -> &[Self::Vertex];

    /// Get indices
    fn indices(&self) -> &[u16];

    /// Get primitive type
    fn primitive_type(&self) -> PrimitiveType {
        PrimitiveType::Triangles
    }
}

/// Types with material information
pub trait HasMaterial {
    /// Get material
    fn material(&self) -> &Material;

    /// Get texture IDs
    fn texture_ids(&self) -> &[TextureId];

    /// Get shader type
    fn shader_type(&self) -> ShaderType;
}
```

### Level of Detail

```rust
/// Types with LOD support
pub trait HasLod {
    /// Get number of LOD levels
    fn lod_count(&self) -> u8;

    /// Get LOD by index
    fn get_lod(&self, level: u8) -> Option<&dyn Drawable>;

    /// Select appropriate LOD
    fn select_lod(&self, distance: f32) -> u8 {
        // Default implementation
        let thresholds = [30.0, 60.0, 120.0, 250.0];

        for (i, &threshold) in thresholds.iter().enumerate() {
            if distance < threshold {
                return i.min(self.lod_count() - 1) as u8;
            }
        }

        self.lod_count() - 1
    }
}
```

## Animation Traits

### Animated

```rust
/// Types with animation support
pub trait Animated {
    /// Animation state type
    type State;

    /// Get animation by ID
    fn get_animation(&self, id: AnimationId) -> Option<&Animation>;

    /// List all animations
    fn animations(&self) -> impl Iterator<Item = (AnimationId, &Animation)>;

    /// Create animation state
    fn create_state(&self) -> Self::State;

    /// Update animation state
    fn update_state(&self, state: &mut Self::State, delta_ms: u32);

    /// Apply animation state
    fn apply_state(&self, state: &Self::State) -> AnimatedData;
}

/// Animation data
pub trait Animation {
    /// Duration in milliseconds
    fn duration(&self) -> u32;

    /// Whether animation loops
    fn is_looping(&self) -> bool;

    /// Animation name
    fn name(&self) -> &str;

    /// Next animation in sequence
    fn next_animation(&self) -> Option<AnimationId>;
}
```

## Collection Traits

### Container

```rust
/// Generic container trait
pub trait Container {
    /// Item type
    type Item;

    /// Number of items
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get item by index
    fn get(&self, index: usize) -> Option<&Self::Item>;

    /// Iterate over items
    fn iter(&self) -> impl Iterator<Item = &Self::Item>;
}

/// Mutable container
pub trait ContainerMut: Container {
    /// Get mutable item
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item>;

    /// Add item
    fn push(&mut self, item: Self::Item);

    /// Remove item
    fn remove(&mut self, index: usize) -> Option<Self::Item>;

    /// Clear all items
    fn clear(&mut self);
}
```

## Utility Traits

### Cloneable with Context

```rust
/// Clone with additional context
pub trait CloneWithContext {
    /// Context type
    type Context;

    /// Clone with context
    fn clone_with_context(&self, context: &Self::Context) -> Self;
}
```

### Resource Management

```rust
/// Types that manage resources
pub trait Resource {
    /// Resource identifier
    fn resource_id(&self) -> ResourceId;

    /// Estimated memory usage in bytes
    fn memory_usage(&self) -> usize;

    /// Load resource data
    fn load(&mut self) -> Result<(), Error>;

    /// Unload resource data
    fn unload(&mut self);

    /// Check if loaded
    fn is_loaded(&self) -> bool;
}

/// Resource cache
pub trait ResourceCache<T: Resource> {
    /// Get resource by ID
    fn get(&mut self, id: ResourceId) -> Result<&T, Error>;

    /// Preload resource
    fn preload(&mut self, id: ResourceId) -> Result<(), Error>;

    /// Clear cache
    fn clear(&mut self);

    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}
```

## Extension Traits

### Result Extensions

```rust
/// Extensions for Result types
pub trait ResultExt<T> {
    /// Log error and return default
    fn log_err_default(self, default: T) -> T
    where
        Self: Sized;

    /// Convert to option, logging error
    fn log_err(self) -> Option<T>
    where
        Self: Sized;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn log_err_default(self, default: T) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Error: {}", e);
                default
            }
        }
    }

    fn log_err(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(e) => {
                eprintln!("Error: {}", e);
                None
            }
        }
    }
}
```

## Implementation Example

```rust
/// Example implementation for M2 model
impl LoadFromPath for M2Model {
    type Error = Error;

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Self::Error> {
        let data = std::fs::read(path.as_ref())?;
        Self::from_bytes(&data)
    }
}

impl LoadFromBytes for M2Model {
    type Error = Error;

    fn from_bytes(data: &[u8]) -> Result<Self, Self::Error> {
        let header = M2Header::parse(data)?;
        // ... parse rest of model
        Ok(M2Model { header, /* ... */ })
    }
}

impl Validate for M2Model {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        // Validate model data
        if self.vertices.is_empty() {
            return Err(ValidationError::new("No vertices"));
        }
        // ... more validation
        Ok(())
    }
}

impl Bounded for M2Model {
    fn bounds(&self) -> BoundingBox {
        self.header.bounding_box
    }
}

impl HasLod for M2Model {
    fn lod_count(&self) -> u8 {
        self.skins.len() as u8
    }

    fn get_lod(&self, level: u8) -> Option<&dyn Drawable> {
        self.skins.get(level as usize)
            .map(|skin| skin as &dyn Drawable)
    }
}
```

## See Also

- [Core Types](core-types.md)
- [Error Handling](error-handling.md)
- [API Guidelines](guidelines.md)
