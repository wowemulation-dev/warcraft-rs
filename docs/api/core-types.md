# 🔧 Core Types

Common types and traits used throughout `warcraft-rs`.

## Basic Types

### Vector Types

```rust
/// 2D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// 3D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// 4D vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Quaternion for rotations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
```

### Color Types

```rust
/// RGBA color (0-255 per channel)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// RGBA color (0.0-1.0 per channel)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// BGR color (used in some formats)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bgr {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}
```

### Geometry Types

```rust
/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

/// Bounding sphere
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

/// Plane equation (ax + by + cz + d = 0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

/// Ray for intersection tests
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}
```

### Transform Types

```rust
/// 4x4 transformation matrix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    pub data: [[f32; 4]; 4],
}

/// Transformation components
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

## File Format Types

### Chunk Identifier

```rust
/// Four-character code for chunk identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FourCC([u8; 4]);

impl FourCC {
    pub const fn new(bytes: &[u8; 4]) -> Self {
        Self(*bytes)
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        if s.len() != 4 {
            return Err(Error::InvalidFourCC);
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(s.as_bytes());
        Ok(Self(bytes))
    }
}

// Common chunk identifiers
impl FourCC {
    pub const MVER: Self = Self::new(b"MVER");
    pub const MOHD: Self = Self::new(b"MOHD");
    pub const MCNK: Self = Self::new(b"MCNK");
    // ... more chunk types
}
```

### Version Information

```rust
/// File format version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}

/// WoW client version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientVersion {
    Classic,      // 1.12.x
    TBC,          // 2.4.3
    WotLK,        // 3.3.5
    Cataclysm,    // 4.3.4
    MoP,          // 5.4.8
    Custom(Version),
}
```

### String Types

```rust
/// Reference to a string in a string block
#[derive(Debug, Clone, Copy)]
pub struct StringRef {
    pub offset: u32,
}

/// Localized string with multiple language versions
#[derive(Debug, Clone)]
pub struct LocalizedString {
    pub en_us: Option<String>,
    pub ko_kr: Option<String>,
    pub fr_fr: Option<String>,
    pub de_de: Option<String>,
    pub zh_cn: Option<String>,
    pub zh_tw: Option<String>,
    pub es_es: Option<String>,
    pub es_mx: Option<String>,
    pub ru_ru: Option<String>,
    // ... other locales
}

/// Fixed-size string buffer
#[derive(Debug, Clone)]
pub struct FixedString<const N: usize> {
    data: [u8; N],
}
```

## Collections

### Indexed Collection

```rust
/// Collection with O(1) lookup by ID
pub struct IndexedVec<T: HasId> {
    items: Vec<T>,
    index: HashMap<u32, usize>,
}

impl<T: HasId> IndexedVec<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, item: T) {
        let id = item.id();
        let index = self.items.len();
        self.items.push(item);
        self.index.insert(id, index);
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        self.index.get(&id).map(|&idx| &self.items[idx])
    }
}
```

### Chunked Data

```rust
/// Data divided into chunks
pub struct ChunkedData<T> {
    chunk_size: usize,
    chunks: Vec<Vec<T>>,
}

impl<T> ChunkedData<T> {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            chunks: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let chunk_idx = index / self.chunk_size;
        let item_idx = index % self.chunk_size;

        self.chunks.get(chunk_idx)
            .and_then(|chunk| chunk.get(item_idx))
    }
}
```

## Traits

### Core Traits

```rust
/// Types that can be loaded from files
pub trait LoadFromFile: Sized {
    type Options: Default;

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error>;

    fn load_with_options<P: AsRef<Path>>(
        path: P,
        options: Self::Options
    ) -> Result<Self, Error>;
}

/// Types that can be saved to files
pub trait SaveToFile {
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>;
}

/// Types that have an ID
pub trait HasId {
    fn id(&self) -> u32;
}

/// Types that have a name
pub trait HasName {
    fn name(&self) -> &str;
}
```

### Format Traits

```rust
/// Chunk-based file format
pub trait ChunkedFormat {
    fn read_chunk(&mut self, fourcc: FourCC) -> Result<Chunk, Error>;
    fn write_chunk(&mut self, chunk: Chunk) -> Result<(), Error>;
}

/// Versioned format
pub trait Versioned {
    fn version(&self) -> Version;
    fn is_compatible(&self, version: Version) -> bool;
}

/// Format with validation
pub trait Validatable {
    type ValidationError;

    fn validate(&self) -> Result<(), Self::ValidationError>;
}
```

## Utility Types

### Flags

```rust
/// Type-safe flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags<T> {
    bits: u32,
    _phantom: PhantomData<T>,
}

impl<T> Flags<T> {
    pub const fn empty() -> Self {
        Self {
            bits: 0,
            _phantom: PhantomData,
        }
    }

    pub const fn from_bits(bits: u32) -> Self {
        Self {
            bits,
            _phantom: PhantomData,
        }
    }

    pub fn contains(&self, other: Self) -> bool {
        self.bits & other.bits == other.bits
    }
}
```

### Ranges

```rust
/// Inclusive range of values
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T: PartialOrd> Range<T> {
    pub fn contains(&self, value: &T) -> bool {
        value >= &self.min && value <= &self.max
    }
}
```

### Optional References

```rust
/// Optional reference with a null value
#[derive(Debug, Clone, Copy)]
pub struct OptionalRef<T> {
    value: T,
    null_value: T,
}

impl<T: PartialEq> OptionalRef<T> {
    pub fn new(value: T, null_value: T) -> Self {
        Self { value, null_value }
    }

    pub fn get(&self) -> Option<&T> {
        if self.value != self.null_value {
            Some(&self.value)
        } else {
            None
        }
    }
}
```

## Memory Types

### Aligned Data

```rust
/// Data with specific alignment requirements
#[repr(align(16))]
pub struct Aligned16<T> {
    pub data: T,
}

#[repr(align(64))]
pub struct CacheAligned<T> {
    pub data: T,
}
```

### Packed Structures

```rust
/// Packed data (no padding)
#[repr(packed)]
pub struct Packed<T> {
    pub data: T,
}
```

## Conversions

### Common Implementations

```rust
// Vec3 operations
impl Vec3 {
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }
}

// Color conversions
impl From<Rgba> for Rgba32 {
    fn from(color: Rgba) -> Self {
        Self {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0,
        }
    }
}

impl From<Rgba32> for Rgba {
    fn from(color: Rgba32) -> Self {
        Self {
            r: (color.r * 255.0) as u8,
            g: (color.g * 255.0) as u8,
            b: (color.b * 255.0) as u8,
            a: (color.a * 255.0) as u8,
        }
    }
}
```

## Constants

```rust
/// Common constants
pub mod constants {
    /// Chunk size in bytes
    pub const CHUNK_SIZE: usize = 8192;

    /// Maximum texture size
    pub const MAX_TEXTURE_SIZE: u32 = 4096;

    /// Tile size in world units
    pub const TILE_SIZE: f32 = 533.33333;

    /// Chunk count per tile
    pub const CHUNKS_PER_TILE: usize = 16;
}
```

## Type Aliases

```rust
/// Common type aliases
pub type FileId = u32;
pub type ChunkId = FourCC;
pub type LocaleId = u8;
pub type AnimationId = u16;
pub type TextureId = u16;
pub type ModelId = u32;

/// Result type with warcraft-rs error
pub type Result<T> = std::result::Result<T, Error>;
```
