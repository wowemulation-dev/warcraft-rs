# 📝 DBC Data Extraction

## Overview

DBC (DataBase Client) files contain game data in a structured table format, similar
to a database. These files store everything from spell information and item stats
to world map data and creature definitions. This guide covers extracting, parsing,
and working with DBC files using `warcraft-rs`.

## Prerequisites

Before working with DBC files, ensure you have:

- Understanding of database concepts (tables, rows, columns)
- Basic knowledge of binary file formats
- `warcraft-rs` installed with the `dbc` feature enabled
- Access to WoW client files (MPQ archives)
- Familiarity with WoW data structures

## Understanding DBC Files

### DBC Structure

DBC files have a consistent structure:

- **Header**: File signature, record count, field count, record size
- **Records**: Fixed-size rows of data
- **String Block**: Variable-length strings referenced by offsets

### Common DBC Files

- `Spell.dbc`: Spell definitions and properties
- `Item.dbc`: Item templates and stats
- `Map.dbc`: World map information
- `AreaTable.dbc`: Zone and area definitions
- `ChrRaces.dbc`: Playable race data
- `ChrClasses.dbc`: Class definitions
- `CreatureDisplayInfo.dbc`: Creature model information
- `Achievement.dbc`: Achievement data

## Step-by-Step Instructions

### 1. Extracting DBC Files from MPQ

```rust
use wow_mpq::Archive;
use std::path::Path;
use std::fs;
use std::io::Write;

fn extract_dbc_files(mpq_path: &str, output_dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut archive = Archive::open(mpq_path)?;
    let mut extracted_files = Vec::new();

    // Create output directory
    fs::create_dir_all(output_dir)?;

    // List all files in the archive
    let entries = archive.list_all()?;
    for entry in entries {
        if entry.name.ends_with(".dbc") && entry.name.contains("DBFilesClient") {
            match archive.read_file(&entry.name) {
                Ok(data) => {
                    let filename = Path::new(&entry.name).file_name().unwrap().to_str().unwrap();
                    let output_path = Path::new(output_dir).join(filename);

                    let mut file = fs::File::create(&output_path)?;
                    file.write_all(&data)?;
                    extracted_files.push(filename.to_string());
                    println!("Extracted: {}", filename);
                }
                Err(e) => eprintln!("Failed to extract {}: {}", entry.name, e),
            }
        }
    }

    Ok(extracted_files)
}
```

### 2. Parsing DBC Files

Note: The DBC parsing implementation is still under development. For now, you can extract the raw DBC files and use external tools or implement basic parsing:

```rust
// Basic DBC header structure (for reference)
#[repr(C, packed)]
struct DbcHeader {
    signature: [u8; 4],    // 'WDBC'
    record_count: u32,     // Number of records
    field_count: u32,      // Number of fields per record
    record_size: u32,      // Size of each record in bytes
    string_block_size: u32, // Size of string block
}

fn parse_dbc_header(data: &[u8]) -> Option<DbcHeader> {
    if data.len() < 20 || &data[0..4] != b"WDBC" {
        return None;
    }

    // Parse header manually for now
    let record_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let field_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let record_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let string_block_size = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

    Some(DbcHeader {
        signature: [data[0], data[1], data[2], data[3]],
        record_count,
        field_count,
        record_size,
        string_block_size,
    })
}

fn analyze_dbc_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(file_path)?;

    if let Some(header) = parse_dbc_header(&data) {
        println!("DBC File: {}", file_path);
        println!("Records: {}", header.record_count);
        println!("Fields: {}", header.field_count);
        println!("Record Size: {} bytes", header.record_size);
        println!("String Block Size: {} bytes", header.string_block_size);

        let records_start = 20; // After header
        let records_end = records_start + (header.record_count * header.record_size) as usize;
        let strings_start = records_end;

        println!("Total file size: {} bytes", data.len());
        println!("Records section: {} - {} bytes", records_start, records_end);
        println!("String block: {} - {} bytes", strings_start, data.len());
    } else {
        println!("Invalid DBC file: {}", file_path);
    }

    Ok(())
}
### 3. Working with Extracted DBC Files

Once you have extracted DBC files, you can work with them using external tools or implement custom parsing logic:

```rust
use std::fs;

// Example: Basic DBC analysis
fn analyze_extracted_dbc_files(dbc_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dbc_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "dbc") {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                println!("\n=== {} ===", filename);
                analyze_dbc_file(path.to_str().unwrap())?;
            }
        }
    }

    Ok(())
}

// Example: Spell.dbc structure
#[derive(Debug, Clone)]
pub struct SpellRecord {
    pub id: u32,
    pub category: u32,
    pub dispel_type: u32,
    pub mechanic: u32,
    pub attributes: [u32; 8],
    pub stances: u32,
    pub stances_not: u32,
    pub targets: u32,
    pub target_creature_type: u32,
    pub requires_spell_focus: u32,
    pub facing_caster_flags: u32,
    pub caster_aura_state: u32,
    pub target_aura_state: u32,
    pub casting_time_index: u32,
    pub recovery_time: u32,
    pub category_recovery_time: u32,
    pub interrupt_flags: u32,
    pub aura_interrupt_flags: u32,
    pub channel_interrupt_flags: u32,
    pub proc_flags: u32,
    pub proc_chance: u32,
    pub proc_charges: u32,
    pub max_level: u32,
    pub base_level: u32,
    pub spell_level: u32,
    pub duration_index: u32,
    pub power_type: i32,
    pub mana_cost: u32,
    pub mana_cost_per_level: u32,
    pub mana_per_second: u32,
    pub range_index: u32,
    pub speed: f32,
    pub modal_next_spell: u32,
    pub stack_amount: u32,
    pub totem: [u32; 2],
    pub reagent: [i32; 8],
    pub reagent_count: [u32; 8],
    pub equipped_item_class: i32,
    pub equipped_item_sub_class_mask: i32,
    pub equipped_item_inventory_type_mask: i32,
    pub effect: [SpellEffect; 3],
    pub spell_visual: [u32; 2],
    pub spell_icon_id: u32,
    pub active_icon_id: u32,
    pub spell_priority: u32,
    pub spell_name: DbcString,
    pub spell_name_subtext: DbcString,
    pub description: DbcString,
    pub tooltip: DbcString,
}

#[derive(Debug, Clone)]
pub struct SpellEffect {
    pub effect: u32,
    pub die_sides: u32,
    pub real_points_per_level: f32,
    pub base_points: i32,
    pub mechanic: u32,
    pub implicit_target_a: u32,
    pub implicit_target_b: u32,
    pub radius_index: u32,
    pub aura: u32,
    pub amplitude: u32,
    pub multiple_value: f32,
    pub chain_target: u32,
    pub item_type: u32,
    pub misc_value: i32,
    pub misc_value_b: i32,
    pub trigger_spell: u32,
    pub points_per_combo_point: f32,
    pub class_mask: [u32; 3],
    pub spell_class_mask: [u32; 3],
}

impl DbcRecord for SpellRecord {
    fn read(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self> {
        let id = cursor.read_u32::<LittleEndian>()?;
        let category = cursor.read_u32::<LittleEndian>()?;
        let dispel_type = cursor.read_u32::<LittleEndian>()?;
        let mechanic = cursor.read_u32::<LittleEndian>()?;

        let mut attributes = [0u32; 8];
        for i in 0..8 {
            attributes[i] = cursor.read_u32::<LittleEndian>()?;
        }

        // ... read remaining fields ...

        let spell_name = DbcString::read(cursor, strings)?;
        let spell_name_subtext = DbcString::read(cursor, strings)?;
        let description = DbcString::read(cursor, strings)?;
        let tooltip = DbcString::read(cursor, strings)?;

        Ok(SpellRecord {
            id,
            category,
            dispel_type,
            mechanic,
            attributes,
            // ... all fields ...
            spell_name,
            spell_name_subtext,
            description,
            tooltip,
        })
    }
}
```

### 4. Creating a DBC Database

```rust
use std::collections::HashMap;
use wow_cdbc::*;

pub struct DbcDatabase {
    spells: HashMap<u32, SpellRecord>,
    items: HashMap<u32, ItemRecord>,
    maps: HashMap<u32, MapRecord>,
    areas: HashMap<u32, AreaTableRecord>,
    creatures: HashMap<u32, CreatureDisplayInfoRecord>,
}

impl DbcDatabase {
    pub fn new() -> Self {
        Self {
            spells: HashMap::new(),
            items: HashMap::new(),
            maps: HashMap::new(),
            areas: HashMap::new(),
            creatures: HashMap::new(),
        }
    }

    pub fn load_from_directory(&mut self, dbc_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::path::Path;

        // Load Spell.dbc
        let spell_path = Path::new(dbc_dir).join("Spell.dbc");
        if spell_path.exists() {
            let spells = read_dbc_records::<SpellRecord>(&spell_path.to_string_lossy())?;
            for spell in spells {
                self.spells.insert(spell.id, spell);
            }
            println!("Loaded {} spells", self.spells.len());
        }

        // Load Item.dbc
        let item_path = Path::new(dbc_dir).join("Item.dbc");
        if item_path.exists() {
            let items = read_dbc_records::<ItemRecord>(&item_path.to_string_lossy())?;
            for item in items {
                self.items.insert(item.id, item);
            }
            println!("Loaded {} items", self.items.len());
        }

        // Load other DBC files...

        Ok(())
    }

    pub fn get_spell(&self, id: u32) -> Option<&SpellRecord> {
        self.spells.get(&id)
    }

    pub fn find_spells_by_name(&self, name: &str) -> Vec<&SpellRecord> {
        self.spells
            .values()
            .filter(|spell| spell.spell_name.to_string().contains(name))
            .collect()
    }
}
```

### 5. Exporting DBC Data

```rust
use serde::{Serialize, Deserialize};
use csv::Writer;

// Export to JSON
fn export_dbc_to_json<T: Serialize>(records: &[T], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(records)?;
    std::fs::write(output_path, json)?;
    Ok(())
}

// Export to CSV
fn export_spells_to_csv(spells: &[SpellRecord], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = Writer::from_path(output_path)?;

    // Write header
    writer.write_record(&[
        "ID", "Name", "Description", "Category", "CastTime", "Range", "ManaCost", "Level"
    ])?;

    // Write records
    for spell in spells {
        writer.write_record(&[
            spell.id.to_string(),
            spell.spell_name.to_string(),
            spell.description.to_string(),
            spell.category.to_string(),
            spell.casting_time_index.to_string(),
            spell.range_index.to_string(),
            spell.mana_cost.to_string(),
            spell.spell_level.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

// Export to SQL
fn export_dbc_to_sql(table_name: &str, records: &[impl DbcRecord]) -> String {
    let mut sql = String::new();

    sql.push_str(&format!("CREATE TABLE {} (\n", table_name));
    // Define schema based on record type
    sql.push_str(");\n\n");

    // Insert statements
    for record in records {
        sql.push_str(&format!("INSERT INTO {} VALUES (", table_name));
        // Add values
        sql.push_str(");\n");
    }

    sql
}
```

### 6. Building a DBC Query Tool

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dbc-tool")]
#[command(about = "DBC file extraction and query tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract DBC files from MPQ
    Extract {
        #[arg(short, long)]
        mpq: String,
        #[arg(short, long)]
        output: String,
    },
    /// Query spell information
    Spell {
        #[arg(short, long)]
        id: Option<u32>,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Export DBC to various formats
    Export {
        #[arg(short, long)]
        dbc: String,
        #[arg(short, long)]
        format: String,
        #[arg(short, long)]
        output: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut db = DbcDatabase::new();

    match cli.command {
        Commands::Extract { mpq, output } => {
            extract_dbc_files(&mpq, &output)?;
        }
        Commands::Spell { id, name } => {
            db.load_from_directory("./dbc")?;

            if let Some(spell_id) = id {
                if let Some(spell) = db.get_spell(spell_id) {
                    print_spell_info(spell);
                }
            } else if let Some(spell_name) = name {
                let spells = db.find_spells_by_name(&spell_name);
                for spell in spells {
                    print_spell_info(spell);
                }
            }
        }
        Commands::Export { dbc, format, output } => {
            match format.as_str() {
                "json" => {
                    let records = read_dbc_records::<SpellRecord>(&dbc)?;
                    export_dbc_to_json(&records, &output)?;
                }
                "csv" => {
                    let records = read_dbc_records::<SpellRecord>(&dbc)?;
                    export_spells_to_csv(&records, &output)?;
                }
                _ => eprintln!("Unsupported format: {}", format),
            }
        }
    }

    Ok(())
}

fn print_spell_info(spell: &SpellRecord) {
    println!("Spell ID: {}", spell.id);
    println!("Name: {}", spell.spell_name);
    println!("Description: {}", spell.description);
    println!("Level: {}", spell.spell_level);
    println!("Mana Cost: {}", spell.mana_cost);
    println!("Cast Time: {}", spell.casting_time_index);
    println!("---");
}
```

## Code Examples

### Complete DBC Parser Library

```rust
use wow_cdbc::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DbcParser {
    cache: HashMap<String, Arc<DbcFile>>,
    string_cache: HashMap<String, String>,
}

impl DbcParser {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            string_cache: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str) -> Result<Arc<DbcFile>, Box<dyn std::error::Error>> {
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        let data = std::fs::read(path)?;
        let dbc = DbcFile::from_bytes(&data)?;
        let arc_dbc = Arc::new(dbc);

        self.cache.insert(path.to_string(), arc_dbc.clone());
        Ok(arc_dbc)
    }

    pub fn parse_generic<T>(&mut self, path: &str) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        T: DbcRecord + 'static,
    {
        let dbc = self.parse_file(path)?;
        let mut records = Vec::with_capacity(dbc.header.record_count as usize);

        let mut cursor = Cursor::new(&dbc.records);
        for _ in 0..dbc.header.record_count {
            let record = T::read(&mut cursor, &dbc.strings)?;
            records.push(record);
        }

        Ok(records)
    }

    pub fn get_string(&mut self, offset: u32, strings: &[u8]) -> String {
        if let Some(cached) = self.string_cache.get(&offset.to_string()) {
            return cached.clone();
        }

        let string = read_cstring_at_offset(strings, offset as usize);
        self.string_cache.insert(offset.to_string(), string.clone());
        string
    }
}

fn read_cstring_at_offset(data: &[u8], offset: usize) -> String {
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }

    String::from_utf8_lossy(&data[offset..end]).to_string()
}
```

### Localized DBC Support

```rust
use wow_cdbc::{DbcString, LocalizedString};

#[derive(Debug)]
pub struct LocalizedDbcString {
    pub en_us: String,
    pub ko_kr: String,
    pub fr_fr: String,
    pub de_de: String,
    pub en_cn: String,
    pub zh_cn: String,
    pub en_tw: String,
    pub zh_tw: String,
    pub es_es: String,
    pub es_mx: String,
    pub ru_ru: String,
    pub ja_jp: String,
    pub pt_br: String,
    pub it_it: String,
    pub unknown: String,
    pub flags: u32,
}

impl LocalizedDbcString {
    pub fn read(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self> {
        let mut locales = Vec::with_capacity(16);

        // Read 16 locale string offsets
        for _ in 0..16 {
            let offset = cursor.read_u32::<LittleEndian>()?;
            let string = if offset > 0 {
                read_cstring_at_offset(strings, offset as usize)
            } else {
                String::new()
            };
            locales.push(string);
        }

        let flags = cursor.read_u32::<LittleEndian>()?;

        Ok(LocalizedDbcString {
            en_us: locales[0].clone(),
            ko_kr: locales[1].clone(),
            fr_fr: locales[2].clone(),
            de_de: locales[3].clone(),
            en_cn: locales[4].clone(),
            zh_cn: locales[5].clone(),
            en_tw: locales[6].clone(),
            zh_tw: locales[7].clone(),
            es_es: locales[8].clone(),
            es_mx: locales[9].clone(),
            ru_ru: locales[10].clone(),
            ja_jp: locales[11].clone(),
            pt_br: locales[12].clone(),
            it_it: locales[13].clone(),
            unknown: locales[14].clone(),
            flags,
        })
    }

    pub fn get_locale(&self, locale: &str) -> &str {
        match locale {
            "enUS" => &self.en_us,
            "koKR" => &self.ko_kr,
            "frFR" => &self.fr_fr,
            "deDE" => &self.de_de,
            "enCN" => &self.en_cn,
            "zhCN" => &self.zh_cn,
            "enTW" => &self.en_tw,
            "zhTW" => &self.zh_tw,
            "esES" => &self.es_es,
            "esMX" => &self.es_mx,
            "ruRU" => &self.ru_ru,
            "jaJP" => &self.ja_jp,
            "ptBR" => &self.pt_br,
            "itIT" => &self.it_it,
            _ => &self.en_us, // Default to English
        }
    }
}
```

## Best Practices

### 1. Lazy Loading

```rust
use std::cell::RefCell;
use std::rc::Rc;

pub struct LazyDbcLoader {
    dbc_dir: String,
    loaded: RefCell<HashMap<String, Rc<Box<dyn Any>>>>,
}

impl LazyDbcLoader {
    pub fn new(dbc_dir: String) -> Self {
        Self {
            dbc_dir,
            loaded: RefCell::new(HashMap::new()),
        }
    }

    pub fn get<T: DbcRecord + 'static>(&self, filename: &str) -> Result<Rc<Vec<T>>, Box<dyn std::error::Error>> {
        let mut loaded = self.loaded.borrow_mut();

        if let Some(data) = loaded.get(filename) {
            if let Ok(records) = data.clone().downcast::<Vec<T>>() {
                return Ok(records);
            }
        }

        // Load and parse
        let path = Path::new(&self.dbc_dir).join(filename);
        let records = read_dbc_records::<T>(&path.to_string_lossy())?;
        let rc_records = Rc::new(records);

        loaded.insert(filename.to_string(), Rc::new(Box::new(rc_records.clone()) as Box<dyn Any>));

        Ok(rc_records)
    }
}
```

### 2. Indexing for Fast Lookups

```rust
pub struct IndexedDbc<T> {
    records: Vec<T>,
    by_id: HashMap<u32, usize>,
    by_name: HashMap<String, Vec<usize>>,
}

impl<T: DbcRecord + HasId + HasName> IndexedDbc<T> {
    pub fn new(records: Vec<T>) -> Self {
        let mut by_id = HashMap::new();
        let mut by_name = HashMap::new();

        for (idx, record) in records.iter().enumerate() {
            by_id.insert(record.id(), idx);

            by_name
                .entry(record.name().to_lowercase())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        Self {
            records,
            by_id,
            by_name,
        }
    }

    pub fn get_by_id(&self, id: u32) -> Option<&T> {
        self.by_id.get(&id).map(|&idx| &self.records[idx])
    }

    pub fn search_by_name(&self, query: &str) -> Vec<&T> {
        let query_lower = query.to_lowercase();

        self.by_name
            .iter()
            .filter(|(name, _)| name.contains(&query_lower))
            .flat_map(|(_, indices)| indices.iter().map(|&idx| &self.records[idx]))
            .collect()
    }
}
```

### 3. Version Compatibility

```rust
pub enum DbcVersion {
    Classic,
    TBC,
    WotLK,
    Cataclysm,
    MoP,
}

pub trait VersionedDbcRecord: Sized {
    fn read_classic(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self>;
    fn read_tbc(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self>;
    fn read_wotlk(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self>;
    fn read_cata(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self>;
    fn read_mop(cursor: &mut Cursor<&[u8]>, strings: &[u8]) -> Result<Self>;

    fn read_for_version(
        cursor: &mut Cursor<&[u8]>,
        strings: &[u8],
        version: DbcVersion
    ) -> Result<Self> {
        match version {
            DbcVersion::Classic => Self::read_classic(cursor, strings),
            DbcVersion::TBC => Self::read_tbc(cursor, strings),
            DbcVersion::WotLK => Self::read_wotlk(cursor, strings),
            DbcVersion::Cataclysm => Self::read_cata(cursor, strings),
            DbcVersion::MoP => Self::read_mop(cursor, strings),
        }
    }
}
```

## Common Issues and Solutions

### Issue: String Encoding

**Problem**: Non-ASCII characters appear corrupted.

**Solution**:

```rust
use encoding_rs::WINDOWS_1252;

fn decode_dbc_string(bytes: &[u8]) -> String {
    // DBC files often use Windows-1252 encoding
    let (decoded, _, had_errors) = WINDOWS_1252.decode(bytes);

    if had_errors {
        // Fall back to UTF-8 lossy
        String::from_utf8_lossy(bytes).to_string()
    } else {
        decoded.to_string()
    }
}
```

### Issue: Missing DBC Files

**Problem**: Some DBC files are not in the MPQ archives.

**Solution**:

```rust
fn find_dbc_in_multiple_mpqs(filename: &str, mpq_paths: &[&str]) -> Option<Vec<u8>> {
    for mpq_path in mpq_paths {
        if let Ok(mut archive) = Archive::open(mpq_path) {
            if let Ok(data) = archive.read_file(&format!("DBFilesClient/{}", filename)) {
                return Some(data);
            }
        }
    }
    None
}

// Search in patch MPQs first (higher priority)
let mpq_search_order = [
    "Data/patch-3.MPQ",
    "Data/patch-2.MPQ",
    "Data/patch.MPQ",
    "Data/common.MPQ",
    "Data/expansion.MPQ",
];
```

### Issue: Record Size Mismatch

**Problem**: DBC parser fails due to unexpected record size.

**Solution**:

```rust
fn validate_dbc_header(header: &DbcHeader, expected_size: usize) -> Result<(), DbcError> {
    if header.record_size != expected_size as u32 {
        // Some DBCs have padding or version differences
        eprintln!(
            "Warning: Expected record size {}, got {}. Attempting to parse anyway.",
            expected_size, header.record_size
        );

        // Check if it's a known variation
        match header.record_size {
            size if size > expected_size as u32 => {
                // Newer version with additional fields
                Ok(())
            }
            size if size < expected_size as u32 => {
                // Older version, may need special handling
                Err(DbcError::IncompatibleVersion)
            }
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}
```

## Performance Tips

### 1. Parallel DBC Loading

```rust
use rayon::prelude::*;
use std::sync::Arc;

pub fn load_all_dbcs_parallel(dbc_dir: &str) -> HashMap<String, Arc<DbcFile>> {
    let dbc_files: Vec<_> = std::fs::read_dir(dbc_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "dbc" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    dbc_files
        .par_iter()
        .filter_map(|path| {
            let filename = path.file_name()?.to_str()?.to_string();
            let data = std::fs::read(path).ok()?;
            let dbc = DbcFile::from_bytes(&data).ok()?;
            Some((filename, Arc::new(dbc)))
        })
        .collect()
}
```

### 2. Memory-Mapped DBC Files

```rust
use memmap2::MmapOptions;
use std::fs::File;

pub struct MappedDbc {
    _file: File,
    mmap: memmap2::Mmap,
    header: DbcHeader,
}

impl MappedDbc {
    pub fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };

        // Read header
        let header = DbcHeader::read(&mmap[..20])?;

        Ok(Self {
            _file: file,
            mmap,
            header,
        })
    }

    pub fn get_record(&self, index: u32) -> Option<&[u8]> {
        if index >= self.header.record_count {
            return None;
        }

        let offset = 20 + (index * self.header.record_size) as usize;
        let end = offset + self.header.record_size as usize;

        Some(&self.mmap[offset..end])
    }
}
```

### 3. DBC Compression

```rust
use flate2::{Compression, write::GzEncoder, read::GzDecoder};
use std::io::prelude::*;

pub fn compress_dbc(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_data = std::fs::read(input_path)?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&input_data)?;
    let compressed = encoder.finish()?;

    std::fs::write(output_path, compressed)?;

    println!("Compressed {} bytes to {} bytes", input_data.len(), compressed.len());
    Ok(())
}

pub fn decompress_dbc(input_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let compressed = std::fs::read(input_path)?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
```

## Related Guides

- [📦 Working with MPQ Archives](./mpq-archives.md) - Extract DBC files from game archives
- [🎭 Loading M2 Models](./m2-models.md) - Use DBC data to load models
- [🌍 Rendering ADT Terrain](./adt-rendering.md) - Use map DBC data for terrain

## References

- [DBC Format Documentation](https://wowdev.wiki/DBC) - Complete DBC format specification
- [DBC File List](https://wowdev.wiki/Category:DBC) - List of all DBC files and their purposes
- [Trinity Core DBC Structures](https://github.com/TrinityCore/TrinityCore/tree/master/src/server/game/DataStores) - Reference DBC structures
- [WoW Dev Tools](https://github.com/WoWDevTools) - Tools for working with WoW files
