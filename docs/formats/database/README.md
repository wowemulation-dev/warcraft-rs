# Database Formats

Database formats store game data in structured, tabular formats.

## Supported Formats

### [DBC Format](dbc.md)

**DataBase Client** - Client-side database files containing game data.

- Fixed-size records
- String block for text data
- Indexed by ID
- Localization support

## DBC Structure

DBCs are essentially binary CSV files with a header:

```text
Header (20 bytes)
Records (n × record_size)
String Block (variable)
```

## Common DBC Files

### Core Game Data

- `Item.dbc` - Item definitions
- `Spell.dbc` - Spell data
- `CreatureDisplayInfo.dbc` - Creature models
- `Map.dbc` - Map definitions

### Display Data

- `ItemDisplayInfo.dbc` - Item visuals
- `CharSections.dbc` - Character customization
- `CreatureModelData.dbc` - Model parameters

### Game Mechanics

- `Talent.dbc` - Talent trees
- `SkillLine.dbc` - Skills and professions
- `Achievement.dbc` - Achievement data

## Usage Patterns

### Reading DBC Files

```rust
use warcraft_rs::dbc::{Dbc, DbcReader};

// Generic DBC reading
let dbc = Dbc::open("DBFilesClient/Item.dbc")?;
println!("Records: {}", dbc.record_count());

// Typed reading
let items: DbcReader<Item> = DbcReader::open("DBFilesClient/Item.dbc")?;
for item in items {
    println!("Item {}: {}", item.id, item.name);
}
```

### Working with Strings

```rust
// DBCs store strings as offsets into a string block
let name = dbc.get_string(record.name_offset)?;

// Localized strings have multiple offsets
let localized_name = dbc.get_localized_string(
    record.name_offsets,
    Locale::EnUS
)?;
```

### Indexing and Lookups

```rust
use warcraft_rs::dbc::IndexedDbc;

// Build index for fast lookups
let items = IndexedDbc::<Item>::open("DBFilesClient/Item.dbc")?;

// O(1) lookup by ID
if let Some(item) = items.get(12345) {
    println!("Found: {}", item.name);
}
```

## Localization

WoW supports 16 locales in DBC files:

| ID | Locale | Description |
|----|--------|-------------|
| 0 | enUS | English (US) |
| 1 | koKR | Korean |
| 2 | frFR | French |
| 3 | deDE | German |
| 4 | zhCN | Chinese (Simplified) |
| 5 | zhTW | Chinese (Traditional) |
| 6 | esES | Spanish (Spain) |
| 7 | esMX | Spanish (Mexico) |
| 8 | ruRU | Russian |

## Schema Definitions

Define DBC schemas using derive macros:

```rust
#[derive(DbcRecord)]
struct Item {
    #[dbc(id)]
    id: u32,
    class: u32,
    subclass: u32,
    #[dbc(string)]
    name: String,
    #[dbc(localized_string)]
    description: LocalizedString,
    display_id: u32,
    quality: u32,
    flags: u32,
}
```

## Performance Considerations

1. **Memory Usage**: Large DBCs can use significant memory
2. **Indexing**: Build indexes for frequently accessed data
3. **Caching**: Cache parsed records
4. **Lazy Loading**: Load strings on demand

## Tools

- `warcraft-dbc` - CLI tool for DBC operations
- DBC to CSV conversion
- Schema generation from DBCs

## See Also

- [DBC Data Extraction Guide](../../guides/dbc-extraction.md)
- [DBC Schema Reference](../../api/dbc-schemas.md)
- [Localization Guide](../../guides/localization.md)
