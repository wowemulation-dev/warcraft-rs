"""
Validation modes for M2 vertex data parsing.

Provides different levels of validation and corruption fixes for M2 vertex data,
matching the behavior of the Rust implementation to ensure consistency.
"""

import math
from enum import Enum
from typing import List, Tuple, Optional, Dict, Any
from dataclasses import dataclass


class ValidationMode(Enum):
    """
    Validation mode for vertex data parsing.
    
    Controls how aggressive the validation fixes are when parsing vertex data.
    """
    STRICT = "strict"          # Apply all fixes aggressively - forces bone weights on vertices with zero weights
    PERMISSIVE = "permissive"  # Only fix clearly corrupted data - preserves intentional zero weights for static geometry  
    NONE = "none"              # No automatic fixes - preserves all original data


@dataclass
class ValidationResult:
    """Result of vertex data validation."""
    bone_indices: List[int]
    bone_weights: List[int] 
    fixed_indices: bool = False
    fixed_weights: bool = False
    corruption_detected: bool = False


class VertexValidator:
    """
    Validates and fixes vertex bone data to ensure data integrity.
    
    Fixes critical issues where vertices in vanilla/TBC models have:
    1. Bone indices that exceed the actual number of bones in the model
    2. Zero bone weights for all bones (making vertices unmovable) 
    3. Invalid weight distributions
    """
    
    @staticmethod
    def validate_bone_data(
        bone_indices: List[int],
        bone_weights: List[int], 
        bone_count: Optional[int],
        validation_mode: ValidationMode
    ) -> ValidationResult:
        """
        Validate and fix bone indices and weights.
        
        The behavior depends on the validation mode:
        - Strict: Applies all fixes aggressively, forces weights on zero-weight vertices
        - Permissive: Fixes out-of-bounds indices, preserves valid zero weights for static geometry
        - None: Preserves original data without any fixes
        
        Args:
            bone_indices: The raw bone indices from the vertex data
            bone_weights: The raw bone weights from the vertex data
            bone_count: The actual number of bones available in the model
            validation_mode: Controls how aggressive the validation fixes are
            
        Returns:
            ValidationResult with fixes applied based on validation mode
        """
        result = ValidationResult(
            bone_indices=bone_indices.copy(),
            bone_weights=bone_weights.copy()
        )
        
        # Skip all validation if mode is None
        if validation_mode == ValidationMode.NONE:
            return result
        
        # Skip validation if no bone count provided
        if bone_count is None:
            return result
        
        # Track if we had invalid indices before fixing them (for Permissive mode corruption detection)
        had_invalid_indices = any(idx >= bone_count for idx in bone_indices)
        result.corruption_detected = had_invalid_indices
        
        # Fix invalid bone indices (both Strict and Permissive modes)
        for i, bone_index in enumerate(result.bone_indices):
            if bone_index >= bone_count:
                # Invalid bone index found - clamp to valid range
                # Using 0 as a safe fallback (root bone)
                result.bone_indices[i] = 0
                result.fixed_indices = True
        
        # Fix zero bone weights issue - behavior depends on validation mode
        total_weight = sum(result.bone_weights)
        if total_weight == 0:
            if validation_mode == ValidationMode.STRICT:
                # Strict mode: Force bone weight assignment for zero-weight vertices
                # This ensures all vertices are animated but may break static geometry
                result.bone_weights[0] = 255
                result.bone_indices[0] = 0  # Ensure it's the root bone
                result.fixed_weights = True
            elif validation_mode == ValidationMode.PERMISSIVE:
                # Permissive mode: Preserve zero weights for static geometry
                # Only fix if this appears to be corruption (had invalid bone indices with zero weights)
                if had_invalid_indices:
                    # This looks like corruption - fix it
                    result.bone_weights[0] = 255
                    result.bone_indices[0] = 0
                    result.fixed_weights = True
                # Otherwise, preserve the zero weights as they may be intentional for static geometry
        
        return result
    
    @staticmethod
    def validate_nan_values(position: Tuple[float, float, float]) -> Tuple[float, float, float]:
        """
        Validate position vector for NaN values and fix them.
        
        Args:
            position: Position tuple (x, y, z)
            
        Returns:
            Fixed position tuple with NaN values replaced by 0.0
        """
        x, y, z = position
        
        # Replace NaN values with 0.0
        if math.isnan(x):
            x = 0.0
        if math.isnan(y):
            y = 0.0
        if math.isnan(z):
            z = 0.0
        
        return (x, y, z)
    
    @staticmethod
    def validate_normal_vector(normal: Tuple[float, float, float]) -> Tuple[float, float, float]:
        """
        Validate and normalize a normal vector.
        
        Args:
            normal: Normal vector tuple (x, y, z)
            
        Returns:
            Normalized normal vector or default up vector if invalid
        """
        x, y, z = normal
        
        # Check for NaN values
        if math.isnan(x) or math.isnan(y) or math.isnan(z):
            return (0.0, 0.0, 1.0)  # Default up vector
        
        # Calculate magnitude
        magnitude = math.sqrt(x*x + y*y + z*z)
        
        if magnitude < 1e-8:
            return (0.0, 0.0, 1.0)  # Default up vector if zero length
        
        # Normalize
        inv_mag = 1.0 / magnitude
        return (x * inv_mag, y * inv_mag, z * inv_mag)
    
    @staticmethod
    def validate_texture_coordinates(uv: Tuple[float, float]) -> Tuple[float, float]:
        """
        Validate texture coordinates for NaN values.
        
        Args:
            uv: UV coordinates tuple (u, v)
            
        Returns:
            Fixed UV coordinates with NaN values replaced by 0.0
        """
        u, v = uv
        
        # Replace NaN values with 0.0
        if math.isnan(u):
            u = 0.0
        if math.isnan(v):
            v = 0.0
        
        return (u, v)


class M2VertexValidator:
    """High-level validator for complete M2 vertex data."""
    
    def __init__(self, validation_mode: ValidationMode = ValidationMode.PERMISSIVE):
        """
        Initialize validator with specified mode.
        
        Args:
            validation_mode: Validation mode to use
        """
        self.validation_mode = validation_mode
        self.validator = VertexValidator()
    
    def validate_vertex(self, vertex_data: Dict[str, Any], bone_count: Optional[int] = None) -> Dict[str, Any]:
        """
        Validate complete vertex data.
        
        Args:
            vertex_data: Dictionary containing vertex data
            bone_count: Number of bones in the model for validation
            
        Returns:
            Dictionary with validated vertex data
        """
        validated = vertex_data.copy()
        
        # Validate position
        if 'position' in validated:
            pos = validated['position']
            if isinstance(pos, (list, tuple)) and len(pos) >= 3:
                validated['position'] = self.validator.validate_nan_values((pos[0], pos[1], pos[2]))
        
        # Validate normal  
        if 'normal' in validated:
            norm = validated['normal']
            if isinstance(norm, (list, tuple)) and len(norm) >= 3:
                validated['normal'] = self.validator.validate_normal_vector((norm[0], norm[1], norm[2]))
        
        # Validate texture coordinates
        if 'tex_coords' in validated:
            uv = validated['tex_coords']
            if isinstance(uv, (list, tuple)) and len(uv) >= 2:
                validated['tex_coords'] = self.validator.validate_texture_coordinates((uv[0], uv[1]))
        
        if 'tex_coords2' in validated:
            uv2 = validated['tex_coords2']
            if isinstance(uv2, (list, tuple)) and len(uv2) >= 2:
                validated['tex_coords2'] = self.validator.validate_texture_coordinates((uv2[0], uv2[1]))
        
        # Validate bone data
        if 'bone_indices' in validated and 'bone_weights' in validated:
            bone_indices = validated['bone_indices']
            bone_weights = validated['bone_weights']
            
            if isinstance(bone_indices, (list, tuple)) and isinstance(bone_weights, (list, tuple)):
                # Ensure we have at least 4 elements
                bone_indices = list(bone_indices) + [0] * (4 - len(bone_indices))
                bone_weights = list(bone_weights) + [0] * (4 - len(bone_weights))
                
                # Validate bone data
                result = self.validator.validate_bone_data(
                    bone_indices[:4], 
                    bone_weights[:4],
                    bone_count,
                    self.validation_mode
                )
                
                validated['bone_indices'] = result.bone_indices
                validated['bone_weights'] = result.bone_weights
                
                # Add validation metadata
                validated['_validation_applied'] = True
                validated['_fixed_indices'] = result.fixed_indices
                validated['_fixed_weights'] = result.fixed_weights
                validated['_corruption_detected'] = result.corruption_detected
        
        return validated
    
    def validate_vertices(self, vertices: List[Dict[str, Any]], bone_count: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Validate a list of vertices.
        
        Args:
            vertices: List of vertex data dictionaries
            bone_count: Number of bones in the model for validation
            
        Returns:
            List of validated vertex data dictionaries
        """
        return [self.validate_vertex(vertex, bone_count) for vertex in vertices]
    
    def get_validation_stats(self, vertices: List[Dict[str, Any]]) -> Dict[str, int]:
        """
        Get statistics about validation results.
        
        Args:
            vertices: List of validated vertex data dictionaries
            
        Returns:
            Dictionary with validation statistics
        """
        stats = {
            'total_vertices': len(vertices),
            'indices_fixed': 0,
            'weights_fixed': 0,
            'corruption_detected': 0,
            'zero_weight_vertices': 0,
        }
        
        for vertex in vertices:
            if vertex.get('_fixed_indices', False):
                stats['indices_fixed'] += 1
            if vertex.get('_fixed_weights', False):
                stats['weights_fixed'] += 1
            if vertex.get('_corruption_detected', False):
                stats['corruption_detected'] += 1
            
            # Check for zero weights
            bone_weights = vertex.get('bone_weights', [0, 0, 0, 0])
            if sum(bone_weights) == 0:
                stats['zero_weight_vertices'] += 1
        
        return stats


def create_validator(mode_name: str) -> M2VertexValidator:
    """
    Create a validator from a string mode name.
    
    Args:
        mode_name: Name of validation mode ("strict", "permissive", "none")
        
    Returns:
        M2VertexValidator instance
        
    Raises:
        ValueError: If mode name is not recognized
    """
    mode_map = {
        'strict': ValidationMode.STRICT,
        'permissive': ValidationMode.PERMISSIVE, 
        'none': ValidationMode.NONE,
    }
    
    mode = mode_map.get(mode_name.lower())
    if mode is None:
        raise ValueError(f"Unknown validation mode: {mode_name}. Valid modes: {list(mode_map.keys())}")
    
    return M2VertexValidator(mode)


if __name__ == "__main__":
    # Example usage and testing
    print("Testing M2 vertex validation...")
    
    # Test data with corruption issues
    corrupted_vertex = {
        'position': [1.0, 2.0, 3.0],
        'bone_weights': [0, 0, 0, 0],  # Zero weights - issue #1
        'bone_indices': [51, 196, 141, 62],  # Invalid indices for 34-bone model - issue #2
        'normal': [0.5, 0.5, 0.707],
        'tex_coords': [0.25, 0.75],
        'tex_coords2': [0.1, 0.9],
    }
    
    bone_count = 34  # Rabbit.m2 scenario
    
    # Test all validation modes
    modes = ['strict', 'permissive', 'none']
    
    for mode_name in modes:
        print(f"\n--- Testing {mode_name.upper()} mode ---")
        validator = create_validator(mode_name)
        
        validated = validator.validate_vertex(corrupted_vertex, bone_count)
        
        print(f"Original indices: {corrupted_vertex['bone_indices']}")
        print(f"Validated indices: {validated['bone_indices']}")
        print(f"Original weights: {corrupted_vertex['bone_weights']}")
        print(f"Validated weights: {validated['bone_weights']}")
        print(f"Corruption detected: {validated.get('_corruption_detected', False)}")
        print(f"Indices fixed: {validated.get('_fixed_indices', False)}")
        print(f"Weights fixed: {validated.get('_fixed_weights', False)}")
    
    # Test batch validation
    print(f"\n--- Batch validation test ---")
    vertices = [corrupted_vertex] * 3  # Test with 3 copies
    
    validator = create_validator('permissive')
    validated_batch = validator.validate_vertices(vertices, bone_count)
    stats = validator.get_validation_stats(validated_batch)
    
    print(f"Validation stats: {stats}")
    
    print("\nValidation testing complete!")