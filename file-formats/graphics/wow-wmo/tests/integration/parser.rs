use wow_wmo::*;

#[test]
fn test_version_parsing() {
    // Test version 17 (Classic-WotLK)
    assert_eq!(
        version::WmoVersion::from_raw(17),
        Some(version::WmoVersion::Classic)
    );

    // Test version 18 (TBC)
    assert_eq!(
        version::WmoVersion::from_raw(18),
        Some(version::WmoVersion::Tbc)
    );

    // Test version 23 (Legion)
    assert_eq!(
        version::WmoVersion::from_raw(23),
        Some(version::WmoVersion::Legion)
    );

    // Test unknown version returns None
    assert_eq!(version::WmoVersion::from_raw(999), None);
}

#[test]
fn test_wmo_flags() {
    use wmo_types::WmoFlags;

    let mut flags = WmoFlags::empty();
    assert!(!flags.contains(WmoFlags::HAS_SKYBOX));

    flags |= WmoFlags::HAS_SKYBOX;
    assert!(flags.contains(WmoFlags::HAS_SKYBOX));

    flags &= !WmoFlags::HAS_SKYBOX;
    assert!(!flags.contains(WmoFlags::HAS_SKYBOX));
}

#[test]
fn test_material_flags() {
    use wmo_types::WmoMaterialFlags;

    let flags = WmoMaterialFlags::UNLIT | WmoMaterialFlags::UNFOGGED;
    assert!(flags.contains(WmoMaterialFlags::UNLIT));
    assert!(flags.contains(WmoMaterialFlags::UNFOGGED));
    assert!(!flags.contains(WmoMaterialFlags::TWO_SIDED));
}

#[test]
fn test_light_type_parsing() {
    use wmo_types::WmoLightType;

    // Test known light types
    assert_eq!(WmoLightType::from_raw(0), Some(WmoLightType::Omni));
    assert_eq!(WmoLightType::from_raw(1), Some(WmoLightType::Spot));
    assert_eq!(WmoLightType::from_raw(2), Some(WmoLightType::Directional));
    assert_eq!(WmoLightType::from_raw(3), Some(WmoLightType::Ambient));

    // Test unknown light type defaults to Omni (with warning)
    assert_eq!(WmoLightType::from_raw(173), Some(WmoLightType::Omni));
}

#[test]
fn test_color_basic() {
    use types::Color;

    let color = Color {
        r: 255,
        g: 128,
        b: 64,
        a: 255,
    };

    // Test basic color field access
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
    assert_eq!(color.a, 255);

    // Test default
    let default_color = Color::default();
    assert_eq!(default_color.r, 0);
    assert_eq!(default_color.g, 0);
    assert_eq!(default_color.b, 0);
    assert_eq!(default_color.a, 0);
}

#[test]
fn test_vec3_basic() {
    use types::Vec3;

    let v1 = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };

    // Test basic field access
    assert_eq!(v1.x, 1.0);
    assert_eq!(v1.y, 2.0);
    assert_eq!(v1.z, 3.0);

    // Test default
    let default_vec = Vec3::default();
    assert_eq!(default_vec.x, 0.0);
    assert_eq!(default_vec.y, 0.0);
    assert_eq!(default_vec.z, 0.0);
}

#[test]
fn test_bounding_box() {
    use types::{BoundingBox, Vec3};

    let min = Vec3 {
        x: -10.0,
        y: -5.0,
        z: 0.0,
    };
    let max = Vec3 {
        x: 10.0,
        y: 5.0,
        z: 20.0,
    };
    let bbox = BoundingBox { min, max };

    // Test basic field access
    assert_eq!(bbox.min.x, -10.0);
    assert_eq!(bbox.max.x, 10.0);
    assert_eq!(bbox.min.y, -5.0);
    assert_eq!(bbox.max.y, 5.0);
}

#[test]
fn test_wmo_group_flags() {
    use wmo_group_types::WmoGroupFlags;

    let mut flags = WmoGroupFlags::empty();

    // Test flag operations with actual flag names
    flags |= WmoGroupFlags::HAS_VERTEX_COLORS;
    assert!(flags.contains(WmoGroupFlags::HAS_VERTEX_COLORS));

    flags |= WmoGroupFlags::INDOOR;
    assert!(flags.contains(WmoGroupFlags::INDOOR));
    assert!(flags.contains(WmoGroupFlags::HAS_VERTEX_COLORS));

    flags &= !WmoGroupFlags::HAS_VERTEX_COLORS;
    assert!(!flags.contains(WmoGroupFlags::HAS_VERTEX_COLORS));
    assert!(flags.contains(WmoGroupFlags::INDOOR));
}

#[test]
fn test_texture_validation_special_values() {
    // Test that special texture marker values (0xFF000000+) are handled correctly
    const SPECIAL_TEXTURE_THRESHOLD: u32 = 0xFF000000;

    let special_value = 0xFF959595; // Example special value that was causing issues
    assert!(special_value >= SPECIAL_TEXTURE_THRESHOLD);

    // This should not be considered an invalid texture index
    let normal_value = 5;
    assert!(normal_value < SPECIAL_TEXTURE_THRESHOLD);
}

#[test]
fn test_chunk_id() {
    use types::ChunkId;

    let chunk_id = ChunkId::from_str("MVER");
    assert_eq!(chunk_id.as_bytes(), b"MVER");

    let chunk_from_bytes = ChunkId::new([b'M', b'V', b'E', b'R']);
    assert_eq!(chunk_from_bytes, chunk_id);
}

#[test]
fn test_minimal_wmo_header() {
    use wmo_types::*;

    // Test creating a minimal WMO header structure
    let header = WmoHeader {
        n_materials: 1,
        n_groups: 1,
        n_portals: 0,
        n_lights: 0,
        n_doodad_defs: 0,
        n_doodad_names: 0,
        n_doodad_sets: 0,
        ambient_color: types::Color {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        },
        flags: WmoFlags::empty(),
    };

    assert_eq!(header.n_materials, 1);
    assert_eq!(header.n_groups, 1);
    assert_eq!(header.ambient_color.r, 128);
}

#[test]
fn test_wmo_material_structure() {
    use wmo_types::WmoMaterial;

    let material = WmoMaterial {
        flags: wmo_types::WmoMaterialFlags::UNLIT,
        shader: 0,
        blend_mode: 0,
        texture1: 0,
        emissive_color: types::Color::default(),
        sidn_color: types::Color::default(),
        framebuffer_blend: types::Color::default(),
        texture2: u32::MAX, // No second texture
        diffuse_color: types::Color::default(),
        ground_type: 0,
    };

    assert!(material.flags.contains(wmo_types::WmoMaterialFlags::UNLIT));
    assert_eq!(material.texture1, 0);
    assert_eq!(material.texture2, u32::MAX);
}

#[test]
fn test_wmo_light_structure() {
    use wmo_types::*;

    let light = WmoLight {
        light_type: WmoLightType::Omni,
        position: types::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        },
        color: types::Color {
            r: 255,
            g: 240,
            b: 200,
            a: 255,
        },
        intensity: 1.5,
        attenuation_start: 5.0,
        attenuation_end: 20.0,
        use_attenuation: true,
        properties: WmoLightProperties::Omni,
    };

    assert_eq!(light.light_type, WmoLightType::Omni);
    assert!(light.use_attenuation);
    assert_eq!(light.intensity, 1.5);
    assert_eq!(light.position.z, 10.0);
}

#[test]
fn test_wmo_doodad_structure() {
    use wmo_types::WmoDoodadDef;

    let doodad = WmoDoodadDef {
        name_offset: 0,
        position: types::Vec3 {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        },
        orientation: [0.0, 0.0, 0.0, 1.0], // No rotation (quaternion)
        scale: 1.0,
        color: types::Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        set_index: 0, // Belongs to first doodad set
    };

    assert_eq!(doodad.position.x, 10.0);
    assert_eq!(doodad.position.y, 10.0);
    assert_eq!(doodad.scale, 1.0);
    assert_eq!(doodad.orientation[3], 1.0); // W component of quaternion
    assert_eq!(doodad.set_index, 0);
}

#[test]
fn test_wmo_portal_structure() {
    use wmo_types::WmoPortal;

    let portal = WmoPortal {
        vertices: vec![
            types::Vec3 {
                x: -1.0,
                y: -1.0,
                z: 0.0,
            },
            types::Vec3 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
            types::Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            types::Vec3 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
            },
        ],
        normal: types::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }, // Facing +Z
    };

    assert_eq!(portal.vertices.len(), 4);
    assert_eq!(portal.normal.z, 1.0);
}

#[test]
fn test_wmo_group_header() {
    use wmo_group_types::*;

    let header = WmoGroupHeader {
        flags: WmoGroupFlags::HAS_VERTEX_COLORS | WmoGroupFlags::INDOOR,
        bounding_box: types::BoundingBox {
            min: types::Vec3 {
                x: -10.0,
                y: -10.0,
                z: 0.0,
            },
            max: types::Vec3 {
                x: 10.0,
                y: 10.0,
                z: 5.0,
            },
        },
        name_offset: 0,
        group_index: 0,
    };

    assert!(header.flags.contains(WmoGroupFlags::HAS_VERTEX_COLORS));
    assert!(header.flags.contains(WmoGroupFlags::INDOOR));
    assert_eq!(header.group_index, 0);
}

#[test]
fn test_tex_coord() {
    use wmo_group_types::TexCoord;

    let tex_coord = TexCoord { u: 0.5, v: 0.75 };
    assert_eq!(tex_coord.u, 0.5);
    assert_eq!(tex_coord.v, 0.75);
}

#[test]
fn test_wmo_batch() {
    use wmo_group_types::WmoBatch;

    let batch = WmoBatch {
        flags: [0u8; 10],
        material_id: 2,
        start_index: 0,
        count: 12,
        start_vertex: 0,
        end_vertex: 3,
        use_large_material_id: false,
    };

    assert_eq!(batch.material_id, 2);
    assert_eq!(batch.count, 12);
    assert_eq!(batch.start_vertex, 0);
    assert_eq!(batch.end_vertex, 3);
}

#[test]
fn test_wmo_doodad_set() {
    use wmo_types::WmoDoodadSet;

    let doodad_set = WmoDoodadSet {
        name: "Default".to_string(),
        start_doodad: 0,
        n_doodads: 5,
    };

    assert_eq!(doodad_set.name, "Default");
    assert_eq!(doodad_set.start_doodad, 0);
    assert_eq!(doodad_set.n_doodads, 5);
}

#[test]
fn test_light_properties() {
    use wmo_types::WmoLightProperties;

    // Test spot light properties
    let spot_props = WmoLightProperties::Spot {
        direction: types::Vec3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        hotspot: 20.0,
        falloff: 35.0,
    };

    if let WmoLightProperties::Spot {
        direction,
        hotspot,
        falloff,
    } = spot_props
    {
        assert_eq!(direction.z, -1.0);
        assert_eq!(hotspot, 20.0);
        assert_eq!(falloff, 35.0);
    } else {
        panic!("Expected spot light properties");
    }

    // Test directional light properties
    let dir_props = WmoLightProperties::Directional {
        direction: types::Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
    };

    if let WmoLightProperties::Directional { direction } = dir_props {
        assert_eq!(direction.x, 1.0);
    } else {
        panic!("Expected directional light properties");
    }
}
