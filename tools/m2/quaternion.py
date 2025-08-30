"""
M2 Compressed Quaternion Support

Provides compressed quaternion parsing with X-component negation to match
the pywowlib reference implementation and maintain compatibility.
"""

import struct
import math
from typing import Tuple
from dataclasses import dataclass


@dataclass
class M2CompQuat:
    """
    Compressed quaternion using 16-bit integers for WoW model animations.
    
    Used for rotation animations to save space. The X component is negated
    during parsing to match Python reference implementation (pywowlib).
    
    Attributes:
        x: X component (16-bit signed integer)
        y: Y component (16-bit signed integer) 
        z: Z component (16-bit signed integer)
        w: W component (16-bit signed integer)
    """
    x: int
    y: int
    z: int
    w: int
    
    @classmethod
    def parse(cls, data: bytes, offset: int) -> 'M2CompQuat':
        """
        Parse a compressed quaternion from binary data.
        
        Note: X component is negated to match Python reference implementation (pywowlib).
        
        Args:
            data: Binary data to parse from
            offset: Offset in data to start parsing
            
        Returns:
            M2CompQuat instance with parsed values
            
        Raises:
            struct.error: If not enough data available
        """
        if offset + 8 > len(data):
            raise ValueError(f"Not enough data for M2CompQuat at offset {offset}")
        
        x, y, z, w = struct.unpack('<hhhh', data[offset:offset + 8])
        
        # Negate X component to match Python reference implementation
        return cls(x=-x, y=y, z=z, w=w)
    
    def to_float_quaternion(self) -> Tuple[float, float, float, float]:
        """
        Convert compressed quaternion to normalized float quaternion.
        
        The compressed values are typically normalized to [-1.0, 1.0] range.
        
        Returns:
            Tuple of (x, y, z, w) float values
        """
        scale = 1.0 / 32767.0
        return (
            self.x * scale,
            self.y * scale, 
            self.z * scale,
            self.w * scale
        )
    
    @classmethod
    def from_float_quaternion(cls, x: float, y: float, z: float, w: float) -> 'M2CompQuat':
        """
        Create compressed quaternion from normalized float values.
        
        Args:
            x: X component (normalized float)
            y: Y component (normalized float)
            z: Z component (normalized float) 
            w: W component (normalized float)
            
        Returns:
            M2CompQuat instance with compressed values
        """
        scale = 32767.0
        return cls(
            x=int(x * scale),
            y=int(y * scale),
            z=int(z * scale), 
            w=int(w * scale)
        )
    
    def normalize_float(self) -> Tuple[float, float, float, float]:
        """
        Convert to normalized float quaternion.
        
        Returns:
            Normalized quaternion as (x, y, z, w) tuple
        """
        x, y, z, w = self.to_float_quaternion()
        
        # Calculate magnitude
        magnitude = math.sqrt(x*x + y*y + z*z + w*w)
        
        if magnitude < 1e-8:
            # Return identity quaternion if magnitude is too small
            return (0.0, 0.0, 0.0, 1.0)
        
        # Normalize
        inv_mag = 1.0 / magnitude
        return (x * inv_mag, y * inv_mag, z * inv_mag, w * inv_mag)
    
    def to_euler_degrees(self) -> Tuple[float, float, float]:
        """
        Convert quaternion to Euler angles in degrees.
        
        Returns:
            Tuple of (pitch, yaw, roll) in degrees
        """
        x, y, z, w = self.normalize_float()
        
        # Roll (x-axis rotation)
        sin_r_cp = 2 * (w * x + y * z)
        cos_r_cp = 1 - 2 * (x * x + y * y)
        roll = math.atan2(sin_r_cp, cos_r_cp)
        
        # Pitch (y-axis rotation)  
        sin_p = 2 * (w * y - z * x)
        if abs(sin_p) >= 1:
            pitch = math.copysign(math.pi / 2, sin_p)  # Use 90 degrees if out of range
        else:
            pitch = math.asin(sin_p)
        
        # Yaw (z-axis rotation)
        sin_y_cp = 2 * (w * z + x * y)
        cos_y_cp = 1 - 2 * (y * y + z * z)
        yaw = math.atan2(sin_y_cp, cos_y_cp)
        
        # Convert to degrees
        return (
            math.degrees(pitch),
            math.degrees(yaw), 
            math.degrees(roll)
        )
    
    def __repr__(self) -> str:
        return f"M2CompQuat(x={self.x}, y={self.y}, z={self.z}, w={self.w})"
    
    def __str__(self) -> str:
        x, y, z, w = self.to_float_quaternion()
        return f"Quat({x:.3f}, {y:.3f}, {z:.3f}, {w:.3f})"


class M2TrackBase:
    """
    Base structure for M2Track that contains common fields.
    This represents the header part of an M2Track before the value data.
    """
    
    def __init__(self, interpolation_type: int = 0, global_sequence: int = 65535):
        """
        Initialize M2TrackBase.
        
        Args:
            interpolation_type: Type of interpolation (0=None, 1=Linear, 2=Bezier, 3=Hermite)
            global_sequence: Global sequence index (65535 = no global sequence)
        """
        self.interpolation_type = interpolation_type
        self.global_sequence = global_sequence
    
    @classmethod
    def parse(cls, data: bytes, offset: int) -> 'M2TrackBase':
        """
        Parse the base track structure from binary data.
        
        Args:
            data: Binary data to parse from
            offset: Offset in data to start parsing
            
        Returns:
            M2TrackBase instance
        """
        if offset + 4 > len(data):
            raise ValueError(f"Not enough data for M2TrackBase at offset {offset}")
        
        interpolation_type, global_sequence = struct.unpack('<HH', data[offset:offset + 4])
        return cls(interpolation_type, global_sequence)
    
    def uses_global_sequence(self) -> bool:
        """Check if this track uses a global sequence."""
        return self.global_sequence != 65535


class M2Track:
    """
    Generic M2Track structure for handling animated values.
    Supports version-aware parsing for pre-WotLK vs WotLK+ formats.
    """
    
    def __init__(self):
        """Initialize empty M2Track."""
        self.base = M2TrackBase()
        self.ranges = None  # Pre-264 only
        self.timestamps = {'count': 0, 'offset': 0}
        self.values = {'count': 0, 'offset': 0}
    
    @classmethod
    def parse(cls, data: bytes, offset: int, version: int) -> 'M2Track':
        """
        Parse an M2Track from binary data.
        
        Args:
            data: Binary data to parse from
            offset: Offset in data to start parsing
            version: M2 version to determine parsing format
            
        Returns:
            M2Track instance
        """
        track = cls()
        pos = offset
        
        # Parse base
        track.base = M2TrackBase.parse(data, pos)
        pos += 4
        
        # Version-specific parsing
        if version < 264:
            # Pre-WotLK format: ranges + timestamps + values
            if pos + 24 > len(data):
                raise ValueError(f"Not enough data for pre-264 M2Track at offset {pos}")
            
            ranges_count, ranges_offset = struct.unpack('<II', data[pos:pos + 8])
            pos += 8
            track.ranges = {'count': ranges_count, 'offset': ranges_offset}
            
            timestamps_count, timestamps_offset = struct.unpack('<II', data[pos:pos + 8]) 
            pos += 8
            track.timestamps = {'count': timestamps_count, 'offset': timestamps_offset}
            
            values_count, values_offset = struct.unpack('<II', data[pos:pos + 8])
            pos += 8
            track.values = {'count': values_count, 'offset': values_offset}
        else:
            # WotLK+ format: timestamps + values only
            if pos + 16 > len(data):
                raise ValueError(f"Not enough data for 264+ M2Track at offset {pos}")
            
            timestamps_count, timestamps_offset = struct.unpack('<II', data[pos:pos + 8])
            pos += 8
            track.timestamps = {'count': timestamps_count, 'offset': timestamps_offset}
            
            values_count, values_offset = struct.unpack('<II', data[pos:pos + 8])
            pos += 8
            track.values = {'count': values_count, 'offset': values_offset}
        
        return track
    
    def has_data(self) -> bool:
        """Check if this track has any animation data."""
        return self.timestamps['count'] > 0 and self.values['count'] > 0
    
    def is_static(self) -> bool:
        """Check if this track is static (no animation)."""
        return self.timestamps['count'] <= 1
    
    def uses_global_sequence(self) -> bool:
        """Check if this track uses a global sequence."""
        return self.base.uses_global_sequence()


# Type aliases for common M2Track usage patterns
M2TrackQuat = M2Track  # Track containing M2CompQuat values
M2TrackVec3 = M2Track  # Track containing Vec3 values
M2TrackFloat = M2Track  # Track containing float values