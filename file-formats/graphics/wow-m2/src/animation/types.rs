//! Common types for M2 animation system

use crate::common::C3Vector;

/// Quaternion representation for rotations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    /// Identity quaternion (no rotation)
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// Create a new quaternion
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Normalize the quaternion
    pub fn normalize(&self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            Self::IDENTITY
        }
    }

    /// Spherical linear interpolation
    pub fn slerp(&self, other: &Self, t: f32) -> Self {
        // Compute dot product
        let dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;

        // If dot < 0, negate one quaternion to take shorter arc
        let (other, dot) = if dot < 0.0 {
            (
                Self {
                    x: -other.x,
                    y: -other.y,
                    z: -other.z,
                    w: -other.w,
                },
                -dot,
            )
        } else {
            (*other, dot)
        };

        // If quaternions are very close, use linear interpolation
        if dot > 0.9995 {
            return Self {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            }
            .normalize();
        }

        // Compute spherical interpolation
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        Self {
            x: s0 * self.x + s1 * other.x,
            y: s0 * self.y + s1 * other.y,
            z: s0 * self.z + s1 * other.z,
            w: s0 * self.w + s1 * other.w,
        }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// 3D vector for positions and scales
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Zero vector
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Unit scale vector
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    /// Create a new vector
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<C3Vector> for Vec3 {
    fn from(v: C3Vector) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3> for C3Vector {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

/// Trait for types that can be linearly interpolated
pub trait Lerp: Clone {
    /// Linear interpolation between self and other
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl Lerp for Vec3 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.lerp(&other.x, t),
            y: self.y.lerp(&other.y, t),
            z: self.z.lerp(&other.z, t),
        }
    }
}

impl Lerp for C3Vector {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x.lerp(&other.x, t),
            y: self.y.lerp(&other.y, t),
            z: self.z.lerp(&other.z, t),
        }
    }
}

impl Lerp for Quat {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        // Use slerp for quaternion interpolation
        self.slerp(other, t)
    }
}

/// Fixed-point 16-bit integer scaled to [0.0, 1.0] range
/// Used for texture weights and alpha values
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fixedi16(pub i16);

impl Fixedi16 {
    /// Convert to float in [0.0, 1.0] range
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / 32767.0
    }
}

impl From<Fixedi16> for f32 {
    fn from(v: Fixedi16) -> Self {
        v.to_f32()
    }
}

impl From<f32> for Fixedi16 {
    fn from(v: f32) -> Self {
        Self((v.clamp(0.0, 1.0) * 32767.0) as i16)
    }
}

impl Lerp for Fixedi16 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let a = self.to_f32();
        let b = other.to_f32();
        Fixedi16::from(a.lerp(&b, t))
    }
}

/// Resolved animation track data with keyframes loaded into memory
#[derive(Debug, Clone)]
pub struct ResolvedTrack<T> {
    /// Interpolation type (0=None, 1=Linear, 2=Bezier, 3=Hermite)
    pub interpolation_type: u16,
    /// Global sequence index (65535 = no global sequence)
    pub global_sequence: i16,
    /// Timestamps per animation sequence
    pub timestamps: Vec<Vec<u32>>,
    /// Values per animation sequence
    pub values: Vec<Vec<T>>,
}

impl<T> ResolvedTrack<T> {
    /// Create an empty track
    pub fn empty() -> Self {
        Self {
            interpolation_type: 0,
            global_sequence: -1,
            timestamps: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Check if track has animation data
    pub fn has_data(&self) -> bool {
        !self.timestamps.is_empty() && self.timestamps.iter().any(|ts| !ts.is_empty())
    }

    /// Check if track uses a global sequence
    pub fn uses_global_sequence(&self) -> bool {
        self.global_sequence >= 0
    }
}

impl<T: Default> Default for ResolvedTrack<T> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_lerp() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);

        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < 0.001);
        assert!((mid.y - 10.0).abs() < 0.001);
        assert!((mid.z - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_quat_identity() {
        let q = Quat::IDENTITY;
        assert_eq!(q.x, 0.0);
        assert_eq!(q.y, 0.0);
        assert_eq!(q.z, 0.0);
        assert_eq!(q.w, 1.0);
    }

    #[test]
    fn test_quat_normalize() {
        let q = Quat::new(1.0, 1.0, 1.0, 1.0);
        let normalized = q.normalize();
        let len = (normalized.x.powi(2)
            + normalized.y.powi(2)
            + normalized.z.powi(2)
            + normalized.w.powi(2))
        .sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_fixedi16_conversion() {
        let f = Fixedi16::from(0.5);
        let back = f.to_f32();
        assert!((back - 0.5).abs() < 0.001);
    }
}
