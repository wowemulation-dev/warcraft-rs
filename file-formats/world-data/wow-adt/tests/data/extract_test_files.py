#!/usr/bin/env python3
"""Extract test ADT files from WoW MPQ archives for compliance testing."""

import subprocess
import sys
from pathlib import Path

# Test files to extract (zone, x, y coordinates)
VANILLA_FILES = [
    ("Azeroth", 30, 48),  # Elwynn Forest - varied terrain
    ("Kalimdor", 43, 51),  # Durotar - simple terrain
    ("Azeroth", 36, 50),  # Redridge Mountains - complex heightmap
    ("Kalimdor", 47, 8),   # Darkshore - water/coastline
    ("Azeroth", 28, 47),   # Stormwind City - WMO heavy
]

def extract_adt(mpq_path: str, map_name: str, x: int, y: int, output_dir: Path):
    """Extract a single ADT file from MPQ archive."""
    # Construct the internal MPQ path
    adt_name = f"{map_name}_{x}_{y}.adt"
    internal_path = f"World\\Maps\\{map_name}\\{adt_name}"

    output_file = output_dir / adt_name

    if output_file.exists():
        print(f"✓ {adt_name} already exists")
        return True

    print(f"Extracting {adt_name}...", end=" ")

    # Use subprocess to call cargo run for MPQ extraction
    # This mimics the warcraft-rs mpq extract command
    try:
        result = subprocess.run(
            [
                "cargo", "run", "--release", "-p", "warcraft-rs", "--",
                "mpq", "extract", mpq_path, internal_path
            ],
            capture_output=True,
            text=True,
            check=True,
            cwd=Path(__file__).parent.parent.parent.parent.parent.parent
        )

        # The command extracts to current directory, move to output_dir
        extracted_file = Path(adt_name)
        if extracted_file.exists():
            extracted_file.rename(output_file)
            print("✓")
            return True
        else:
            print("✗ File not found after extraction")
            return False

    except subprocess.CalledProcessError as e:
        print(f"✗ Error: {e.stderr}")
        return False

def main():
    # Determine base directory
    base_dir = Path(__file__).parent
    vanilla_dir = base_dir / "vanilla"
    vanilla_dir.mkdir(parents=True, exist_ok=True)

    # MPQ archive path
    mpq_path = "/home/danielsreichenbach/Downloads/wow/1.12.1/Data/terrain.MPQ"

    if not Path(mpq_path).exists():
        print(f"Error: MPQ archive not found: {mpq_path}")
        sys.exit(1)

    print("Extracting Vanilla 1.12.1 test ADT files...")
    print(f"Source: {mpq_path}")
    print(f"Target: {vanilla_dir}")
    print()

    success_count = 0
    for map_name, x, y in VANILLA_FILES:
        if extract_adt(mpq_path, map_name, x, y, vanilla_dir):
            success_count += 1

    print()
    print(f"Extracted {success_count}/{len(VANILLA_FILES)} files successfully")

    if success_count == len(VANILLA_FILES):
        print("✓ All test files ready for compliance testing")
        return 0
    else:
        print("✗ Some files failed to extract")
        return 1

if __name__ == "__main__":
    sys.exit(main())
