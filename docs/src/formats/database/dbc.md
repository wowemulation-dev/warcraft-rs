# DBC Format 📊

DBC (DataBase Client) files are binary data files used by World of Warcraft to
store game data that needs to be accessible by the client. These files contain
structured records with information about spells, items, zones, creatures, and
many other game elements.

## Overview

- **Extension**: `.dbc`
- **Magic**: `WDBC` (0x43424457 in little-endian) - ✅ Implemented
- **Purpose**: Client-side database tables for game data
- **Structure**: Fixed-size records with defined schemas - ✅ Implemented
- **Encoding**: Little-endian binary format - ✅ Implemented
- **String Storage**: Separate string block with offset references - ✅ Implemented  
- **Localization**: Built-in support for multiple languages - ⚠️ Partial - Multiple language support available but not all localizations tested

## Key Characteristics

- **Binary Format**: Optimized for fast loading and minimal storage
- **Fixed-Size Records**: Each record in a DBC file has the same size
- **String Pooling**: Strings are stored in a shared pool at the end of the file
- **Version Stability**: Format remained largely unchanged from Vanilla through WotLK

## Version History

| WoW Version | DBC Format | Notable Changes |
|-------------|------------|-----------------|
| Classic (1.12.x) | WDBC v1 | Original format |
| TBC (2.4.3) | WDBC v1 | No format changes, new files |
| WotLK (3.3.5) | WDBC v1 | No format changes, new fields |
| Cataclysm (4.x) | DB2 | New format with variable record sizes |
| MoP+ (5.x+) | DB2/DB5/DB6 | Progressive format enhancements |

This documentation focuses on the WDBC v1 format used in Classic through WotLK.

## File Structure

A DBC file consists of three main sections:

```
+----------------+
|     Header     |  20 bytes
+----------------+
|                |
|    Records     |  record_count * record_size bytes
|                |
+----------------+
|  String Block  |  string_block_size bytes
+----------------+
```

### DBC Header

The DBC header is always 20 bytes and contains essential metadata:

```rust
#[repr(C, packed)]
struct DbcHeader {
    /// Magic signature: "WDBC" (0x43424457 in little-endian)
    magic: [u8; 4],

    /// Number of records in the file
    record_count: u32,

    /// Number of fields per record
    field_count: u32,

    /// Size of each record in bytes
    record_size: u32,

    /// Size of the string block
    string_block_size: u32,
}

impl DbcHeader {
    const MAGIC: &'static [u8; 4] = b"WDBC";
    const HEADER_SIZE: usize = 20;

    /// Verify header validity
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC &&
        self.record_size == self.field_count * 4 && // Each field is 4 bytes
        self.record_count > 0
    }

    /// Calculate total file size
    pub fn file_size(&self) -> usize {
        Self::HEADER_SIZE +
        (self.record_count as usize * self.record_size as usize) +
        self.string_block_size as usize
    }
}
```

### Record Structure

Records immediately follow the header. Each record:

- Has a fixed size (specified in header)
- Contains `field_count` fields
- Each field is 4 bytes (can be interpreted as different types)

```rust
/// Generic DBC record representation
struct DbcRecord {
    /// Record data as raw bytes
    data: Vec<u8>,
}

impl DbcRecord {
    /// Read a u32 field
    pub fn get_u32(&self, field_index: usize) -> u32 {
        let offset = field_index * 4;
        u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }

    /// Read an i32 field
    pub fn get_i32(&self, field_index: usize) -> i32 {
        self.get_u32(field_index) as i32
    }

    /// Read a f32 field
    pub fn get_f32(&self, field_index: usize) -> f32 {
        f32::from_bits(self.get_u32(field_index))
    }
}
```

### String Block

The string block contains null-terminated UTF-8 strings referenced by offset:

```
Offset  Content
0x0000  \0              (empty string)
0x0001  "Fireball\0"
0x000A  "Frost Bolt\0"
0x0015  "Healing Touch\0"
```

## Data Types

### Basic Types

All fields in DBC files are 4 bytes, interpreted as:

| Type | Size | Description |
|------|------|-------------|
| `u32` | 4 bytes | Unsigned integer |
| `i32` | 4 bytes | Signed integer |
| `f32` | 4 bytes | IEEE 754 single-precision float |
| `StringRef` | 4 bytes | Offset into string block |

### Special Types

```rust
/// Reference to another DBC record
type RecordId = u32;

/// Bit flags
type Flags = u32;

/// String reference (offset into string block)
type StringRef = u32;

/// Unused/padding field
type Padding = u32;
```

### Localized Strings

Localized strings use a special pattern of 16 consecutive fields plus a flags field:

```rust
#[repr(C)]
struct LocalizedString {
    /// String references for each locale (16 locales)
    locale_strings: [StringRef; 16],

    /// Bitmask of locales present
    locale_mask: u32,
}

/// Locale indices
const LOCALE_EN_US: usize = 0;   // English (US)
const LOCALE_KO_KR: usize = 1;   // Korean
const LOCALE_FR_FR: usize = 2;   // French
const LOCALE_DE_DE: usize = 3;   // German
const LOCALE_EN_CN: usize = 4;   // English (China)
const LOCALE_EN_TW: usize = 5;   // English (Taiwan)
const LOCALE_ES_ES: usize = 6;   // Spanish (Spain)
const LOCALE_ES_MX: usize = 7;   // Spanish (Mexico)
const LOCALE_RU_RU: usize = 8;   // Russian
const LOCALE_JA_JP: usize = 9;   // Japanese
const LOCALE_PT_PT: usize = 10;  // Portuguese
const LOCALE_IT_IT: usize = 11;  // Italian
const LOCALE_UNKNOWN_12: usize = 12;
const LOCALE_UNKNOWN_13: usize = 13;
const LOCALE_UNKNOWN_14: usize = 14;
const LOCALE_UNKNOWN_15: usize = 15;
```

## String Handling

### String References

Strings are referenced by their byte offset in the string block:

```rust
/// Resolve a string reference
fn get_string(string_block: &[u8], string_ref: StringRef) -> Result<&str, DbcError> {
    if string_ref == 0 {
        return Ok(""); // Null reference
    }

    let offset = string_ref as usize;
    if offset >= string_block.len() {
        return Err(DbcError::InvalidStringRef(string_ref));
    }

    // Find null terminator
    let string_data = &string_block[offset..];
    let null_pos = string_data.iter()
        .position(|&b| b == 0)
        .ok_or(DbcError::UnterminatedString)?;

    // Convert to UTF-8
    std::str::from_utf8(&string_data[..null_pos])
        .map_err(|_| DbcError::InvalidUtf8)
}
```

### Localization

The locale mask indicates which locales have valid strings:

```rust
/// Check if a locale has a valid string
fn has_locale(locale_mask: u32, locale_index: usize) -> bool {
    if locale_index >= 16 {
        return false;
    }
    (locale_mask & (1 << locale_index)) != 0
}

/// Get the best available locale string
fn get_best_locale_string(
    loc_string: &LocalizedString,
    preferred_locale: usize,
    string_block: &[u8]
) -> Result<String, DbcError> {
    // Try preferred locale first
    if loc_string.locale_strings[preferred_locale] != 0 {
        return get_string(string_block, loc_string.locale_strings[preferred_locale])
            .map(|s| s.to_string());
    }

    // Fall back to enUS
    if loc_string.locale_strings[LOCALE_EN_US] != 0 {
        return get_string(string_block, loc_string.locale_strings[LOCALE_EN_US])
            .map(|s| s.to_string());
    }

    // Try any available locale
    for i in 0..16 {
        if loc_string.locale_strings[i] != 0 {
            return get_string(string_block, loc_string.locale_strings[i])
                .map(|s| s.to_string());
        }
    }

    Ok(String::new())
}
```

## Common DBC Files

### Spell.dbc

Contains spell definitions (very large structure with ~240 fields in WotLK):

```rust
#[repr(C)]
struct SpellRecord {
    id: u32,                          // Spell ID
    category: u32,                    // Spell category
    dispel_type: u32,                 // Dispel type (magic, curse, etc.)
    mechanic: u32,                    // Spell mechanic
    attributes: [u32; 7],             // Spell attributes (7 fields)
    stances: u32,                     // Required stances
    stances_not: u32,                 // Excluded stances
    targets: u32,                     // Valid targets
    target_creature_type: u32,        // Target creature type mask
    requires_spell_focus: u32,        // Required spell focus object
    facing_caster_flags: u32,         // Facing requirements
    caster_aura_state: u32,           // Required aura state
    target_aura_state: u32,           // Target aura state
    caster_aura_state_not: u32,       // Excluded caster aura state
    target_aura_state_not: u32,       // Excluded target aura state
    caster_aura_spell: u32,           // Required aura spell
    target_aura_spell: u32,           // Target aura spell
    exclude_caster_aura_spell: u32,   // Excluded caster aura
    exclude_target_aura_spell: u32,   // Excluded target aura
    casting_time_index: u32,          // Index into CastingTime.dbc
    recovery_time: u32,               // Recovery time
    category_recovery_time: u32,      // Category recovery time
    interrupt_flags: u32,             // Interrupt flags
    aura_interrupt_flags: u32,        // Aura interrupt flags
    channel_interrupt_flags: u32,     // Channel interrupt flags
    proc_flags: u32,                  // Proc event flags
    proc_chance: u32,                 // Proc chance
    proc_charges: u32,                // Proc charges
    max_level: u32,                   // Maximum level
    base_level: u32,                  // Base level
    spell_level: u32,                 // Spell level
    duration_index: u32,              // Index into Duration.dbc
    power_type: u32,                  // Power type (mana, rage, etc.)
    mana_cost: u32,                   // Mana cost
    mana_cost_perlevel: u32,          // Mana cost per level
    mana_per_second: u32,             // Mana per second
    mana_per_second_per_level: u32,   // Mana per second per level
    range_index: u32,                 // Index into Range.dbc
    speed: f32,                       // Projectile speed
    modal_next_spell: u32,            // Next spell in sequence
    stack_amount: u32,                // Stack amount
    totem: [u32; 2],                  // Required totems
    reagent: [i32; 8],                // Required reagents
    reagent_count: [u32; 8],          // Reagent counts
    equipped_item_class: i32,         // Required item class
    equipped_item_subclass_mask: i32, // Required item subclass
    equipped_item_inventory_type_mask: i32, // Required inventory type
    effect: [u32; 3],                 // Spell effects
    effect_die_sides: [i32; 3],       // Effect die sides
    effect_real_points_per_level: [f32; 3], // Points per level
    effect_base_points: [i32; 3],     // Base points
    effect_mechanic: [u32; 3],        // Effect mechanics
    effect_implicit_target_a: [u32; 3], // Implicit targets A
    effect_implicit_target_b: [u32; 3], // Implicit targets B
    effect_radius_index: [u32; 3],    // Radius indices
    effect_apply_aura_name: [u32; 3], // Aura types
    effect_amplitude: [u32; 3],       // Effect amplitude
    effect_multiple_value: [f32; 3],  // Multiple value
    effect_chain_target: [u32; 3],    // Chain targets
    effect_item_type: [u32; 3],       // Created items
    effect_misc_value: [i32; 3],      // Misc values
    effect_misc_value_b: [i32; 3],    // Misc values B
    effect_trigger_spell: [u32; 3],   // Triggered spells
    effect_points_per_combo_point: [f32; 3], // Points per combo
    effect_spell_class_mask_a: [u32; 3], // Class mask A
    effect_spell_class_mask_b: [u32; 3], // Class mask B
    effect_spell_class_mask_c: [u32; 3], // Class mask C
    spell_visual: [u32; 2],           // Visual effects
    spell_icon_id: u32,               // Icon ID
    active_icon_id: u32,              // Active icon ID
    spell_priority: u32,              // Priority
    spell_name: LocalizedString,      // Spell name (17 fields)
    spell_rank: LocalizedString,      // Spell rank (17 fields)
    spell_description: LocalizedString, // Description (17 fields)
    spell_tooltip: LocalizedString,   // Tooltip (17 fields)
    mana_cost_percentage: u32,        // Mana cost percentage
    start_recovery_category: u32,     // Recovery category
    start_recovery_time: u32,         // Recovery time
    max_target_level: u32,            // Max target level
    spell_family_name: u32,           // Spell family
    spell_family_flags: [u32; 3],     // Family flags
    max_affected_targets: u32,        // Max targets
    dmg_class: u32,                   // Damage class
    prevention_type: u32,             // Prevention type
    stance_bar_order: u32,            // Stance bar position
    dmg_multiplier: [f32; 3],         // Damage multipliers
    min_faction_id: u32,              // Min faction ID
    min_reputation: u32,              // Min reputation
    required_aura_vision: u32,        // Required aura vision
    totem_category: [u32; 2],         // Totem categories
    area_group_id: u32,               // Area group
    school_mask: u32,                 // School mask
    rune_cost_id: u32,                // Rune cost ID
    spell_missile_id: u32,            // Missile ID
    power_display_id: u32,            // Power display
    effect_bonus_multiplier: [f32; 3], // Bonus multipliers
    spell_description_variable_id: u32, // Description variable
    spell_difficulty_id: u32,         // Difficulty ID
}
```

### Item.dbc

Contains item template data:

```rust
#[repr(C)]
struct ItemRecord {
    id: u32,                    // Item ID
    class: u32,                 // Item class (weapon, armor, etc.)
    subclass: u32,              // Item subclass
    sound_override_subclass: i32, // Sound override
    material: u32,              // Material type
    display_id: u32,            // Display info ID
    inventory_type: u32,        // Equipment slot
    sheath_type: u32,           // Sheath animation type
}

enum ItemClass {
    Consumable = 0,
    Container = 1,
    Weapon = 2,
    Gem = 3,
    Armor = 4,
    Reagent = 5,
    Projectile = 6,
    TradeGoods = 7,
    Generic = 8,
    Recipe = 9,
    Money = 10,
    Quiver = 11,
    Quest = 12,
    Key = 13,
    Permanent = 14,
    Misc = 15,
    Glyph = 16,
}
```

### Map.dbc

Contains map/instance information:

```rust
#[repr(C)]
struct MapRecord {
    id: u32,                    // Map ID
    directory: StringRef,       // Map directory name
    instance_type: u32,         // Instance type (world, dungeon, raid, etc.)
    flags: u32,                 // Map flags
    pvp: u32,                   // PvP type
    map_name: LocalizedString,  // Map name (17 fields)
    area_table_id: u32,         // Link to AreaTable.dbc
    map_description0: LocalizedString, // Description (17 fields)
    map_description1: LocalizedString, // Description (17 fields)
    loading_screen: u32,        // Loading screen ID
    minimap_icon_scale: f32,    // Minimap icon scale
    corpse_map_id: u32,         // Corpse location map
    corpse_x: f32,              // Corpse X coordinate
    corpse_y: f32,              // Corpse Y coordinate
    time_of_day_override: u32,  // Time override
    expansion_id: u32,          // Required expansion
    raid_offset: u32,           // Raid reset offset
    max_players: u32,           // Maximum players
}
```

### AreaTable.dbc

Contains zone/area information:

```rust
#[repr(C)]
struct AreaTableRecord {
    id: u32,                          // Area ID
    map_id: u32,                      // Map ID
    parent_area_id: u32,              // Parent area ID
    area_bit: u32,                    // Area bit for exploration
    flags: u32,                       // Area flags
    sound_preferences: u32,           // Sound preferences
    sound_preferences_underwater: u32, // Underwater sound
    sound_ambience: u32,              // Ambience sound
    zone_music: u32,                  // Zone music
    zone_intro_music: u32,            // Intro music
    exploration_level: u32,           // Min level for exploration XP
    area_name: LocalizedString,       // Area name (17 fields)
    faction_group_mask: u32,          // Faction group
    liquid_type_id: [u32; 4],         // Liquid types
    min_elevation: f32,               // Minimum elevation
    ambient_multiplier: f32,          // Ambient light multiplier
    light_id: u32,                    // Light parameters
}
```

## Reading DBC Files

### Algorithm

1. Read and validate header
2. Allocate memory for records
3. Read all records sequentially
4. Read string block
5. Build any necessary indices

### Implementation Example - ✅ Implemented

```rust
use std::fs::File;
use std::io::BufReader;
use wow_cdbc::{DbcParser, FieldType, Schema, SchemaField};

// Open a DBC file  
let file = File::open("SpellItemEnchantment.dbc")?;
let mut reader = BufReader::new(file);

// Parse the DBC file
let parser = DbcParser::parse(&mut reader)?;

// Print header information
let header = parser.header();
println!("Record Count: {}", header.record_count);
println!("Field Count: {}", header.field_count);

// Define a schema for SpellItemEnchantment.dbc
let mut schema = Schema::new("SpellItemEnchantment");
schema.add_field(SchemaField::new("ID", FieldType::UInt32));
schema.add_field(SchemaField::new("Charges", FieldType::UInt32));
schema.add_field(SchemaField::new("Description", FieldType::String));
schema.set_key_field("ID");

// Apply the schema and parse records
let parser = parser.with_schema(schema)?;
let record_set = parser.parse_records()?;
```

## Writing DBC Files - ❌ Not Implemented

### Algorithm

1. Build string block with deduplication
2. Calculate header values
3. Write header
4. Write records with updated string references
5. Write string block

DBC writing functionality is not currently implemented in the `wow-cdbc` crate.

## Performance Considerations - ✅ Implemented

### Memory Mapping

For large DBC files, consider memory mapping:
    /// Create a new DBC builder
    pub fn new(field_count: u32) -> Self {
        let mut builder = DbcBuilder {
            field_count,
            records: Vec::new(),
            strings: HashMap::new(),
            string_data: vec![0], // Start with null string
        };

        // Add empty string at offset 0
        builder.strings.insert(String::new(), 0);

        builder
    }

    /// Add a string to the string block
    pub fn add_string(&mut self, string: &str) -> u32 {
        if string.is_empty() {
            return 0;
        }

        // Check if string already exists
        if let Some(&offset) = self.strings.get(string) {
            return offset;
        }

        // Add new string
        let offset = self.string_data.len() as u32;
        self.string_data.extend_from_slice(string.as_bytes());
        self.string_data.push(0); // Null terminator

        self.strings.insert(string.to_string(), offset);
        offset
    }

    /// Add a record
    pub fn add_record(&mut self, fields: Vec<u32>) -> Result<(), DbcError> {
        if fields.len() != self.field_count as usize {
            return Err(DbcError::InvalidHeader);
        }

        self.records.push(fields);
        Ok(())
    }

    /// Write the DBC file
    pub fn write_file(&self, path: &str) -> Result<(), DbcError> {
        let file = File::create(path).map_err(DbcError::Io)?;
        let mut writer = BufWriter::new(file);

        // Build header
        let header = DbcHeader {
            magic: *b"WDBC",
            record_count: self.records.len() as u32,
            field_count: self.field_count,
            record_size: self.field_count * 4,
            string_block_size: self.string_data.len() as u32,
        };

        // Write header
        writer.write_all(&header.magic).map_err(DbcError::Io)?;
        writer.write_all(&header.record_count.to_le_bytes()).map_err(DbcError::Io)?;
        writer.write_all(&header.field_count.to_le_bytes()).map_err(DbcError::Io)?;
        writer.write_all(&header.record_size.to_le_bytes()).map_err(DbcError::Io)?;
        writer.write_all(&header.string_block_size.to_le_bytes()).map_err(DbcError::Io)?;

        // Write records
        for record in &self.records {
            for &field in record {
                writer.write_all(&field.to_le_bytes()).map_err(DbcError::Io)?;
            }
        }

        // Write string block
        writer.write_all(&self.string_data).map_err(DbcError::Io)?;

        writer.flush().map_err(DbcError::Io)?;
        Ok(())
    }
}

/// Example: Create a simple Item.dbc
fn create_item_dbc() -> Result<(), DbcError> {
    let mut builder = DbcBuilder::new(8); // Item.dbc has 8 fields

    // Add Hearthstone
    builder.add_record(vec![
        6948,  // ID
        15,    // Class (Miscellaneous)
        0,     // Subclass
        0,     // Sound override
        0,     // Material
        6418,  // Display ID
        0,     // Inventory type
        0,     // Sheath type
    ])?;

    // Add Thunderfury
    builder.add_record(vec![
        19019, // ID
        2,     // Class (Weapon)
        7,     // Subclass (1H Sword)
        1,     // Sound override
        1,     // Material (Metal)
        30606, // Display ID
        13,    // Inventory type (1H Weapon)
        3,     // Sheath type
    ])?;

    builder.write_file("Item.dbc")?;
    Ok(())
}
```

## Performance Considerations

### Memory Mapping

For large DBC files, consider memory mapping:

```rust
use memmap2::MmapOptions;

/// Memory-mapped DBC reader
pub struct MappedDbc {
    mmap: memmap2::Mmap,
    header: DbcHeader,
}

impl MappedDbc {
    pub fn open(path: &str) -> Result<Self, DbcError> {
        let file = File::open(path).map_err(DbcError::Io)?;
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(DbcError::Io)?
        };

        // Parse header from mmap
        let header = Self::parse_header(&mmap)?;

        Ok(MappedDbc { mmap, header })
    }

    fn parse_header(data: &[u8]) -> Result<DbcHeader, DbcError> {
        if data.len() < 20 {
            return Err(DbcError::InvalidHeader);
        }

        Ok(DbcHeader {
            magic: [data[0], data[1], data[2], data[3]],
            record_count: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            field_count: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            record_size: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            string_block_size: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
        })
    }
}
```

### Indexing and Caching

Build indices for frequently accessed fields:

```rust
struct DbcIndex<T> {
    by_id: HashMap<u32, usize>,
    by_name: HashMap<String, Vec<usize>>,
    records: Vec<T>,
}

impl<T: DbcRecord> DbcIndex<T> {
    fn build(dbc: DbcFile<T>) -> Self {
        let mut by_id = HashMap::new();
        let mut by_name = HashMap::new();

        for (idx, record) in dbc.records.iter().enumerate() {
            by_id.insert(record.id(), idx);

            if let Some(name) = record.name() {
                by_name.entry(name.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        DbcIndex {
            by_id,
            by_name,
            records: dbc.records,
        }
    }
}
```

## Implementation Notes

### Memory Alignment

DBC files use packed structures with no padding:

```rust
#[repr(C, packed)]
struct PackedRecord {
    // Fields are tightly packed
}
```

### Endianness

All multi-byte values are little-endian:

```rust
fn read_u32_le(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}
```

### String Block Organization

The string block is typically organized as:

1. Empty string at offset 0 (for null references)
2. Strings in order of first reference
3. No duplicate strings (string pooling)

## Common Patterns

### DBC Validation

```rust
struct DbcValidator {
    errors: Vec<ValidationError>,
}

impl DbcValidator {
    fn validate_spell(&mut self, spell: &SpellRecord, db: &DbcDatabase) {
        // Check foreign key references
        for &reagent_id in &spell.reagent {
            if reagent_id > 0 && !db.items.by_id.contains_key(&(reagent_id as u32)) {
                self.errors.push(ValidationError::InvalidReference {
                    table: "Spell",
                    field: "reagent",
                    id: spell.id,
                    ref_id: reagent_id as u32,
                });
            }
        }

        // Validate spell schools
        if spell.school_mask == 0 {
            self.errors.push(ValidationError::InvalidValue {
                table: "Spell",
                field: "school_mask",
                id: spell.id,
                reason: "No spell school defined",
            });
        }
    }
}
```

### Cross-Reference Resolution

```rust
struct DbcDatabase {
    items: DbcIndex<ItemRecord>,
    spells: DbcIndex<SpellRecord>,
    item_display: DbcIndex<ItemDisplayInfoRecord>,
    // ... more tables
}

impl DbcDatabase {
    fn resolve_item_display(&self, item: &ItemRecord) -> Option<&ItemDisplayInfoRecord> {
        self.item_display.by_id.get(&item.display_info_id)
            .map(|&idx| &self.item_display.records[idx])
    }

    fn get_item_spells(&self, item: &ItemRecord) -> Vec<&SpellRecord> {
        item.spell_trigger.iter()
            .filter_map(|&spell_id| {
                if spell_id > 0 {
                    self.spells.by_id.get(&(spell_id as u32))
                        .map(|&idx| &self.spells.records[idx])
                } else {
                    None
                }
            })
            .collect()
    }
}
```

## Test Vectors

### Header Verification

Valid DBC header bytes:

```text
57 44 42 43  // "WDBC"
0A 00 00 00  // 10 records
05 00 00 00  // 5 fields
14 00 00 00  // 20 bytes per record
64 00 00 00  // 100 bytes string block
```

### String Reference Tests

String block with test data:

```text
Offset  Hex                          ASCII
0x0000  00                           .           (null string)
0x0001  48 65 6C 6C 6F 00            Hello.
0x0007  57 6F 72 6C 64 00            World.
0x000D  54 65 73 74 20 31 32 33 00   Test 123.
```

Test cases:

- StringRef(0) → ""
- StringRef(1) → "Hello"
- StringRef(7) → "World"
- StringRef(13) → "Test 123"

### Localization Tests

LocalizedString test data (17 fields):

```text
Field   Value       Description
0-15    StringRef   Locale strings
16      0x0009      Locale mask (enUS and deDE present)
```

## Common Issues

### String Encoding

- Strings are null-terminated UTF-8
- Check string block bounds
- Handle missing translations
- Validate UTF-8 encoding

### Schema Changes

- Field counts vary by version
- New fields often added at end
- Some fields repurposed between versions
- Use version-specific schemas

### Data Integrity

- Validate foreign key references
- Check for orphaned records
- Verify enum values are valid
- Handle circular references

## Version Differences

### Classic (1.12.x)

- Original WDBC format
- 16 locale fields in LocalizedString
- Basic spell, item, and zone data

### The Burning Crusade (2.4.3)

- Same file format as Classic
- New DBC files for:
  - Flying mounts
  - Heroic dungeons
  - Arena data
  - Jewelcrafting
- Extended spell attributes (2 more attribute fields)

### Wrath of the Lich King (3.3.5)

- Still uses WDBC format
- New DBC files for:
  - Achievement data
  - Vehicle data
  - Glyphs
  - Extended quest data
- Spell.dbc grew to ~240 fields

### Cataclysm+ (4.x+)

- Introduced DB2 format
- Variable record sizes
- Field types in header
- Inline strings
- Relationship data

## References

- [WoWDev Wiki - DBC](https://wowdev.wiki/DBC)
- [TrinityCore DBC Structures](https://github.com/TrinityCore/TrinityCore/tree/master/src/server/game/DataStores)
- [WoWDBDefs](https://github.com/wowdev/WoWDBDefs)
- [Modcraft DBC Tutorial](https://model-changing.net/tutorials/article/50-dbc-editing-basics/)

## See Also

- [DBC Extraction Guide](../../guides/dbc-extraction.md)
- [DBC Schema Guide](../../guides/dbc-schemas.md)
- [Data Mining Guide](../../guides/data-mining.md)
- [Localization Guide](../../guides/localization.md)
