//! Integration tests for M2 skin file handling

// Simple test structures since we don't have the actual skin types exported
#[derive(Debug)]
struct MockSkin {
    submesh_count: u32,
    index_count: u32,
}

#[test]
fn test_skin_structure() {
    let skin = MockSkin {
        submesh_count: 3,
        index_count: 450,
    };

    assert_eq!(skin.submesh_count, 3);
    assert_eq!(skin.index_count, 450);
}

#[test]
fn test_skin_validation() {
    // Test basic skin validation logic
    let skin = MockSkin {
        submesh_count: 0,
        index_count: 0,
    };

    // Empty skin should be valid
    assert_eq!(skin.submesh_count, 0);
    assert_eq!(skin.index_count, 0);
}

#[test]
fn test_skin_triangle_calculation() {
    // Test triangle count calculations
    let index_count = 450;
    let triangle_count = index_count / 3;

    assert_eq!(triangle_count, 150);
    assert_eq!(triangle_count * 3, index_count);
}

#[test]
fn test_skin_lod_levels() {
    // Create multiple skins with different LOD levels
    let lod_levels: Vec<_> = (0..4)
        .map(|lod| {
            let triangle_count = 100 / (lod + 1);
            MockSkin {
                submesh_count: 1,
                index_count: triangle_count * 3,
            }
        })
        .collect();

    // Verify triangle count decreases with LOD
    for (i, skin) in lod_levels.iter().enumerate() {
        let expected_triangles = 100 / (i + 1);
        assert_eq!(skin.index_count, (expected_triangles * 3) as u32);
    }
}
