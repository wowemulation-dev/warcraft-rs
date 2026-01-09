//! BSP tree point query for WMO groups
//!
//! WMO groups use BSP (Binary Space Partitioning) trees for efficient point containment
//! and collision testing. The BSP tree partitions space using axis-aligned planes,
//! allowing rapid determination of which faces/triangles contain a given point.
//!
//! # Algorithm
//!
//! 1. Start at the root node (index 0)
//! 2. If the node is a leaf, collect it as a candidate
//! 3. If the plane is Z-axis aligned, traverse both children (for height queries)
//! 4. Otherwise, compare the point's coordinate to the plane distance and traverse
//!    the appropriate child
//! 5. After collecting candidate leaves, perform ray-triangle intersection
//!    (shooting a ray in negative Z direction) to find the closest triangle
//!
//! # Reference
//!
//! Based on noclip.website's WMO BSP implementation:
//! <https://github.com/magcius/noclip.website/blob/master/rust/src/wow/wmo.rs>

use crate::types::Vec3;
use crate::wmo_group_types::WmoBspNode;

/// Axis type for BSP plane classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BspAxisType {
    /// Plane normal points along X axis
    X,
    /// Plane normal points along Y axis
    Y,
    /// Plane normal points along Z axis
    Z,
    /// Non-axis-aligned plane (should not occur in WMO BSP)
    Other,
}

/// Extension trait for `WmoBspNode` to add query functionality
pub trait BspNodeExt {
    /// Check if this node is a leaf node
    fn is_leaf(&self) -> bool;

    /// Get the axis type of this node's plane
    fn get_axis_type(&self) -> BspAxisType;

    /// Get the negative child index (-1 means no child)
    fn negative_child(&self) -> i16;

    /// Get the positive child index (-1 means no child)
    fn positive_child(&self) -> i16;
}

impl BspNodeExt for WmoBspNode {
    fn is_leaf(&self) -> bool {
        // A leaf node has face references (num_faces > 0) or both children are -1
        self.num_faces > 0 || (self.children[0] == -1 && self.children[1] == -1)
    }

    fn get_axis_type(&self) -> BspAxisType {
        let nx = self.plane.normal.x.abs();
        let ny = self.plane.normal.y.abs();
        let nz = self.plane.normal.z.abs();

        // The plane is axis-aligned if one component is ~1 and others are ~0
        const EPSILON: f32 = 0.0001;

        if nx > 1.0 - EPSILON && ny < EPSILON && nz < EPSILON {
            BspAxisType::X
        } else if ny > 1.0 - EPSILON && nx < EPSILON && nz < EPSILON {
            BspAxisType::Y
        } else if nz > 1.0 - EPSILON && nx < EPSILON && ny < EPSILON {
            BspAxisType::Z
        } else {
            BspAxisType::Other
        }
    }

    fn negative_child(&self) -> i16 {
        self.children[0]
    }

    fn positive_child(&self) -> i16 {
        self.children[1]
    }
}

/// BSP tree wrapper for efficient spatial queries
#[derive(Debug, Clone)]
pub struct BspTree {
    nodes: Vec<WmoBspNode>,
}

impl BspTree {
    /// Create a new BSP tree from nodes
    pub fn new(nodes: Vec<WmoBspNode>) -> Self {
        Self { nodes }
    }

    /// Create an empty BSP tree
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Get a node by index
    pub fn get_node(&self, index: usize) -> Option<&WmoBspNode> {
        self.nodes.get(index)
    }

    /// Query the BSP tree to find all leaf nodes that might contain the point.
    ///
    /// Returns indices of leaf nodes that need to be checked for triangle intersection.
    pub fn query_point(&self, point: &[f32; 3]) -> Vec<usize> {
        let mut leaves = Vec::new();
        if !self.nodes.is_empty() {
            self.query_recursive(point, 0, &mut leaves);
        }
        leaves
    }

    /// Recursive BSP traversal
    fn query_recursive(&self, point: &[f32; 3], node_index: i16, leaves: &mut Vec<usize>) {
        if node_index < 0 {
            return;
        }

        let idx = node_index as usize;
        if idx >= self.nodes.len() {
            return;
        }

        let node = &self.nodes[idx];

        // If this is a leaf, collect it
        if node.is_leaf() {
            leaves.push(idx);
            return;
        }

        let axis = node.get_axis_type();

        // For Z-axis planes, traverse both children (needed for height-based queries)
        // This is because we want to find all triangles above/below the point
        if axis == BspAxisType::Z {
            self.query_recursive(point, node.negative_child(), leaves);
            self.query_recursive(point, node.positive_child(), leaves);
        } else {
            // For X/Y planes, only traverse the side the point is on
            let component = match axis {
                BspAxisType::X => point[0],
                BspAxisType::Y => point[1],
                _ => {
                    // Non-axis-aligned: use signed distance
                    let dist = node.plane.normal.x * point[0]
                        + node.plane.normal.y * point[1]
                        + node.plane.normal.z * point[2]
                        - node.plane.distance;
                    if dist < 0.0 {
                        self.query_recursive(point, node.negative_child(), leaves);
                    } else {
                        self.query_recursive(point, node.positive_child(), leaves);
                    }
                    return;
                }
            };

            if component < node.plane.distance {
                self.query_recursive(point, node.negative_child(), leaves);
            } else {
                self.query_recursive(point, node.positive_child(), leaves);
            }
        }
    }

    /// Find the closest triangle below the given point using ray-triangle intersection.
    ///
    /// Shoots a ray in the negative Z direction from the point and finds the
    /// first triangle it intersects.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point
    /// * `vertices` - All vertices in the WMO group
    /// * `indices` - Triangle indices (3 per triangle)
    ///
    /// # Returns
    ///
    /// The index of the closest triangle (in terms of triangle count, not face index),
    /// or `None` if no triangle is below the point.
    pub fn pick_closest_tri_neg_z(
        &self,
        point: &[f32; 3],
        vertices: &[Vec3],
        indices: &[u16],
    ) -> Option<usize> {
        let leaf_indices = self.query_point(point);

        let mut closest_t = f32::NEG_INFINITY;
        let mut closest_tri = None;

        for leaf_idx in leaf_indices {
            let node = &self.nodes[leaf_idx];
            if node.num_faces == 0 {
                continue;
            }

            // Check each face in this leaf
            for face_offset in 0..node.num_faces {
                let face_index = node.first_face as usize + face_offset as usize;
                let tri_start = face_index * 3;

                if tri_start + 2 >= indices.len() {
                    continue;
                }

                let i0 = indices[tri_start] as usize;
                let i1 = indices[tri_start + 1] as usize;
                let i2 = indices[tri_start + 2] as usize;

                if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
                    continue;
                }

                let v0 = &vertices[i0];
                let v1 = &vertices[i1];
                let v2 = &vertices[i2];

                // Ray-triangle intersection: ray from point going in -Z direction
                if let Some(t) = ray_triangle_intersect_neg_z(point, v0, v1, v2) {
                    // t should be positive (triangle is below the point)
                    // We want the closest triangle (smallest positive t)
                    if t > 0.0 && t > closest_t {
                        closest_t = t;
                        closest_tri = Some(face_index);
                    }
                }
            }
        }

        closest_tri
    }
}

/// Ray-triangle intersection for a ray going in the negative Z direction.
///
/// Uses the Möller–Trumbore algorithm optimized for -Z direction rays.
///
/// # Returns
///
/// The t parameter (distance in Z) if intersection exists, `None` otherwise.
fn ray_triangle_intersect_neg_z(origin: &[f32; 3], v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Option<f32> {
    // Ray direction is (0, 0, -1)
    let edge1 = [v1.x - v0.x, v1.y - v0.y, v1.z - v0.z];
    let edge2 = [v2.x - v0.x, v2.y - v0.y, v2.z - v0.z];

    // h = dir × edge2 = (0,0,-1) × edge2
    let h = [edge2[1], -edge2[0], 0.0];

    let a = edge1[0] * h[0] + edge1[1] * h[1] + edge1[2] * h[2];

    const EPSILON: f32 = 0.000001;
    if a > -EPSILON && a < EPSILON {
        return None; // Ray is parallel to triangle
    }

    let f = 1.0 / a;
    let s = [origin[0] - v0.x, origin[1] - v0.y, origin[2] - v0.z];

    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    // q = s × edge1
    let q = [
        s[1] * edge1[2] - s[2] * edge1[1],
        s[2] * edge1[0] - s[0] * edge1[2],
        s[0] * edge1[1] - s[1] * edge1[0],
    ];

    // v = f * dot(dir, q) = f * (0*q[0] + 0*q[1] + (-1)*q[2])
    let v = f * (-q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // t = f * dot(edge2, q)
    let t = f * (edge2[0] * q[0] + edge2[1] * q[1] + edge2[2] * q[2]);

    if t > EPSILON { Some(t) } else { None }
}

/// Check if a point is inside a WMO group using BSP tree and ray-casting
pub fn point_in_group(point: &[f32; 3], bsp: &BspTree, vertices: &[Vec3], indices: &[u16]) -> bool {
    bsp.pick_closest_tri_neg_z(point, vertices, indices)
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wmo_group_types::WmoPlane;

    fn make_plane(axis: BspAxisType, dist: f32) -> WmoPlane {
        let normal = match axis {
            BspAxisType::X => Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            BspAxisType::Y => Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            BspAxisType::Z => Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            BspAxisType::Other => Vec3 {
                x: 0.577,
                y: 0.577,
                z: 0.577,
            },
        };
        WmoPlane {
            normal,
            distance: dist,
        }
    }

    #[test]
    fn test_bsp_node_is_leaf() {
        let leaf = WmoBspNode {
            plane: make_plane(BspAxisType::X, 0.0),
            children: [-1, -1],
            first_face: 0,
            num_faces: 2,
        };
        assert!(leaf.is_leaf());

        let internal = WmoBspNode {
            plane: make_plane(BspAxisType::X, 0.0),
            children: [1, 2],
            first_face: 0,
            num_faces: 0,
        };
        assert!(!internal.is_leaf());
    }

    #[test]
    fn test_bsp_axis_type() {
        let x_node = WmoBspNode {
            plane: make_plane(BspAxisType::X, 5.0),
            children: [-1, -1],
            first_face: 0,
            num_faces: 0,
        };
        assert_eq!(x_node.get_axis_type(), BspAxisType::X);

        let y_node = WmoBspNode {
            plane: make_plane(BspAxisType::Y, 5.0),
            children: [-1, -1],
            first_face: 0,
            num_faces: 0,
        };
        assert_eq!(y_node.get_axis_type(), BspAxisType::Y);

        let z_node = WmoBspNode {
            plane: make_plane(BspAxisType::Z, 5.0),
            children: [-1, -1],
            first_face: 0,
            num_faces: 0,
        };
        assert_eq!(z_node.get_axis_type(), BspAxisType::Z);
    }

    #[test]
    fn test_empty_bsp() {
        let bsp = BspTree::empty();
        assert!(bsp.is_empty());
        assert_eq!(bsp.query_point(&[0.0, 0.0, 0.0]), Vec::<usize>::new());
    }

    #[test]
    fn test_single_leaf_bsp() {
        let nodes = vec![WmoBspNode {
            plane: make_plane(BspAxisType::X, 0.0),
            children: [-1, -1],
            first_face: 0,
            num_faces: 1,
        }];
        let bsp = BspTree::new(nodes);

        let leaves = bsp.query_point(&[0.0, 0.0, 0.0]);
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0], 0);
    }

    #[test]
    fn test_simple_x_split() {
        // Root splits at x=0, left child at index 1, right child at index 2
        let nodes = vec![
            WmoBspNode {
                plane: make_plane(BspAxisType::X, 0.0),
                children: [1, 2],
                first_face: 0,
                num_faces: 0,
            },
            WmoBspNode {
                plane: make_plane(BspAxisType::X, 0.0),
                children: [-1, -1],
                first_face: 0,
                num_faces: 1,
            },
            WmoBspNode {
                plane: make_plane(BspAxisType::X, 0.0),
                children: [-1, -1],
                first_face: 1,
                num_faces: 1,
            },
        ];
        let bsp = BspTree::new(nodes);

        // Point with x < 0 should go to negative child (index 1)
        let leaves_neg = bsp.query_point(&[-5.0, 0.0, 0.0]);
        assert_eq!(leaves_neg, vec![1]);

        // Point with x > 0 should go to positive child (index 2)
        let leaves_pos = bsp.query_point(&[5.0, 0.0, 0.0]);
        assert_eq!(leaves_pos, vec![2]);
    }

    #[test]
    fn test_z_split_traverses_both() {
        // Z-axis split should traverse both children
        let nodes = vec![
            WmoBspNode {
                plane: make_plane(BspAxisType::Z, 0.0),
                children: [1, 2],
                first_face: 0,
                num_faces: 0,
            },
            WmoBspNode {
                plane: make_plane(BspAxisType::X, 0.0),
                children: [-1, -1],
                first_face: 0,
                num_faces: 1,
            },
            WmoBspNode {
                plane: make_plane(BspAxisType::X, 0.0),
                children: [-1, -1],
                first_face: 1,
                num_faces: 1,
            },
        ];
        let bsp = BspTree::new(nodes);

        // Should return both children for Z-split
        let mut leaves = bsp.query_point(&[0.0, 0.0, 5.0]);
        leaves.sort();
        assert_eq!(leaves, vec![1, 2]);
    }

    #[test]
    fn test_ray_triangle_intersect() {
        // Triangle in XY plane at z=0
        let v0 = Vec3 {
            x: -1.0,
            y: -1.0,
            z: 0.0,
        };
        let v1 = Vec3 {
            x: 1.0,
            y: -1.0,
            z: 0.0,
        };
        let v2 = Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };

        // Point above center of triangle
        let hit = ray_triangle_intersect_neg_z(&[0.0, 0.0, 5.0], &v0, &v1, &v2);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 5.0).abs() < 0.001);

        // Point outside triangle bounds
        let miss = ray_triangle_intersect_neg_z(&[10.0, 10.0, 5.0], &v0, &v1, &v2);
        assert!(miss.is_none());

        // Point below triangle (should not hit with -Z ray)
        let below = ray_triangle_intersect_neg_z(&[0.0, 0.0, -5.0], &v0, &v1, &v2);
        assert!(below.is_none());
    }
}
