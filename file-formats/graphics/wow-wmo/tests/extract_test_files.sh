#!/bin/bash
# Script to extract test WMO files from MPQ archives
# This will be used once MPQ extraction is available

# Test files to extract for Classic (1.12.1)
CLASSIC_FILES=(
    "World/wmo/Azeroth/Buildings/Stormwind/Stormwind.wmo"
    "World/wmo/Azeroth/Buildings/Stormwind/Stormwind_000.wmo"
    "World/wmo/Azeroth/Buildings/Cathedral/Cathedral.wmo"
    "World/wmo/Azeroth/Buildings/Cathedral/Cathedral_000.wmo"
)

# Test files to extract for WotLK (3.3.5a)
WOTLK_FILES=(
    "World/wmo/Northrend/Dalaran/ND_Dalaran.wmo"
    "World/wmo/Northrend/Dalaran/ND_Dalaran_000.wmo"
)

# Test files to extract for Cataclysm (4.3.4)
CATA_FILES=(
    # Transport WMOs with MCVP chunks
    "World/wmo/Transport/Transport_Zeppelin_BG.wmo"
    "World/wmo/Transport/Transport_Zeppelin_BG_000.wmo"
)

echo "This script will extract WMO test files once MPQ extraction is available"
echo "Required MPQ archives:"
echo "  - ~/Downloads/wow/1.12.1/Data/*.MPQ"
echo "  - ~/Downloads/wow/3.3.5a/Data/*.MPQ"
echo "  - ~/Downloads/wow/4.3.4/4.3.4/Data/*.MPQ"

# TODO: Use wow-mpq crate or external tool to extract files