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

DBCs are binary tabular files with a header:

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
use wow_cdbc::DbcParser;
use std::io::BufReader;
use std::fs::File;

let file = File::open("DBFilesClient/Item.dbc")?;
let mut reader = BufReader::new(file);
let parser = DbcParser::parse(&mut reader)?;

let header = parser.header();
println!("Records: {}", header.record_count);
println!("Fields per record: {}", header.field_count);
```

### Schema-Based Parsing

```rust
use wow_cdbc::{DbcParser, Schema, SchemaField, FieldType};

let parser = DbcParser::parse(&mut reader)?;

// Define a schema for the DBC file
let mut schema = Schema::new("SpellItemEnchantment");
schema.add_field(SchemaField::new("ID", FieldType::UInt32));
schema.add_field(SchemaField::new("Charges", FieldType::UInt32));
schema.add_field(SchemaField::new("Description", FieldType::String));
schema.set_key_field("ID");

// Apply the schema and parse records
let parser = parser.with_schema(schema)?;
let record_set = parser.parse_records()?;

for record in record_set.iter() {
    println!("{:?}", record);
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

## Tools

The CLI provides DBC operations:

```bash
warcraft-rs dbc info Item.dbc
warcraft-rs dbc export Item.dbc --format csv
```

## See Also

- [DBC Data Extraction Guide](../../guides/dbc-extraction.md)
