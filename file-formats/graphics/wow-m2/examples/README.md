# WoW M2 Examples

This directory contains practical examples demonstrating how to use the wow-m2 library.

## Available Examples

### 1. load_model.rs - Loading and Inspecting Models

Shows how to load an M2 model file and print detailed information about its contents.

```bash
cargo run --example load_model -- path/to/model.m2
```

### 2. convert_model.rs - Version Conversion

Demonstrates converting M2 models between different WoW client versions.

```bash
cargo run --example convert_model -- input.m2 output.m2 mop
```

Supported target versions:

- `classic`, `vanilla`, `tbc`, `wrath`, or `wotlk` - Classic WoW (includes Vanilla, TBC, and WotLK)
- `cata` or `cataclysm` - Cataclysm
- `mop` - Mists of Pandaria
- `wod` - Warlords of Draenor
- `legion` - Legion
- `bfa` - Battle for Azeroth
- `sl` or `shadowlands` - Shadowlands
- `df`, `dragonflight`, or `modern` - Dragon Flight and beyond

### 3. validate_model.rs - Model Validation

Validates M2 model files for correctness and integrity issues.

```bash
cargo run --example validate_model -- path/to/model.m2
```

### 4. work_with_skins.rs - Skin File Processing

Shows how to load and analyze skin files (.skin) that contain mesh and LOD data.

```bash
cargo run --example work_with_skins -- path/to/model00.skin
```

## Running Examples with Test Data

The repository includes test data files that you can use with these examples:

```bash
# Load a Classic model
cargo run --example load_model -- test-data/1.12.1/Creature/MadScientist/MadScientist.m2

# Convert a TBC model to MoP format
cargo run --example convert_model -- test-data/2.4.3/Creature/Rexxar/Rexxar.M2 converted_rexxar.m2 mop

# Validate a Wrath model
cargo run --example validate_model -- test-data/3.3.5a/Creature/FelBeast/FelBeast.m2

# Analyze a skin file
cargo run --example work_with_skins -- test-data/3.3.5a/Creature/tigon/TigonFemale00.skin
```

## Tips

- Use the `--release` flag for better performance when processing large models:

  ```bash
  cargo run --release --example load_model -- large_model.m2
  ```

- The examples include error handling and will provide helpful messages if something goes wrong.

- When converting models, be aware that downgrading to older versions may result in data loss, as newer formats support features that older ones don't.
