use crate::io_ext::{ReadExt, WriteExt};
use std::io::{Read, Write};

use crate::common::C3Vector;
use crate::error::Result;
use crate::version::M2Version;

/// Physics simulation shape types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2PhysicsShapeType {
    /// No shape (physics disabled)
    None = 0,
    /// Sphere collision
    Sphere = 1,
    /// Capsule (cylinder with rounded ends)
    Capsule = 2,
    /// Plane (infinite flat surface)
    Plane = 3,
    /// Box (cuboid)
    Box = 4,
}

impl M2PhysicsShapeType {
    /// Parse from integer value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Sphere),
            2 => Some(Self::Capsule),
            3 => Some(Self::Plane),
            4 => Some(Self::Box),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    /// Physics flags as defined in the M2 format
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct M2PhysicsFlags: u16 {
        /// Use general collision
        const GENERAL_COLLISION = 0x1;
        /// Is this element a collision trigger
        const COLLISION_TRIGGER = 0x2;
        /// Unknown (added in WoD)
        const UNKNOWN_WOD_1 = 0x4;
        /// Unknown (added in WoD)
        const UNKNOWN_WOD_2 = 0x8;
        /// Unknown (added in WoD)
        const UNKNOWN_WOD_3 = 0x10;
        /// Causes vertices to inherit position from physics
        const ANIMATED_BY_PHYSICS = 0x20;
        /// Unknown
        const UNKNOWN_0x40 = 0x40;
        /// Has precise collision geometry
        const PRECISE_COLLISION = 0x80;
    }
}

/// Represents a physics joint between physics bodies
#[derive(Debug, Clone)]
pub struct M2PhysicsJoint {
    /// First physics body
    pub body1: u32,
    /// Second physics body
    pub body2: u32,
    /// Joint types match Havok/PhysX standard enum
    pub joint_type: u32,
    /// Position of the joint (pivot point)
    pub position: C3Vector,
    /// Orientation of the joint (radians)
    pub orientation: C3Vector,
    /// Lower limits for rotation/movement
    pub lower_limits: C3Vector,
    /// Upper limits for rotation/movement
    pub upper_limits: C3Vector,
    /// Spring coefficients for the 3 axes
    pub spring_coefficients: C3Vector,
    /// Dampening coefficients for the 3 axes
    pub damping_coefficients: C3Vector,
}

impl M2PhysicsJoint {
    /// Parse a physics joint from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let body1 = reader.read_u32_le()?;
        let body2 = reader.read_u32_le()?;
        let joint_type = reader.read_u32_le()?;

        let position = C3Vector::parse(reader)?;
        let orientation = C3Vector::parse(reader)?;
        let lower_limits = C3Vector::parse(reader)?;
        let upper_limits = C3Vector::parse(reader)?;
        let spring_coefficients = C3Vector::parse(reader)?;
        let damping_coefficients = C3Vector::parse(reader)?;

        Ok(Self {
            body1,
            body2,
            joint_type,
            position,
            orientation,
            lower_limits,
            upper_limits,
            spring_coefficients,
            damping_coefficients,
        })
    }

    /// Write a physics joint to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32_le(self.body1)?;
        writer.write_u32_le(self.body2)?;
        writer.write_u32_le(self.joint_type)?;

        self.position.write(writer)?;
        self.orientation.write(writer)?;
        self.lower_limits.write(writer)?;
        self.upper_limits.write(writer)?;
        self.spring_coefficients.write(writer)?;
        self.damping_coefficients.write(writer)?;

        Ok(())
    }
}

/// Represents a physics collision element
#[derive(Debug, Clone)]
pub struct M2PhysicsShape {
    /// Shape type
    pub shape_type: M2PhysicsShapeType,
    /// Index of the bone this shape is attached to
    pub bone_index: u16,
    /// Physics flags
    pub flags: M2PhysicsFlags,
    /// Position relative to the bone
    pub position: C3Vector,
    /// Orientation (radians)
    pub orientation: C3Vector,
    /// Size parameters (interpretation depends on shape type)
    pub dimensions: [f32; 5],
}

impl M2PhysicsShape {
    /// Parse a physics shape from a reader
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let shape_type_raw = reader.read_u8()?;
        let shape_type =
            M2PhysicsShapeType::from_u8(shape_type_raw).unwrap_or(M2PhysicsShapeType::None);

        reader.read_u8()?; // Skip 1 byte of padding

        let bone_index = reader.read_u16_le()?;
        let flags = M2PhysicsFlags::from_bits_retain(reader.read_u16_le()?);

        reader.read_u16_le()?; // Skip another 2 bytes of padding

        let position = C3Vector::parse(reader)?;
        let orientation = C3Vector::parse(reader)?;

        let mut dimensions = [0.0; 5];
        for item in &mut dimensions {
            *item = reader.read_f32_le()?;
        }

        Ok(Self {
            shape_type,
            bone_index,
            flags,
            position,
            orientation,
            dimensions,
        })
    }

    /// Write a physics shape to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(self.shape_type as u8)?;
        writer.write_u8(0)?; // Write padding

        writer.write_u16_le(self.bone_index)?;
        writer.write_u16_le(self.flags.bits())?;
        writer.write_u16_le(0)?; // Write more padding

        self.position.write(writer)?;
        self.orientation.write(writer)?;

        for &dim in &self.dimensions {
            writer.write_f32_le(dim)?;
        }

        Ok(())
    }

    /// Get the size of this physics shape in bytes
    pub fn size_in_bytes() -> usize {
        2 + // shape_type + padding
        2 + // bone_index
        2 + // flags
        2 + // more padding
        3 * 4 + // position
        3 * 4 + // orientation
        5 * 4 // dimensions
    }
}

/// Represents the physics data section of an M2 model
/// Introduced in Mists of Pandaria (5.x)
#[derive(Debug, Clone)]
pub struct M2PhysicsData {
    /// Physics collision shapes
    pub shapes: Vec<M2PhysicsShape>,
    /// Physics bodies
    pub bodies: Vec<u32>,
    /// Physics joints
    pub joints: Vec<M2PhysicsJoint>,
}

impl M2PhysicsData {
    /// Parse physics data from a reader based on the M2 version
    pub fn parse<R: Read + std::io::Seek>(reader: &mut R, version: u32) -> Result<Self> {
        // Physics data was introduced in MoP (5.x)
        if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version < M2Version::MoP {
                return Ok(Self {
                    shapes: Vec::new(),
                    bodies: Vec::new(),
                    joints: Vec::new(),
                });
            }
        }

        let shapes_count = reader.read_u32_le()?;
        let shapes_offset = reader.read_u32_le()?;

        let bodies_count = reader.read_u32_le()?;
        let bodies_offset = reader.read_u32_le()?;

        let joints_count = reader.read_u32_le()?;
        let joints_offset = reader.read_u32_le()?;

        // Parse physics shapes
        let mut shapes = Vec::with_capacity(shapes_count as usize);
        if shapes_count > 0 {
            reader.seek(std::io::SeekFrom::Start(shapes_offset as u64))?;
            for _ in 0..shapes_count {
                shapes.push(M2PhysicsShape::parse(reader)?);
            }
        }

        // Parse physics bodies
        let mut bodies = Vec::with_capacity(bodies_count as usize);
        if bodies_count > 0 {
            reader.seek(std::io::SeekFrom::Start(bodies_offset as u64))?;
            for _ in 0..bodies_count {
                bodies.push(reader.read_u32_le()?);
            }
        }

        // Parse physics joints
        let mut joints = Vec::with_capacity(joints_count as usize);
        if joints_count > 0 {
            reader.seek(std::io::SeekFrom::Start(joints_offset as u64))?;
            for _ in 0..joints_count {
                joints.push(M2PhysicsJoint::parse(reader)?);
            }
        }

        Ok(Self {
            shapes,
            bodies,
            joints,
        })
    }

    /// Write physics data to a writer based on the M2 version
    pub fn write<W: Write + std::io::Seek>(&self, writer: &mut W, version: u32) -> Result<()> {
        // Physics data was introduced in MoP (5.x)
        if let Some(m2_version) = M2Version::from_header_version(version) {
            if m2_version < M2Version::MoP {
                return Ok(());
            }
        }

        let header_pos = writer.stream_position()?;

        // Write placeholders for counts and offsets
        writer.write_u32_le(self.shapes.len() as u32)?;
        writer.write_u32_le(0)?; // shapes_offset placeholder

        writer.write_u32_le(self.bodies.len() as u32)?;
        writer.write_u32_le(0)?; // bodies_offset placeholder

        writer.write_u32_le(self.joints.len() as u32)?;
        writer.write_u32_le(0)?; // joints_offset placeholder

        // Write shapes
        if !self.shapes.is_empty() {
            let shapes_offset = writer.stream_position()?;

            // Update shapes offset in header
            writer.seek(std::io::SeekFrom::Start(header_pos + 4))?;
            writer.write_u32_le(shapes_offset as u32)?;
            writer.seek(std::io::SeekFrom::Start(shapes_offset))?;

            for shape in &self.shapes {
                shape.write(writer)?;
            }
        }

        // Write bodies
        if !self.bodies.is_empty() {
            let bodies_offset = writer.stream_position()?;

            // Update bodies offset in header
            writer.seek(std::io::SeekFrom::Start(header_pos + 12))?;
            writer.write_u32_le(bodies_offset as u32)?;
            writer.seek(std::io::SeekFrom::Start(bodies_offset))?;

            for &body in &self.bodies {
                writer.write_u32_le(body)?;
            }
        }

        // Write joints
        if !self.joints.is_empty() {
            let joints_offset = writer.stream_position()?;

            // Update joints offset in header
            writer.seek(std::io::SeekFrom::Start(header_pos + 20))?;
            writer.write_u32_le(joints_offset as u32)?;
            writer.seek(std::io::SeekFrom::Start(joints_offset))?;

            for joint in &self.joints {
                joint.write(writer)?;
            }
        }

        Ok(())
    }

    /// Convert this physics data to a different version
    pub fn convert(&self, source_version: M2Version, target_version: M2Version) -> Self {
        if target_version < M2Version::MoP {
            // Remove physics data for pre-MoP versions
            Self {
                shapes: Vec::new(),
                bodies: Vec::new(),
                joints: Vec::new(),
            }
        } else if source_version < M2Version::MoP {
            // If we're upgrading to MoP+ from a version without physics,
            // we just return an empty physics data structure
            self.clone()
        } else {
            // No changes needed for MoP+ to MoP+ conversions
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_physics_shape_parse_write() {
        let shape = M2PhysicsShape {
            shape_type: M2PhysicsShapeType::Sphere,
            bone_index: 1,
            flags: M2PhysicsFlags::GENERAL_COLLISION,
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            orientation: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            dimensions: [1.0, 0.0, 0.0, 0.0, 0.0], // Sphere radius is first dimension
        };

        let mut data = Vec::new();
        shape.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_shape = M2PhysicsShape::parse(&mut cursor).unwrap();

        assert_eq!(parsed_shape.shape_type, M2PhysicsShapeType::Sphere);
        assert_eq!(parsed_shape.bone_index, 1);
        assert_eq!(parsed_shape.flags, M2PhysicsFlags::GENERAL_COLLISION);
        assert_eq!(parsed_shape.position.x, 1.0);
        assert_eq!(parsed_shape.position.y, 2.0);
        assert_eq!(parsed_shape.position.z, 3.0);
        assert_eq!(parsed_shape.dimensions[0], 1.0); // Sphere radius
    }

    #[test]
    fn test_physics_joint_parse_write() {
        let joint = M2PhysicsJoint {
            body1: 0,
            body2: 1,
            joint_type: 2, // 2 = hinge joint
            position: C3Vector {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            orientation: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            lower_limits: C3Vector {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            upper_limits: C3Vector {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            spring_coefficients: C3Vector {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            damping_coefficients: C3Vector {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            },
        };

        let mut data = Vec::new();
        joint.write(&mut data).unwrap();

        let mut cursor = Cursor::new(data);
        let parsed_joint = M2PhysicsJoint::parse(&mut cursor).unwrap();

        assert_eq!(parsed_joint.body1, 0);
        assert_eq!(parsed_joint.body2, 1);
        assert_eq!(parsed_joint.joint_type, 2);
        assert_eq!(parsed_joint.position.x, 1.0);
        assert_eq!(parsed_joint.position.y, 2.0);
        assert_eq!(parsed_joint.position.z, 3.0);
        assert_eq!(parsed_joint.lower_limits.x, -1.0);
        assert_eq!(parsed_joint.upper_limits.x, 1.0);
        assert_eq!(parsed_joint.damping_coefficients.x, 0.5);
    }
}
