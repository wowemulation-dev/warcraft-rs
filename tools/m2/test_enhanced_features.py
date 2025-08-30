#!/usr/bin/env python3
"""
Comprehensive test cases for enhanced M2 parser features.
Tests all the critical functionality that achieves parity with the Rust implementation.
"""

import struct
import math
from typing import List, Tuple

from quaternion import M2CompQuat, M2Track, M2TrackBase
from coordinate_systems import (
    CoordinateSystem, CoordinateTransformer, transform_position, transform_quaternion,
    Vec3, Quaternion, get_transformation_matrix
)
from validation import ValidationMode, VertexValidator, M2VertexValidator, create_validator


class TestM2CompQuat:
    """Test compressed quaternion functionality"""
    
    def test_parse_with_x_negation(self):
        """Test that X component is negated during parsing to match pywowlib"""
        # Create test data: x=16384, y=32767, z=0, w=16383
        data = struct.pack('<hhhh', 16384, 32767, 0, 16383)
        
        quat = M2CompQuat.parse(data, 0)
        
        # X component should be negated during parsing
        assert quat.x == -16384, f"Expected -16384, got {quat.x}"
        assert quat.y == 32767, f"Expected 32767, got {quat.y}"
        assert quat.z == 0, f"Expected 0, got {quat.z}"
        assert quat.w == 16383, f"Expected 16383, got {quat.w}"
    
    def test_to_float_quaternion(self):
        """Test conversion to normalized float quaternion"""
        quat = M2CompQuat(x=16384, y=-16384, z=0, w=32767)
        
        x, y, z, w = quat.to_float_quaternion()
        
        # Values should be normalized to [-1, 1] range
        assert abs(x - 0.5) < 0.01, f"Expected ~0.5, got {x}"
        assert abs(y + 0.5) < 0.01, f"Expected ~-0.5, got {y}" 
        assert abs(z) < 0.001, f"Expected ~0, got {z}"
        assert abs(w - 1.0) < 0.01, f"Expected ~1.0, got {w}"
    
    def test_from_float_quaternion(self):
        """Test creation from float values"""
        quat = M2CompQuat.from_float_quaternion(0.707, 0.0, 0.707, 0.0)
        
        # Check values are properly scaled
        expected_x = int(0.707 * 32767)
        expected_z = int(0.707 * 32767)
        
        assert abs(quat.x - expected_x) <= 1, f"X component incorrect: {quat.x} vs {expected_x}"
        assert abs(quat.y) <= 1, f"Y component should be ~0: {quat.y}"
        assert abs(quat.z - expected_z) <= 1, f"Z component incorrect: {quat.z} vs {expected_z}"
        assert abs(quat.w) <= 1, f"W component should be ~0: {quat.w}"
    
    def test_roundtrip_conversion(self):
        """Test that float->compressed->float roundtrip preserves values"""
        original = (0.707, 0.0, 0.707, 0.0)
        
        quat = M2CompQuat.from_float_quaternion(*original)
        recovered = quat.to_float_quaternion()
        
        for i, (orig, rec) in enumerate(zip(original, recovered)):
            assert abs(orig - rec) < 0.01, f"Component {i}: {orig} -> {rec} (diff: {abs(orig-rec)})"
    
    def test_normalize_float(self):
        """Test quaternion normalization"""
        quat = M2CompQuat(x=16384, y=16384, z=16384, w=16384)
        
        x, y, z, w = quat.normalize_float()
        
        # Should be normalized (magnitude = 1)
        magnitude = math.sqrt(x*x + y*y + z*z + w*w)
        assert abs(magnitude - 1.0) < 0.001, f"Magnitude should be 1.0, got {magnitude}"
    
    def test_to_euler_degrees(self):
        """Test conversion to Euler angles"""
        # Test identity quaternion
        quat = M2CompQuat.from_float_quaternion(0.0, 0.0, 0.0, 1.0)
        pitch, yaw, roll = quat.to_euler_degrees()
        
        # Identity should give zero rotation
        assert abs(pitch) < 0.1, f"Pitch should be ~0, got {pitch}"
        assert abs(yaw) < 0.1, f"Yaw should be ~0, got {yaw}" 
        assert abs(roll) < 0.1, f"Roll should be ~0, got {roll}"


class TestM2Track:
    """Test M2Track parsing functionality"""
    
    def test_parse_pre_wotlk_format(self):
        """Test parsing pre-WotLK format (version < 264) with ranges field"""
        # Create test data for pre-264 format
        data = bytearray()
        
        # M2TrackBase: interpolation_type=1 (Linear), global_sequence=65535
        data.extend(struct.pack('<HH', 1, 65535))
        
        # Ranges M2Array: count=1, offset=100
        data.extend(struct.pack('<II', 1, 100))
        
        # Timestamps M2Array: count=3, offset=200  
        data.extend(struct.pack('<II', 3, 200))
        
        # Values M2Array: count=3, offset=300
        data.extend(struct.pack('<II', 3, 300))
        
        track = M2Track.parse(bytes(data), 0, 256)  # Version 256 (pre-264)
        
        assert track.base.interpolation_type == 1
        assert track.base.global_sequence == 65535
        assert track.ranges is not None
        assert track.ranges['count'] == 1
        assert track.ranges['offset'] == 100
        assert track.timestamps['count'] == 3
        assert track.timestamps['offset'] == 200
        assert track.values['count'] == 3
        assert track.values['offset'] == 300
    
    def test_parse_wotlk_format(self):
        """Test parsing WotLK+ format (version >= 264) without ranges field"""
        # Create test data for 264+ format
        data = bytearray()
        
        # M2TrackBase: interpolation_type=2 (Bezier), global_sequence=5
        data.extend(struct.pack('<HH', 2, 5))
        
        # Timestamps M2Array: count=5, offset=400
        data.extend(struct.pack('<II', 5, 400))
        
        # Values M2Array: count=5, offset=500  
        data.extend(struct.pack('<II', 5, 500))
        
        track = M2Track.parse(bytes(data), 0, 264)  # Version 264 (WotLK+)
        
        assert track.base.interpolation_type == 2
        assert track.base.global_sequence == 5
        assert track.ranges is None  # Should not have ranges in 264+
        assert track.timestamps['count'] == 5
        assert track.timestamps['offset'] == 400
        assert track.values['count'] == 5
        assert track.values['offset'] == 500
        assert track.uses_global_sequence() == True
    
    def test_track_methods(self):
        """Test M2Track utility methods"""
        # Create a track with data
        track = M2Track()
        track.timestamps = {'count': 3, 'offset': 100}
        track.values = {'count': 3, 'offset': 200}
        
        assert track.has_data() == True
        assert track.is_static() == False
        
        # Create static track
        static_track = M2Track()
        static_track.timestamps = {'count': 1, 'offset': 100}
        static_track.values = {'count': 1, 'offset': 200}
        
        assert static_track.is_static() == True


class TestCoordinateTransformations:
    """Test coordinate system transformations"""
    
    def test_blender_position_transform(self):
        """Test position transformation to Blender coordinates"""
        wow_pos = Vec3(100.0, 200.0, 50.0)  # 100N, 200W, 50Up
        blender_pos = transform_position(wow_pos, CoordinateSystem.BLENDER)
        
        # WoW (100N, 200W, 50Up) -> Blender (200Right, -100Backward, 50Up)
        assert blender_pos.x == 200.0, f"Expected 200.0, got {blender_pos.x}"
        assert blender_pos.y == -100.0, f"Expected -100.0, got {blender_pos.y}"
        assert blender_pos.z == 50.0, f"Expected 50.0, got {blender_pos.z}"
    
    def test_unity_position_transform(self):
        """Test position transformation to Unity coordinates"""
        wow_pos = Vec3(100.0, 200.0, 50.0)  # 100N, 200W, 50Up
        unity_pos = transform_position(wow_pos, CoordinateSystem.UNITY)
        
        # WoW (100N, 200W, 50Up) -> Unity (-200Left, 50Up, 100Forward)
        assert unity_pos.x == -200.0, f"Expected -200.0, got {unity_pos.x}"
        assert unity_pos.y == 50.0, f"Expected 50.0, got {unity_pos.y}"
        assert unity_pos.z == 100.0, f"Expected 100.0, got {unity_pos.z}"
    
    def test_unreal_position_transform(self):
        """Test position transformation to Unreal coordinates"""
        wow_pos = Vec3(100.0, 200.0, 50.0)
        unreal_pos = transform_position(wow_pos, CoordinateSystem.UNREAL_ENGINE)
        
        # WoW (100N, 200W, 50Up) -> Unreal (100Forward, -200Left, 50Up)
        assert unreal_pos.x == 100.0, f"Expected 100.0, got {unreal_pos.x}"
        assert unreal_pos.y == -200.0, f"Expected -200.0, got {unreal_pos.y}"
        assert unreal_pos.z == 50.0, f"Expected 50.0, got {unreal_pos.z}"
    
    def test_quaternion_transforms(self):
        """Test quaternion transformations"""
        identity = Quaternion(0.0, 0.0, 0.0, 1.0)
        
        blender_quat = transform_quaternion(identity, CoordinateSystem.BLENDER)
        unity_quat = transform_quaternion(identity, CoordinateSystem.UNITY)
        unreal_quat = transform_quaternion(identity, CoordinateSystem.UNREAL_ENGINE)
        
        # Identity quaternion should remain identity in all systems
        assert blender_quat.x == 0.0 and blender_quat.y == 0.0 and blender_quat.z == 0.0 and blender_quat.w == 1.0
        assert unity_quat.x == 0.0 and unity_quat.y == 0.0 and unity_quat.z == 0.0 and unity_quat.w == 1.0
        assert unreal_quat.x == 0.0 and unreal_quat.y == 0.0 and unreal_quat.z == 0.0 and unreal_quat.w == 1.0
    
    def test_coordinate_transformer(self):
        """Test batch coordinate transformation"""
        transformer = CoordinateTransformer(CoordinateSystem.BLENDER)
        
        positions = [
            Vec3(0.0, 0.0, 0.0),
            Vec3(100.0, 200.0, 50.0),
        ]
        
        transformed = transformer.transform_positions(positions)
        
        assert transformed[0].x == 0.0 and transformed[0].y == 0.0 and transformed[0].z == 0.0
        assert transformed[1].x == 200.0 and transformed[1].y == -100.0 and transformed[1].z == 50.0
    
    def test_cardinal_directions(self):
        """Test that cardinal directions transform correctly"""
        # North in WoW
        north = Vec3(1.0, 0.0, 0.0)
        blender_north = transform_position(north, CoordinateSystem.BLENDER)
        assert blender_north.x == 0.0 and blender_north.y == -1.0 and blender_north.z == 0.0
        
        # West in WoW
        west = Vec3(0.0, 1.0, 0.0) 
        blender_west = transform_position(west, CoordinateSystem.BLENDER)
        assert blender_west.x == 1.0 and blender_west.y == 0.0 and blender_west.z == 0.0
        
        # Up in WoW
        up = Vec3(0.0, 0.0, 1.0)
        blender_up = transform_position(up, CoordinateSystem.BLENDER)
        assert blender_up.x == 0.0 and blender_up.y == 0.0 and blender_up.z == 1.0


class TestVertexValidation:
    """Test vertex validation modes"""
    
    def test_validation_mode_strict(self):
        """Test strict validation mode - should fix all zero weights"""
        bone_indices = [0, 1, 2, 3]
        bone_weights = [0, 0, 0, 0]  # All zero weights
        bone_count = 10
        
        result = VertexValidator.validate_bone_data(
            bone_indices, bone_weights, bone_count, ValidationMode.STRICT
        )
        
        # Strict mode should force weight on first bone
        assert result.bone_weights[0] == 255, f"Expected 255, got {result.bone_weights[0]}"
        assert result.bone_weights[1] == 0, f"Expected 0, got {result.bone_weights[1]}"
        assert result.bone_indices[0] == 0, f"Expected 0, got {result.bone_indices[0]}"
        assert result.fixed_weights == True
    
    def test_validation_mode_permissive_valid_data(self):
        """Test permissive mode with valid data - should preserve zero weights"""
        bone_indices = [0, 1, 2, 3]  # Valid indices
        bone_weights = [0, 0, 0, 0]  # Zero weights for static geometry
        bone_count = 10
        
        result = VertexValidator.validate_bone_data(
            bone_indices, bone_weights, bone_count, ValidationMode.PERMISSIVE
        )
        
        # Permissive mode should preserve zero weights when indices are valid
        assert result.bone_weights == [0, 0, 0, 0], f"Expected [0,0,0,0], got {result.bone_weights}"
        assert result.fixed_weights == False
        assert result.corruption_detected == False
    
    def test_validation_mode_permissive_corrupted_data(self):
        """Test permissive mode with corrupted data - should fix it"""
        bone_indices = [200, 150, 100, 255]  # Invalid indices for 10-bone model
        bone_weights = [0, 0, 0, 0]  # Zero weights + invalid indices = corruption
        bone_count = 10
        
        result = VertexValidator.validate_bone_data(
            bone_indices, bone_weights, bone_count, ValidationMode.PERMISSIVE
        )
        
        # Should detect corruption and fix it
        assert result.bone_indices == [0, 0, 0, 0], f"Expected [0,0,0,0], got {result.bone_indices}"
        assert result.bone_weights[0] == 255, f"Expected 255, got {result.bone_weights[0]}"
        assert result.fixed_indices == True
        assert result.fixed_weights == True
        assert result.corruption_detected == True
    
    def test_validation_mode_none(self):
        """Test none validation mode - should preserve all data"""
        bone_indices = [200, 150, 100, 255]  # Invalid indices
        bone_weights = [0, 0, 0, 0]  # Zero weights
        bone_count = 10
        
        result = VertexValidator.validate_bone_data(
            bone_indices, bone_weights, bone_count, ValidationMode.NONE
        )
        
        # None mode should preserve everything
        assert result.bone_indices == [200, 150, 100, 255], f"Indices should be preserved"
        assert result.bone_weights == [0, 0, 0, 0], f"Weights should be preserved"
        assert result.fixed_indices == False
        assert result.fixed_weights == False
    
    def test_bone_index_validation_edge_cases(self):
        """Test edge cases for bone index validation"""
        # Test with bone_count=255 (boundary case)
        bone_indices = [254, 255, 0, 100]  # 254 valid, 255 invalid
        bone_weights = [255, 0, 0, 0]
        bone_count = 255
        
        result = VertexValidator.validate_bone_data(
            bone_indices, bone_weights, bone_count, ValidationMode.PERMISSIVE
        )
        
        # 254 should be preserved, 255 should be clamped
        assert result.bone_indices[0] == 254, f"254 should be valid"
        assert result.bone_indices[1] == 0, f"255 should be clamped to 0"
        assert result.fixed_indices == True
    
    def test_comprehensive_validation_real_scenario(self):
        """Test comprehensive validation with real corruption scenario"""
        # Real data from issue report: Rabbit.m2 with 34 bones
        bone_indices = [51, 196, 141, 62]  # All invalid for 34-bone model
        bone_weights = [0, 0, 0, 0]  # Zero weights
        bone_count = 34
        
        result = VertexValidator.validate_bone_data(
            bone_indices, bone_weights, bone_count, ValidationMode.STRICT
        )
        
        # All issues should be fixed
        assert all(idx < 34 for idx in result.bone_indices), f"All indices should be < 34: {result.bone_indices}"
        assert sum(result.bone_weights) > 0, f"Total weight should be > 0: {sum(result.bone_weights)}"
        assert result.bone_weights[0] == 255, f"First bone should get full weight"
        assert result.bone_indices[0] == 0, f"First bone should be root"
        assert result.fixed_indices == True
        assert result.fixed_weights == True
        assert result.corruption_detected == True
    
    def test_nan_validation(self):
        """Test NaN value validation"""
        # Test position with NaN
        position = (float('nan'), 2.0, 3.0)
        fixed_pos = VertexValidator.validate_nan_values(position)
        
        assert fixed_pos[0] == 0.0, f"NaN should be replaced with 0.0"
        assert fixed_pos[1] == 2.0, f"Valid value should be preserved"
        assert fixed_pos[2] == 3.0, f"Valid value should be preserved"
    
    def test_normal_validation(self):
        """Test normal vector validation and normalization"""
        # Test zero normal
        normal = (0.0, 0.0, 0.0)
        fixed_normal = VertexValidator.validate_normal_vector(normal)
        
        assert fixed_normal == (0.0, 0.0, 1.0), f"Zero normal should become up vector"
        
        # Test unnormalized normal
        normal = (1.0, 1.0, 1.0)
        fixed_normal = VertexValidator.validate_normal_vector(normal)
        
        # Should be normalized
        magnitude = math.sqrt(sum(x*x for x in fixed_normal))
        assert abs(magnitude - 1.0) < 0.001, f"Normal should be normalized"


class TestM2VertexValidator:
    """Test high-level vertex validator"""
    
    def test_complete_vertex_validation(self):
        """Test validation of complete vertex data"""
        validator = M2VertexValidator(ValidationMode.PERMISSIVE)
        
        vertex_data = {
            'position': [1.0, 2.0, 3.0],
            'bone_weights': [0, 0, 0, 0],
            'bone_indices': [51, 196, 141, 62],  # Invalid for 34-bone model
            'normal': [0.5, 0.5, 0.707],
            'tex_coords': [0.25, 0.75],
            'tex_coords2': [0.1, 0.9],
        }
        
        validated = validator.validate_vertex(vertex_data, bone_count=34)
        
        # Check that validation was applied
        assert '_validation_applied' in validated
        assert validated['_fixed_indices'] == True
        assert validated['_corruption_detected'] == True
        
        # Check that indices were fixed
        assert all(idx < 34 for idx in validated['bone_indices'])
    
    def test_batch_validation(self):
        """Test batch validation of multiple vertices"""
        validator = M2VertexValidator(ValidationMode.STRICT)
        
        vertices = [
            {
                'position': [1.0, 2.0, 3.0],
                'bone_weights': [0, 0, 0, 0],
                'bone_indices': [100, 200, 150, 255],
                'normal': [0.0, 1.0, 0.0],
                'tex_coords': [0.0, 1.0],
                'tex_coords2': [0.5, 0.5],
            }
        ] * 3  # 3 identical corrupted vertices
        
        validated_batch = validator.validate_vertices(vertices, bone_count=10)
        stats = validator.get_validation_stats(validated_batch)
        
        assert stats['total_vertices'] == 3
        assert stats['indices_fixed'] == 3
        assert stats['weights_fixed'] == 3
        assert stats['corruption_detected'] == 3


class TestIntegration:
    """Integration tests combining multiple features"""
    
    def test_quaternion_coordinate_transform_integration(self):
        """Test quaternion parsing + coordinate transformation"""
        # Create quaternion data
        data = struct.pack('<hhhh', 16384, 0, 16384, 16384)  # 45° rotation
        quat = M2CompQuat.parse(data, 0)
        
        # Convert to float and apply coordinate transformation
        x, y, z, w = quat.to_float_quaternion()
        coord_quat = Quaternion(x, y, z, w)
        
        # Transform to Blender coordinates
        blender_quat = transform_quaternion(coord_quat, CoordinateSystem.BLENDER)
        
        # Verify the transformation was applied
        assert blender_quat != coord_quat  # Should be different
        assert blender_quat.w == coord_quat.w  # W should be unchanged
    
    def test_validation_coordinate_transform_integration(self):
        """Test validation + coordinate transformation"""
        validator = M2VertexValidator(ValidationMode.PERMISSIVE)
        transformer = CoordinateTransformer(CoordinateSystem.BLENDER)
        
        # Corrupt vertex data
        vertex_data = {
            'position': [100.0, 200.0, 50.0],  # WoW coordinates
            'bone_weights': [0, 0, 0, 0],
            'bone_indices': [200, 150, 100, 255],  # Invalid
            'normal': [0.0, 1.0, 0.0],
            'tex_coords': [0.0, 1.0],
            'tex_coords2': [0.5, 0.5],
        }
        
        # Validate first
        validated = validator.validate_vertex(vertex_data, bone_count=10)
        
        # Then transform coordinates
        pos_vec = Vec3(*validated['position'])
        transformed_pos = transformer.transform_position(pos_vec)
        
        # Verify both operations worked
        assert validated['_fixed_indices'] == True  # Validation worked
        assert transformed_pos.x == 200.0  # Coordinate transform worked
        assert transformed_pos.y == -100.0
        assert transformed_pos.z == 50.0


def run_all_tests():
    """Run all tests and report results"""
    test_classes = [
        TestM2CompQuat,
        TestM2Track, 
        TestCoordinateTransformations,
        TestVertexValidation,
        TestM2VertexValidator,
        TestIntegration,
    ]
    
    total_tests = 0
    passed_tests = 0
    failed_tests = []
    
    print("🧪 Running Enhanced M2 Parser Tests...")
    print("=" * 50)
    
    for test_class in test_classes:
        print(f"\n📋 Testing {test_class.__name__}")
        
        instance = test_class()
        test_methods = [method for method in dir(instance) if method.startswith('test_')]
        
        for method_name in test_methods:
            total_tests += 1
            try:
                method = getattr(instance, method_name)
                method()
                print(f"  ✅ {method_name}")
                passed_tests += 1
            except Exception as e:
                print(f"  ❌ {method_name}: {e}")
                failed_tests.append((test_class.__name__, method_name, str(e)))
    
    print(f"\n" + "=" * 50)
    print(f"📊 Test Results: {passed_tests}/{total_tests} passed")
    
    if failed_tests:
        print(f"❌ Failed Tests ({len(failed_tests)}):")
        for test_class, method, error in failed_tests:
            print(f"  - {test_class}.{method}: {error}")
        return False
    else:
        print("🎉 All tests passed! Enhanced M2 Parser is ready.")
        return True


if __name__ == "__main__":
    success = run_all_tests()
    exit(0 if success else 1)