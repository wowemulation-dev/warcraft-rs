# DBC Format 📊

DBC (DataBase Client) files store client-side game data in a structured table
format, containing everything from spell information to item stats.

## Overview

- **Extension**: `.dbc`
- **Purpose**: Client-side database tables
- **Structure**: Fixed-size records with defined schemas
- **Encoding**: Little-endian binary format
- **String Storage**: Separate string block with offset references

## Version History

- **Classic (1.12.1)**: Original DBC format
- **TBC (2.4.3)**: Added new fields to many tables
- **WotLK (3.3.5)**: Expanded format, more tables
- **Cataclysm (4.3.4)**: Major schema changes
- **MoP (5.4.8)**: Added localization support

## Structure

### DBC Header

```rust
struct DBCHeader {
    magic: [u8; 4],          // "WDBC"
    record_count: u32,       // Number of records
    field_count: u32,        // Number of fields per record
    record_size: u32,        // Size of each record in bytes
    string_block_size: u32,  // Size of string block
}

struct DBCFile<T> {
    header: DBCHeader,
    records: Vec<T>,
    string_block: Vec<u8>,
}
```

### Common Field Types

```rust
// DBC field types
type DBCString = u32;        // Offset into string block
type DBCLocalizedString = [u32; 16]; // 16 locales + flags

enum DBCFieldType {
    Int32,
    Float,
    String,                  // Offset into string block
    LocalizedString,         // Multiple language strings
    Flag,                    // Bitfield
    ForeignKey(String),      // Reference to another DBC
}
```

## Usage Example

```rust
use warcraft_rs::dbc::{DBCFile, ItemRecord, SpellRecord};

// Load Item.dbc
let item_dbc = DBCFile::<ItemRecord>::open("DBFilesClient/Item.dbc")?;

// Access records
for item in item_dbc.records() {
    println!("Item {}: {}", item.id, item.name(Locale::English));
    println!("  Quality: {:?}", item.quality);
    println!("  Item Level: {}", item.item_level);
}

// Find specific item
let thunderfury = item_dbc.find_by_id(19019)?;
println!("Found: {}", thunderfury.name(Locale::English));

// Query items
let epic_weapons = item_dbc.filter(|item| {
    item.quality == ItemQuality::Epic &&
    item.class == ItemClass::Weapon
});

// Load related DBCs
let spell_dbc = DBCFile::<SpellRecord>::open("DBFilesClient/Spell.dbc")?;

// Follow foreign key reference
if let Some(spell_id) = item.spell_trigger[0] {
    let spell = spell_dbc.find_by_id(spell_id)?;
    println!("On Use: {}", spell.name(Locale::English));
}
```

## Common DBC Files

### Item.dbc

```rust
#[repr(C)]
struct ItemRecord {
    id: u32,
    class: u32,              // ItemClass enum
    subclass: u32,           // ItemSubclass enum
    sound_override_subclass: i32,
    material: u32,           // ItemMaterial enum
    display_info_id: u32,    // ItemDisplayInfo.dbc
    inventory_type: u32,     // InventoryType enum
    sheath_type: u32,        // SheathType enum
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

### Spell.dbc

```rust
struct SpellRecord {
    id: u32,
    category: u32,
    dispel_type: u32,
    mechanic: u32,
    attributes: [u32; 8],    // Spell attribute flags
    stances: u32,            // Allowed stances
    stances_not: u32,        // Disallowed stances
    targets: u32,            // Valid targets
    target_creature_type: u32,
    requires_spell_focus: u32,
    facing_caster_flags: u32,
    caster_aura_state: u32,
    target_aura_state: u32,
    caster_aura_state_not: u32,
    target_aura_state_not: u32,
    caster_aura_spell: u32,
    target_aura_spell: u32,
    exclude_caster_aura_spell: u32,
    exclude_target_aura_spell: u32,
    casting_time_index: u32,
    recovery_time: u32,
    category_recovery_time: u32,
    interrupt_flags: u32,
    aura_interrupt_flags: u32,
    channel_interrupt_flags: u32,
    proc_flags: u32,
    proc_chance: u32,
    proc_charges: u32,
    max_level: u32,
    base_level: u32,
    spell_level: u32,
    duration_index: u32,
    power_type: u32,
    mana_cost: u32,
    mana_cost_perlevel: u32,
    mana_per_second: u32,
    mana_per_second_per_level: u32,
    range_index: u32,
    speed: f32,
    modal_next_spell: u32,
    stack_amount: u32,
    totem: [u32; 2],
    reagent: [i32; 8],
    reagent_count: [u32; 8],
    equipped_item_class: i32,
    equipped_item_sub_class_mask: i32,
    equipped_item_inventory_type_mask: i32,
    effect: [u32; 3],        // SpellEffect enum
    effect_die_sides: [i32; 3],
    effect_real_points_per_level: [f32; 3],
    effect_base_points: [i32; 3],
    effect_mechanic: [u32; 3],
    effect_implicit_target_a: [u32; 3],
    effect_implicit_target_b: [u32; 3],
    effect_radius_index: [u32; 3],
    effect_apply_aura_name: [u32; 3],
    effect_amplitude: [u32; 3],
    effect_multiple_value: [f32; 3],
    effect_chain_target: [u32; 3],
    effect_item_type: [u32; 3],
    effect_misc_value: [i32; 3],
    effect_misc_value_b: [i32; 3],
    effect_trigger_spell: [u32; 3],
    effect_points_per_combo_point: [f32; 3],
    effect_spell_class_mask_a: [u32; 3],
    effect_spell_class_mask_b: [u32; 3],
    effect_spell_class_mask_c: [u32; 3],
    visual: [u32; 2],
    spell_icon_id: u32,
    active_icon_id: u32,
    spell_priority: u32,
    spell_name: DBCLocalizedString,
    spell_name_flag: u32,
    rank: DBCLocalizedString,
    rank_flags: u32,
    description: DBCLocalizedString,
    description_flags: u32,
    tooltip: DBCLocalizedString,
    tooltip_flags: u32,
    mana_cost_percentage: u32,
    start_recovery_category: u32,
    start_recovery_time: u32,
    max_target_level: u32,
    spell_family_name: u32,
    spell_family_flags: [u32; 3],
    max_affected_targets: u32,
    dmg_class: u32,
    prevention_type: u32,
    stance_bar_order: u32,
    dmg_multiplier: [f32; 3],
    min_faction_id: u32,
    min_reputation: u32,
    required_aura_vision: u32,
    totem_category: [u32; 2],
    area_group_id: u32,
    school_mask: u32,
    rune_cost_id: u32,
    spell_missile_id: u32,
    power_display_id: u32,
    unk1: [u32; 3],
    spell_description_variable_id: u32,
    spell_difficulty_id: u32,
}
```

## Advanced Features

### Localization Support

```rust
#[derive(Debug, Clone, Copy)]
enum Locale {
    English = 0,
    Korean = 1,
    French = 2,
    German = 3,
    Chinese = 4,
    Taiwanese = 5,
    Spanish = 6,
    SpanishMX = 7,
    Russian = 8,
    Unknown9 = 9,
    Portuguese = 10,
    Italian = 11,
}

impl DBCLocalizedString {
    fn get_string(&self, locale: Locale, string_block: &[u8]) -> String {
        let offset = self[locale as usize] as usize;
        if offset == 0 {
            // Fallback to English
            let en_offset = self[0] as usize;
            read_cstring(string_block, en_offset)
        } else {
            read_cstring(string_block, offset)
        }
    }
}
```

### Indexing and Caching

```rust
struct DBCIndex<T> {
    by_id: HashMap<u32, usize>,
    by_name: HashMap<String, Vec<usize>>,
    records: Vec<T>,
}

impl<T: DBCRecord> DBCIndex<T> {
    fn build(dbc: DBCFile<T>) -> Self {
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

        DBCIndex {
            by_id,
            by_name,
            records: dbc.records,
        }
    }

    fn find_by_name(&self, name: &str) -> Vec<&T> {
        self.by_name.get(&name.to_lowercase())
            .map(|indices| {
                indices.iter()
                    .map(|&idx| &self.records[idx])
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

### Cross-Reference Resolution

```rust
struct DBCDatabase {
    items: DBCIndex<ItemRecord>,
    spells: DBCIndex<SpellRecord>,
    item_display: DBCIndex<ItemDisplayInfoRecord>,
    // ... more tables
}

impl DBCDatabase {
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

### Custom DBC Schemas

```rust
// Define custom DBC schema
#[derive(DBCRecord)]
#[dbc_name = "CustomData.dbc"]
struct CustomDataRecord {
    #[dbc_id]
    id: u32,

    #[dbc_string]
    name: DBCString,

    #[dbc_float]
    value: f32,

    #[dbc_foreign_key("Item.dbc")]
    item_id: u32,

    #[dbc_flags]
    flags: u32,
}

// Auto-generate parser
let custom_dbc = DBCFile::<CustomDataRecord>::open("CustomData.dbc")?;
```

## Common Patterns

### DBC Validation

```rust
struct DBCValidator {
    errors: Vec<ValidationError>,
}

impl DBCValidator {
    fn validate_spell(&mut self, spell: &SpellRecord, db: &DBCDatabase) {
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

### DBC Export/Import

```rust
struct DBCExporter {
    delimiter: char,
    include_headers: bool,
}

impl DBCExporter {
    fn export_csv<T: DBCRecord>(&self, dbc: &DBCFile<T>, output: &mut Write) -> Result<()> {
        if self.include_headers {
            writeln!(output, "{}", T::csv_headers().join(&self.delimiter.to_string()))?;
        }

        for record in &dbc.records {
            let values = record.to_csv_values(&dbc.string_block);
            writeln!(output, "{}", values.join(&self.delimiter.to_string()))?;
        }

        Ok(())
    }
}
```

### Performance Optimization

```rust
struct DBCCache {
    cache: LruCache<(String, u32), Arc<dyn Any>>,
    preloaded: HashSet<String>,
}

impl DBCCache {
    fn get_record<T: DBCRecord + 'static>(&mut self, table: &str, id: u32) -> Result<Arc<T>> {
        let key = (table.to_string(), id);

        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone().downcast::<T>().unwrap());
        }

        // Load from disk
        let dbc = self.load_dbc::<T>(table)?;
        let record = dbc.find_by_id(id)?;
        let arc_record = Arc::new(record.clone());

        self.cache.put(key, arc_record.clone() as Arc<dyn Any>);
        Ok(arc_record)
    }

    fn preload_common_tables(&mut self) -> Result<()> {
        let common_tables = ["Spell", "Item", "ItemDisplayInfo", "CreatureDisplayInfo"];

        for table in &common_tables {
            // Load entire DBC into cache
            self.preload_table(table)?;
            self.preloaded.insert(table.to_string());
        }

        Ok(())
    }
}
```

## Performance Tips

- Index frequently accessed fields
- Cache string lookups
- Use memory-mapped files for large DBCs
- Implement lazy loading for related data
- Consider binary search for sorted data

## Common Issues

### String Encoding

- Strings are null-terminated UTF-8
- Check string block bounds
- Handle missing translations

### Schema Changes

- Field counts vary by version
- New fields often added at end
- Some fields repurposed between versions

### Data Integrity

- Validate foreign key references
- Check for orphaned records
- Verify enum values are valid

## References

- [DBC Format (wowdev.wiki)](https://wowdev.wiki/DBC)
- [DBC File List](https://wowdev.wiki/Category:DBC)
- [Trinity Core DBC Structures](https://github.com/TrinityCore/TrinityCore/tree/master/src/server/game/DataStores)

## See Also

- [DBC Schema Guide](../../guides/dbc-schemas.md)
- [Data Mining Guide](../../guides/data-mining.md)
- [Localization Guide](../../guides/localization.md)
