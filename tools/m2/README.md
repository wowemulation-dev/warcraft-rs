# Enhanced M2 Parser - Python Implementation with Rust Parity

This directory contains an enhanced Python M2 parser that achieves **complete parity** with the Rust implementation by implementing all critical missing features.

## 🚀 New Features (Achieving Rust Parity)

### ✅ 1. Compressed Quaternion Support (`quaternion.py`)
- **M2CompQuat class** with proper X-component negation matching pywowlib reference
- Conversion to/from float quaternions with proper scaling (±32767 range)
- Euler angle conversion for debugging and visualization
- Roundtrip conversion preservation with quantization handling

```python
from quaternion import M2CompQuat

# Parse compressed quaternion from binary data
quat = M2CompQuat.parse(data, offset)

# Convert to float quaternion  
x, y, z, w = quat.to_float_quaternion()

# Convert to Euler angles for debugging
pitch, yaw, roll = quat.to_euler_degrees()
```

### ✅ 2. Coordinate System Transformations (`coordinate_systems.py`)
- Transform WoW coordinates to **Blender**, **Unity**, and **Unreal Engine**
- Position and quaternion transformation support
- Batch transformation for efficient processing
- Matrix-based transformations for advanced use cases

```python
from coordinate_systems import CoordinateSystem, CoordinateTransformer, Vec3

# Transform single position
wow_pos = Vec3(100.0, 200.0, 50.0)  # North, West, Up
blender_pos = transform_position(wow_pos, CoordinateSystem.BLENDER)

# Batch transformation
transformer = CoordinateTransformer(CoordinateSystem.BLENDER)
transformed_positions = transformer.transform_positions(positions)
```

**Supported Coordinate Systems:**
- **WoW**: Right-handed, X=North, Y=West, Z=Up  
- **Blender**: Right-handed, X=Right, Y=Forward, Z=Up
- **Unity**: Left-handed, X=Right, Y=Up, Z=Forward  
- **Unreal**: Left-handed, X=Forward, Y=Right, Z=Up

### ✅ 3. Enhanced Validation Modes (`validation.py`)
Three validation modes matching the Rust implementation exactly:

- **Strict**: Fix all zero weights aggressively (legacy behavior)
- **Permissive**: Fix only clear corruption, preserve static geometry (recommended)
- **None**: Preserve all original data (use with caution)

```python
from validation import create_validator, ValidationMode

validator = create_validator("permissive")
validated_vertex = validator.validate_vertex(vertex_data, bone_count=34)

# Get validation statistics
stats = validator.get_validation_stats(vertices)
print(f"Fixed {stats['indices_fixed']} vertices with invalid bone indices")
```

**Validation Fixes:**
- ✅ Bone indices exceeding model bone count (clamps to valid range)
- ✅ Zero bone weights causing unmovable vertices  
- ✅ NaN values in positions/normals/UVs
- ✅ Invalid normal vector lengths

### ✅ 4. Complete Animation Track Parsing (`enhanced_parser.py`)
- Parse **rotation tracks** containing M2CompQuat values
- Parse **translation** and **scale** tracks  
- Support for both pre-WotLK (v256-263) and WotLK+ (v264+) formats
- Coordinate transformation of animation data

```python
# Access parsed animation data
for bone in parser.enhanced_bones:
    if bone.rotations:
        print(f"Bone {bone.key_bone_id}: {len(bone.rotations)} rotation keyframes")
        first_rotation = bone.rotations[0]
        x, y, z, w = first_rotation.to_float_quaternion()
```

## 📁 File Structure

```
tools/m2/
├── enhanced_parser.py          # Main enhanced parser with all features
├── quaternion.py               # M2CompQuat and M2Track classes  
├── coordinate_systems.py       # Coordinate transformation utilities
├── validation.py               # Validation modes and vertex fixing
├── test_enhanced_features.py   # Comprehensive test suite
├── parser.py                   # Original base parser (preserved)
├── batch_test.py              # Existing batch testing
└── README.md                  # This documentation
```

## 🎯 Usage Examples

### Basic Enhanced Parsing
```bash
# Parse with default settings (permissive validation)
python enhanced_parser.py model.m2

# Use strict validation mode
python enhanced_parser.py model.m2 -v strict

# Transform coordinates to Blender
python enhanced_parser.py model.m2 -c blender

# Show detailed quaternion data
python enhanced_parser.py model.m2 -q

# Export enhanced data to JSON
python enhanced_parser.py model.m2 -e output.json
```

### Programmatic Usage
```python
from enhanced_parser import EnhancedM2Parser

# Create parser with enhanced features
parser = EnhancedM2Parser(
    "model.m2", 
    validation_mode="permissive",
    coordinate_system="blender"
)

# Parse with all enhancements
parser.load_file()
parser.parse_all_enhanced()

# Access enhanced data
print(f"Validation stats: {parser.validation_stats}")
print(f"Bones with animation: {len([b for b in parser.enhanced_bones if b.rotations])}")

# Show visual representation with new features
parser.create_enhanced_visual_representation()
```

## 🧪 Testing

Run the comprehensive test suite to verify all features:

```bash
python test_enhanced_features.py
```

The test suite covers:
- ✅ Quaternion parsing with X-component negation  
- ✅ Coordinate transformations for all target systems
- ✅ All validation modes with edge cases
- ✅ Animation track parsing (pre-WotLK and WotLK+ formats)
- ✅ Integration tests combining multiple features
- ✅ Real corruption scenarios from issue reports

## 🔍 Feature Comparison: Python vs Rust

| Feature | Python (Before) | Python (Enhanced) | Rust Implementation |
|---------|----------------|-------------------|-------------------|
| **Quaternion Support** | ❌ Missing | ✅ Complete | ✅ Complete |
| **X-Component Negation** | ❌ Missing | ✅ Matches pywowlib | ✅ Matches pywowlib |
| **Coordinate Transforms** | ❌ Missing | ✅ Blender/Unity/Unreal | ✅ Blender/Unity/Unreal |
| **Validation Modes** | ❌ Basic only | ✅ Strict/Permissive/None | ✅ Strict/Permissive/None |
| **Animation Tracks** | ❌ Skipped | ✅ Full parsing | ✅ Full parsing |
| **Bone Index Validation** | ❌ Missing | ✅ With edge cases | ✅ With edge cases |
| **NaN Value Handling** | ❌ Missing | ✅ Complete | ✅ Complete |
| **Version Support** | ✅ v256 only | ✅ v256-264+ | ✅ v256-264+ |

## 🚨 Critical Fixes Implemented

The enhanced parser fixes all critical issues identified in the analysis:

### 1. **Quaternion Animation Data** 
- **Issue**: Rotation tracks were completely skipped
- **Fix**: Full M2CompQuat parsing with proper X-component negation
- **Impact**: Enables proper bone rotations for 3D applications

### 2. **Bone Index Corruption**
- **Issue**: 17% of vertices had invalid bone indices (e.g., index 196 when only 34 bones exist)  
- **Fix**: Smart validation that clamps to valid range while preserving intentional data
- **Impact**: Prevents crashes and rendering corruption

### 3. **Zero Bone Weight Vertices**
- **Issue**: 17% of vertices had zero bone weights, making them unmovable
- **Fix**: Validation modes that can fix corruption while preserving static geometry
- **Impact**: Proper vertex animation behavior

### 4. **Coordinate System Incompatibility**  
- **Issue**: WoW coordinates don't match 3D application expectations
- **Fix**: Accurate transformations for Blender, Unity, and Unreal Engine
- **Impact**: Direct import into 3D applications without manual fixes

## 📋 Migration Guide

### From Original Parser
```python
# Old way
from parser import ComprehensiveM2Parser
parser = ComprehensiveM2Parser("model.m2")

# New way  
from enhanced_parser import EnhancedM2Parser
parser = EnhancedM2Parser(
    "model.m2",
    validation_mode="permissive",    # New feature
    coordinate_system="blender"      # New feature
)
```

### New CLI Options
```bash
# Old command
python parser.py model.m2

# New command with enhanced features
python enhanced_parser.py model.m2 \
  --validation-mode permissive \
  --coordinate-system blender \
  --show-quaternions
```

## 🎉 Parity Achievement

The enhanced Python parser now achieves **100% parity** with the Rust implementation:

- ✅ **All critical features implemented**
- ✅ **Same validation behavior**  
- ✅ **Identical quaternion handling**
- ✅ **Same coordinate transformations**
- ✅ **Complete animation data access**
- ✅ **Comprehensive test coverage**

This enables Python users to have the same advanced M2 parsing capabilities as Rust users, with production-ready quality and reliability.

## 🔗 Related Files

- **Rust Implementation**: `/file-formats/graphics/wow-m2/src/`
- **Test Data**: Available in WoW installation directories specified in `CLAUDE.md`
- **Integration Examples**: See `/file-formats/graphics/wow-m2/examples/`

## 📝 Notes

- The enhanced parser maintains backward compatibility with the original parser
- All new features are optional and can be disabled for legacy behavior  
- Performance is optimized with batching and SIMD-ready transformations
- Memory usage is controlled with configurable limits on animation data