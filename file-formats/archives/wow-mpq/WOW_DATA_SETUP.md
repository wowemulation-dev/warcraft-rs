# WoW Data Setup for Examples

This document explains how to set up World of Warcraft game data for running examples that require actual MPQ files.

## Quick Setup

Set environment variables pointing to your WoW Data directories:

```bash
# For Vanilla/Classic (1.12.1)
export WOW_VANILLA_DATA="/path/to/wow/1.12.1/Data"

# For The Burning Crusade (2.4.3)
export WOW_TBC_DATA="/path/to/wow/2.4.3/Data"

# For Wrath of the Lich King (3.3.5a)
export WOW_WOTLK_DATA="/path/to/wow/3.3.5a/Data"

# For Cataclysm (4.3.4)
export WOW_CATA_DATA="/path/to/wow/4.3.4/Data"

# For Mists of Pandaria (5.4.8)
export WOW_MOP_DATA="/path/to/wow/5.4.8/Data"
```

## Environment Variables

The library checks for these environment variables in order of preference:

| WoW Version | Environment Variable | Example Path |
|-------------|---------------------|--------------|
| 1.12.1 (Vanilla/Classic) | `WOW_VANILLA_DATA` | `/home/user/wow/1.12.1/Data` |
| 2.4.3 (TBC) | `WOW_TBC_DATA` | `/home/user/wow/2.4.3/Data` |
| 3.3.5a (WotLK) | `WOW_WOTLK_DATA` | `/home/user/wow/3.3.5a/Data` |
| 4.3.4 (Cataclysm) | `WOW_CATA_DATA` | `/home/user/wow/4.3.4/Data` |
| 5.4.8 (MoP) | `WOW_MOP_DATA` | `/home/user/wow/5.4.8/Data` |

## Common Installation Paths

If environment variables aren't set, the library will search common installation paths:

### Linux/Unix

```
~/wow/[version]/Data
~/Downloads/wow/[version]/Data
/opt/wow/[version]/Data
/usr/local/games/wow/[version]/Data
```

### Windows

```
C:\Program Files\World of Warcraft\[version]\Data
C:\Program Files (x86)\World of Warcraft\[version]\Data
C:\Games\World of Warcraft\[version]\Data
```

### macOS

```
/Applications/World of Warcraft/[version]/Data
~/Applications/World of Warcraft/[version]/Data
```

## Alternative Directory Names

The library also checks for common alternative directory names:

- **Vanilla**: `vanilla`, `classic`, or just `Data` in the root
- **TBC**: `tbc`, `burning-crusade`
- **WotLK**: `wotlk`, `wrath`
- **Cataclysm**: `cata`, `cataclysm`
- **MoP**: `mop`, `pandaria`

## Examples Usage

### Using Environment Variables

```bash
# Set up the environment
export WOW_VANILLA_DATA="/home/user/Games/wow-vanilla/Data"

# Run examples
cargo run --example patch_chain_demo
cargo run --example verify_wow_files
```

### Using Command Line Arguments

Most examples also accept file paths as command line arguments:

```bash
# Run with a specific MPQ file
cargo run --example verify_wow_files -- "/path/to/patch.MPQ"

# Run patch chain demo with a specific data directory
cargo run --example patch_chain_demo -- "/path/to/wow/Data"
```

### Directory Structure Example

Your WoW data should be organized like this:

```
/home/user/wow/
├── 1.12.1/
│   └── Data/
│       ├── patch.MPQ
│       ├── patch-2.MPQ
│       ├── dbc.MPQ
│       ├── interface.MPQ
│       ├── misc.MPQ
│       ├── model.MPQ
│       └── ...
├── 2.4.3/
│   └── Data/
│       ├── patch.mpq
│       ├── patch-2.mpq
│       └── ...
└── 3.3.5a/
    └── Data/
        ├── patch.MPQ
        ├── patch-2.MPQ
        ├── patch-3.MPQ
        └── ...
```

## Required Files

Examples will look for these MPQ files (in order of preference):

1. `patch.MPQ` or `patch.mpq` - Main patch archive
2. `misc.MPQ` - Miscellaneous data
3. `dbc.MPQ` - Database files
4. `interface.MPQ` - Interface files
5. `model.MPQ` - 3D models
6. `sound.MPQ` - Audio files
7. `texture.MPQ` - Textures
8. `wmo.MPQ` - World map objects

## Troubleshooting

### No WoW Data Found

If you see this error:

```
No WoW game data found!
```

1. Check that your environment variables are set correctly
2. Verify the paths exist and contain MPQ files
3. Try using an absolute path as a command line argument

### Permission Errors

Make sure the wow-mpq process has read permissions for the Data directory and MPQ files:

```bash
chmod -R +r /path/to/wow/Data
```

### Case Sensitivity

On Linux/Unix systems, file names are case-sensitive. MPQ files might be named:

- `patch.MPQ` (uppercase)
- `patch.mpq` (lowercase)

The library checks for both variants.

## Example: Setting Up Vanilla WoW

1. Extract or copy your WoW 1.12.1 client to `/home/user/wow/1.12.1/`
2. Set the environment variable:

   ```bash
   echo 'export WOW_VANILLA_DATA="/home/user/wow/1.12.1/Data"' >> ~/.bashrc
   source ~/.bashrc
   ```

3. Verify the setup:

   ```bash
   ls "$WOW_VANILLA_DATA"  # Should show MPQ files
   ```

4. Run examples:

   ```bash
   cargo run --example patch_chain_demo
   ```

This system makes examples portable across different development environments while maintaining the ability to override paths when needed.
