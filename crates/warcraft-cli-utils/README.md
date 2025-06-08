# warcraft-cli-utils 🛠️

Shared utilities for warcraft-rs command-line tools.

## Features

- 📊 Progress bars and spinners
- 📋 Table formatting
- 📏 Human-readable file sizes
- 🎯 Pattern matching
- ⏰ Timestamp formatting
- 🗜️ Compression ratio calculations

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
warcraft-cli-utils = { path = "../warcraft-cli-utils" }
```

## Examples

### Progress Bars

```rust
use warcraft_cli_utils::create_progress_bar;

let pb = create_progress_bar(100, "Processing files");
for i in 0..100 {
    // Do work...
    pb.inc(1);
}
pb.finish_with_message("Complete");
```

### File Size Formatting

```rust
use warcraft_cli_utils::format_bytes;

println!("{}", format_bytes(1_048_576)); // "1.05 MB"
```

### Pattern Matching

```rust
use warcraft_cli_utils::matches_pattern;

assert!(matches_pattern("model.m2", "*.m2"));
assert!(matches_pattern("Interface/Icons/icon.blp", "*Icons*"));
```

### Table Creation

```rust
use warcraft_cli_utils::{create_table, add_table_row};

let mut table = create_table(vec!["Name", "Size", "Type"]);
add_table_row(&mut table, vec![
    "model.m2".to_string(),
    "1.2 MB".to_string(),
    "Model".to_string(),
]);
table.printstd();
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
