# WoW Patch Chain Summary

This document provides an overview of how patch chaining works in
each World of Warcraft version from 1.12.1 through 5.4.8, based on analysis of
actual game archives.

## Overview

WoW's MPQ patch chain system allows the game to override base content with patches,
following a strict priority order. Higher priority archives override files from
lower priority archives, enabling Blizzard to update game content without redistributing
entire archives.

## Version-by-Version Analysis

### WoW 1.12.1 (Vanilla)

**Archive Structure:**

- 7 total archives (simple structure)
- Base: `dbc.MPQ`, `interface.MPQ`, `model.MPQ`, `sound.MPQ`, `texture.MPQ`
- Patches: `patch.MPQ`, `patch-2.MPQ`

**Loading Order:**

```
Priority 0-4:    Base archives
Priority 1000+:  Patch archives (override everything)
```

**Key Characteristics:**

- Simple, straightforward patching
- No locale-specific archives
- Spell.dbc evolution: 14,502 → 22,357 records (+7,855 spells)
- File size growth: 9.1MB → 15.8MB

**Example Override:**

```
DBFilesClient\Spell.dbc:
  dbc.MPQ (base) → patch.MPQ → patch-2.MPQ (final)
```

### WoW 2.4.3 (The Burning Crusade)

**Archive Structure:**

- 8+ archives (introduces locale system)
- Base: `common.MPQ`, `expansion.MPQ`
- Locale: `locale-{LANG}.MPQ`, `expansion-locale-{LANG}.MPQ`
- Patches: `patch.MPQ`, `patch-2.MPQ`, `patch-{LANG}.MPQ`, `patch-{LANG}-2.MPQ`

**Loading Order:**

```
Priority 0-1:      Base archives
Priority 100-101:  Locale base archives
Priority 1000-1001: General patches
Priority 2000-2001: Locale patches (highest priority)
```

**Key Characteristics:**

- Introduces locale-specific override system
- MPQ v2 format
- 185 unique DBCs with 134 having locale overrides
- Spell.dbc: 22.2MB → 25.7MB through patches
- 28,315 spells (864 bytes per record)

**Locale Override Example:**

```
DBFilesClient\Spell.dbc:
  locale-enUS.MPQ → patch-enUS.MPQ → patch-enUS-2.MPQ (final)
```

### WoW 3.3.5a (Wrath of the Lich King)

**Archive Structure:**

- 13 archives (most organized structure)
- Base: `common.MPQ`, `common-2.MPQ`, `expansion.MPQ`, `lichking.MPQ`
- Locale: `locale-{LANG}.MPQ`, `expansion-locale-{LANG}.MPQ`, `lichking-locale-{LANG}.MPQ`
- Patches: `patch.MPQ`, `patch-2.MPQ`, `patch-3.MPQ` (+ locale versions)

**Loading Order (TrinityCore Definitive):**

```
Priority 0-3:      Base archives (in order)
Priority 100-102:  Locale base archives
Priority 1000-1002: General patches
Priority 2000-2002: Locale patches (highest)
```

**Key Characteristics:**

- Most structured patch hierarchy
- Clear content separation (base, TBC, WotLK)
- WotLK features: Achievements, Vehicles, Glyphs
- Largest archives: common.MPQ (2.7GB), lichking.MPQ (2.4GB)

**New Systems:**

- Achievement system (`Achievement.dbc`)
- Vehicle mechanics (`Vehicle.dbc`, `VehicleSeat.dbc`)
- Glyph system (`GlyphProperties.dbc`)
- Dungeon Finder (`BattlemasterList.dbc`)

### WoW 4.3.4 (Cataclysm)

**Archive Structure:**

- Variable (10-50+ archives depending on patch level)
- Content-based organization: `art.MPQ`, `sound.MPQ`, `world.MPQ`, `model.MPQ`
- Expansions: `expansion1.MPQ` through `expansion3.MPQ`
- Patches: `base-{1-20}.MPQ`, `wow-update-{13156-16000+}.MPQ`

**Loading Order:**

```
Priority 0-99:      Base archives (by content type)
Priority 100-199:   Locale base archives
Priority 1000-1999: Base patches
Priority 2000-2999: Locale patches
Priority 3000+:     wow-update archives
Priority 4000+:     Locale wow-update archives
```

**Key Characteristics:**

- Switched to MPQ v4 format
- Introduced DB2 format alongside DBC
- Content reorganization (no more single dbc.MPQ)
- Complex patching with numbered base patches
- Introduced `wow-update-#####.MPQ` system

**New Features:**

- DB2 format (`Item.db2`, `Item-sparse.db2`)
- Guild system revamp
- Flying in old world
- Phasing technology

### WoW 5.4.8 (Mists of Pandaria)

**Archive Structure:**

- Most complex (potentially 100+ archives)
- Base: `art.MPQ`, `expansion1-4.MPQ`, `misc.MPQ`, `model.MPQ`, `sound.MPQ`, `texture.MPQ`, `world.MPQ`, `world2.MPQ`
- Patches: `base-{1-50}.MPQ`, `wow-update-{13156-18500}.MPQ`
- Full locale structure for each component

**Loading Order:**

```
Priority 0-99:      Base archives
Priority 100-199:   Locale base archives
Priority 1000-2999: Base patches & locale patches
Priority 3000-3999: wow-update archives
Priority 4000+:     Locale wow-update archives
```

**Key Characteristics:**

- Peak MPQ complexity before CASC
- Extensive use of wow-update system
- 11 character classes (added Monk)
- Preparing for CASC transition (WoW 6.0)

**MoP Systems:**

- Pet Battles (`BattlePetSpecies.db2`, `BattlePetAbility.db2`)
- Scenarios (`Scenario.dbc`, `ScenarioStep.dbc`)
- Item upgrades (`ItemUpgrade.dbc`)
- Account-wide collections

## Priority System Rules

1. **Higher priority always wins**: Files in higher priority archives override lower priority
2. **Locale overrides general**: Locale-specific archives override their general counterparts
3. **Patches override base**: All patches override all base content
4. **Loading order matters**: Archives must be loaded in the correct sequence

## Best Practices

1. **Always follow the official loading order** (see TrinityCore for 3.3.5a reference)
2. **Test with real game data** to verify patch chains work correctly
3. **Handle missing archives gracefully** - not all installations have all patches
4. **Cache file lookups** for performance in large patch chains
5. **Verify file resolution** matches the game client behavior

## Evolution Summary

- **Vanilla**: Simple base + patch structure
- **TBC**: Added locale system and override hierarchy
- **WotLK**: Perfected the structured approach with clear priorities
- **Cataclysm**: Reorganized by content type, added DB2 format
- **MoP**: Peak complexity, preparing for CASC storage system
- **WoD (6.0)+**: Switched to CASC, abandoning MPQ

The progression shows Blizzard's evolution from simple file patching to increasingly sophisticated content delivery, ultimately leading to the complete replacement of MPQ with the CASC system in Warlords of Draenor.
