"""
Coordinate system transformations for World of Warcraft models.

This module provides utilities for transforming WoW's coordinate system to common
3D application coordinate systems like Blender, Unity, and Unreal Engine.

WoW Coordinate System:
- X-axis: North (positive X = north)
- Y-axis: West (positive Y = west)  
- Z-axis: Up (positive Z = up, 0 = sea level)
"""

import math
from enum import Enum
from typing import List, Tuple
from dataclasses import dataclass


class CoordinateSystem(Enum):
    """Target coordinate systems for transformation."""
    BLENDER = "blender"          # Right-handed: Right=X+, Forward=Y+, Up=Z+
    UNITY = "unity"              # Left-handed: Right=X+, Up=Y+, Forward=Z+
    UNREAL_ENGINE = "unreal"     # Left-handed: Forward=X+, Right=Y+, Up=Z+


@dataclass 
class Vec3:
    """3D Vector for coordinate transformations."""
    x: float
    y: float 
    z: float
    
    def __add__(self, other):
        return Vec3(self.x + other.x, self.y + other.y, self.z + other.z)
    
    def __sub__(self, other):
        return Vec3(self.x - other.x, self.y - other.y, self.z - other.z)
    
    def __mul__(self, scalar):
        return Vec3(self.x * scalar, self.y * scalar, self.z * scalar)
    
    def magnitude(self) -> float:
        return math.sqrt(self.x**2 + self.y**2 + self.z**2)


@dataclass
class Quaternion:
    """Quaternion for rotation transformations."""
    x: float
    y: float
    z: float
    w: float
    
    def normalize(self) -> 'Quaternion':
        """Return normalized quaternion."""
        mag = math.sqrt(self.x**2 + self.y**2 + self.z**2 + self.w**2)
        if mag < 1e-8:
            return Quaternion(0.0, 0.0, 0.0, 1.0)
        return Quaternion(self.x/mag, self.y/mag, self.z/mag, self.w/mag)


def transform_position(wow_pos: Vec3, target: CoordinateSystem) -> Vec3:
    """
    Transform a 3D position from WoW coordinates to the target coordinate system.
    
    Args:
        wow_pos: Position in WoW coordinate system (North=X+, West=Y+, Up=Z+)
        target: Target coordinate system
        
    Returns:
        Transformed position vector
        
    Examples:
        >>> wow_pos = Vec3(x=100.0, y=200.0, z=50.0)  # 100N, 200W, 50Up
        >>> blender_pos = transform_position(wow_pos, CoordinateSystem.BLENDER)
        >>> # Blender: Right=X+, Forward=Y+, Up=Z+
        >>> # WoW (100N, 200W, 50Up) -> Blender (200Right, -100Backward, 50Up)
        >>> print(f"Blender: ({blender_pos.x}, {blender_pos.y}, {blender_pos.z})")
    """
    if target == CoordinateSystem.BLENDER:
        return Vec3(
            x=wow_pos.y,   # WoW Y (west) → Blender X (right)
            y=-wow_pos.x,  # WoW X (north) → Blender -Y (backward)
            z=wow_pos.z,   # WoW Z (up) → Blender Z (up)
        )
    elif target == CoordinateSystem.UNITY:
        return Vec3(
            x=-wow_pos.y,  # WoW Y (west) → Unity -X (left)
            y=wow_pos.z,   # WoW Z (up) → Unity Y (up)
            z=wow_pos.x,   # WoW X (north) → Unity Z (forward)
        )
    elif target == CoordinateSystem.UNREAL_ENGINE:
        return Vec3(
            x=wow_pos.x,   # WoW X (north) → Unreal X (forward)
            y=-wow_pos.y,  # WoW Y (west) → Unreal -Y (left)
            z=wow_pos.z,   # WoW Z (up) → Unreal Z (up)
        )
    else:
        raise ValueError(f"Unsupported coordinate system: {target}")


def transform_quaternion(wow_quat: Quaternion, target: CoordinateSystem) -> Quaternion:
    """
    Transform a quaternion rotation from WoW coordinates to the target coordinate system.
    
    Args:
        wow_quat: Rotation in WoW coordinate system
        target: Target coordinate system
        
    Returns:
        Transformed quaternion
        
    Examples:
        >>> wow_rot = Quaternion(x=0.0, y=0.707, z=0.0, w=0.707)
        >>> blender_rot = transform_quaternion(wow_rot, CoordinateSystem.BLENDER)
    """
    if target == CoordinateSystem.BLENDER:
        return Quaternion(
            x=wow_quat.y,   # WoW Y → Blender X
            y=-wow_quat.x,  # WoW X → Blender -Y
            z=wow_quat.z,   # WoW Z → Blender Z
            w=wow_quat.w,   # W component unchanged
        )
    elif target == CoordinateSystem.UNITY:
        return Quaternion(
            x=wow_quat.y,   # WoW Y → Unity X
            y=-wow_quat.z,  # WoW Z → Unity -Y
            z=-wow_quat.x,  # WoW X → Unity -Z
            w=wow_quat.w,   # W component unchanged
        )
    elif target == CoordinateSystem.UNREAL_ENGINE:
        return Quaternion(
            x=-wow_quat.x,  # WoW X → Unreal -X
            y=wow_quat.y,   # WoW Y → Unreal Y
            z=-wow_quat.z,  # WoW Z → Unreal -Z
            w=wow_quat.w,   # W component unchanged
        )
    else:
        raise ValueError(f"Unsupported coordinate system: {target}")


def transform_vector2(wow_vec: Tuple[float, float], target: CoordinateSystem) -> Tuple[float, float]:
    """
    Transform a 2D vector (typically texture coordinates).
    
    Note: In most cases, texture coordinates don't need coordinate system transformation
    since they represent UV mapping rather than 3D spatial coordinates.
    
    Args:
        wow_vec: 2D vector as (x, y) tuple
        target: Target coordinate system (currently ignored)
        
    Returns:
        Same vector (texture coordinates typically don't need transformation)
    """
    # Texture coordinates typically don't need transformation
    return wow_vec


class CoordinateTransformer:
    """
    A coordinate transformer that can efficiently transform multiple related coordinates
    while maintaining consistency across a model or scene.
    """
    
    def __init__(self, target: CoordinateSystem):
        """
        Create a new coordinate transformer for the target system.
        
        Args:
            target: Target coordinate system
        """
        self.target = target
    
    def transform_position(self, wow_pos: Vec3) -> Vec3:
        """Transform a single position."""
        return transform_position(wow_pos, self.target)
    
    def transform_quaternion(self, wow_quat: Quaternion) -> Quaternion:
        """Transform a single quaternion."""  
        return transform_quaternion(wow_quat, self.target)
    
    def transform_positions(self, positions: List[Vec3]) -> List[Vec3]:
        """Transform multiple positions efficiently."""
        return [self.transform_position(pos) for pos in positions]
    
    def transform_quaternions(self, quaternions: List[Quaternion]) -> List[Quaternion]:
        """Transform multiple quaternions efficiently."""
        return [self.transform_quaternion(quat) for quat in quaternions]
    
    def transform_vertex_positions(self, vertices: List[dict]) -> List[Vec3]:
        """
        Transform vertex positions from a model.
        
        Args:
            vertices: List of vertex dictionaries with 'position' key
            
        Returns:
            List of transformed position vectors
        """
        positions = []
        for vertex in vertices:
            pos = vertex.get('position', {})
            if isinstance(pos, dict):
                wow_pos = Vec3(pos.get('x', 0), pos.get('y', 0), pos.get('z', 0))
            else:
                wow_pos = pos  # Assume it's already a Vec3
            positions.append(self.transform_position(wow_pos))
        return positions
    
    def batch_transform_bones(self, bones: List[dict]) -> List[dict]:
        """
        Transform bone data including positions and rotations.
        
        Args:
            bones: List of bone dictionaries with 'pivot' and optionally 'rotation' keys
            
        Returns:
            List of transformed bone dictionaries
        """
        transformed_bones = []
        for bone in bones:
            transformed_bone = bone.copy()
            
            # Transform pivot point
            if 'pivot' in bone:
                pivot = bone['pivot']
                if isinstance(pivot, dict):
                    wow_pivot = Vec3(pivot.get('x', 0), pivot.get('y', 0), pivot.get('z', 0))
                else:
                    wow_pivot = pivot
                transformed_bone['pivot'] = self.transform_position(wow_pivot)
            
            # Transform rotation if present
            if 'rotation' in bone:
                rot = bone['rotation']
                if isinstance(rot, dict):
                    wow_rot = Quaternion(rot.get('x', 0), rot.get('y', 0), rot.get('z', 0), rot.get('w', 1))
                else:
                    wow_rot = rot
                transformed_bone['rotation'] = self.transform_quaternion(wow_rot)
            
            transformed_bones.append(transformed_bone)
        
        return transformed_bones


def get_transformation_matrix(target: CoordinateSystem) -> List[List[float]]:
    """
    Get the transformation matrix for converting from WoW coordinates to the target system.
    
    Args:
        target: Target coordinate system
        
    Returns:
        4x4 transformation matrix as list of lists
    """
    if target == CoordinateSystem.BLENDER:
        return [
            [0.0, 1.0, 0.0, 0.0],   # X column: Y → X
            [-1.0, 0.0, 0.0, 0.0],  # Y column: -X → Y  
            [0.0, 0.0, 1.0, 0.0],   # Z column: Z → Z
            [0.0, 0.0, 0.0, 1.0],   # W column
        ]
    elif target == CoordinateSystem.UNITY:
        return [
            [0.0, -1.0, 0.0, 0.0],  # X column: -Y → X
            [0.0, 0.0, 1.0, 0.0],   # Y column: Z → Y
            [1.0, 0.0, 0.0, 0.0],   # Z column: X → Z  
            [0.0, 0.0, 0.0, 1.0],   # W column
        ]
    elif target == CoordinateSystem.UNREAL_ENGINE:
        return [
            [1.0, 0.0, 0.0, 0.0],   # X column: X → X
            [0.0, -1.0, 0.0, 0.0],  # Y column: -Y → Y
            [0.0, 0.0, 1.0, 0.0],   # Z column: Z → Z
            [0.0, 0.0, 0.0, 1.0],   # W column
        ]
    else:
        raise ValueError(f"Unsupported coordinate system: {target}")


def test_cardinal_directions():
    """Test that cardinal directions transform correctly."""
    
    # North in WoW
    north = Vec3(1.0, 0.0, 0.0)
    blender_north = transform_position(north, CoordinateSystem.BLENDER)
    print(f"WoW North {north} -> Blender {blender_north}")  # Should be backward in Blender
    
    # West in WoW  
    west = Vec3(0.0, 1.0, 0.0)
    blender_west = transform_position(west, CoordinateSystem.BLENDER)
    print(f"WoW West {west} -> Blender {blender_west}")  # Should be right in Blender
    
    # Up in WoW
    up = Vec3(0.0, 0.0, 1.0)
    blender_up = transform_position(up, CoordinateSystem.BLENDER)
    print(f"WoW Up {up} -> Blender {blender_up}")  # Should be up in Blender (unchanged)


if __name__ == "__main__":
    # Example usage
    print("Testing coordinate transformations...")
    
    # Test position transformation
    wow_pos = Vec3(100.0, 200.0, 50.0)  # 100N, 200W, 50Up
    
    blender_pos = transform_position(wow_pos, CoordinateSystem.BLENDER)
    unity_pos = transform_position(wow_pos, CoordinateSystem.UNITY)
    unreal_pos = transform_position(wow_pos, CoordinateSystem.UNREAL_ENGINE)
    
    print(f"\nWoW position: ({wow_pos.x}, {wow_pos.y}, {wow_pos.z})")
    print(f"Blender: ({blender_pos.x}, {blender_pos.y}, {blender_pos.z})")
    print(f"Unity: ({unity_pos.x}, {unity_pos.y}, {unity_pos.z})")
    print(f"Unreal: ({unreal_pos.x}, {unreal_pos.y}, {unreal_pos.z})")
    
    # Test quaternion transformation
    wow_rot = Quaternion(0.0, 0.707, 0.0, 0.707)
    blender_rot = transform_quaternion(wow_rot, CoordinateSystem.BLENDER)
    
    print(f"\nWoW rotation: ({wow_rot.x}, {wow_rot.y}, {wow_rot.z}, {wow_rot.w})")
    print(f"Blender rotation: ({blender_rot.x}, {blender_rot.y}, {blender_rot.z}, {blender_rot.w})")
    
    # Test batch transformation
    transformer = CoordinateTransformer(CoordinateSystem.BLENDER)
    positions = [Vec3(0.0, 0.0, 0.0), Vec3(100.0, 200.0, 50.0)]
    transformed = transformer.transform_positions(positions)
    
    print(f"\nBatch transformation to Blender:")
    for i, (orig, trans) in enumerate(zip(positions, transformed)):
        print(f"  Position {i}: ({orig.x}, {orig.y}, {orig.z}) -> ({trans.x}, {trans.y}, {trans.z})")
    
    print("\nTesting cardinal directions:")
    test_cardinal_directions()