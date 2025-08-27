pub mod animation;
pub mod attachment;
pub mod bone;
pub mod camera;
pub mod color_animation;
pub mod event;
pub mod file_id;
pub mod light;
pub mod material;
pub mod misc;
pub mod particle_emitter;
pub mod physics;
pub mod ribbon_emitter;
pub mod texture;
pub mod texture_animation;
pub mod texture_transform;
pub mod transparency_animation;
pub mod vertex;

// Re-export common types
pub use attachment::{M2Attachment, M2AttachmentId};
pub use camera::{M2Camera, M2CameraFlags};
pub use color_animation::M2ColorAnimation;
pub use event::{M2Event, M2EventIdentifier};
pub use light::{M2Light, M2LightType};
pub use particle_emitter::{M2ParticleEmitter, M2ParticleEmitterType, M2ParticleFlags};
// pub use physics::{
//     M2PhysicsData, M2PhysicsFlags, M2PhysicsJoint, M2PhysicsShape, M2PhysicsShapeType,
// };
pub use ribbon_emitter::M2RibbonEmitter;
pub use texture::{M2Texture, M2TextureFlags, M2TextureType};
pub use texture_animation::{M2TextureAnimation, M2TextureAnimationType};
pub use texture_transform::{M2TextureTransform, M2TextureTransformType};
pub use transparency_animation::M2TransparencyAnimation;
pub use vertex::M2Vertex;

/// Module for converting chunks between different versions
pub mod converter {
    use crate::error::Result;
    use crate::version::MD20Version;

    /// Convert a chunk from one version to another
    pub trait VersionConverter {
        /// Convert self to a specified target version
        fn convert_to_version(&self, target_version: MD20Version) -> Result<Self>
        where
            Self: Sized;
    }
}
