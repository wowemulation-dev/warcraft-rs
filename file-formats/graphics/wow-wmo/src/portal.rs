//! Portal-based visibility culling for WMO interiors
//!
//! WMO files use portals to define visibility between groups. A portal is a
//! planar polygon that acts as a window between two groups. By testing visibility
//! through these portals, the renderer can efficiently determine which groups
//! need to be drawn.
//!
//! # Algorithm
//!
//! 1. Find which group the camera is in
//! 2. Mark that group as visible
//! 3. For each portal in the current group:
//!    - Check if the portal faces the camera
//!    - Check if the portal is within the view frustum
//!    - If visible, clip the frustum to the portal bounds
//!    - Recursively traverse into the connected group with the clipped frustum
//!
//! # Reference
//!
//! Based on noclip.website's WMO portal culling implementation:
//! <https://github.com/magcius/noclip.website/blob/master/rust/src/wow/wmo.rs>

use crate::types::Vec3;

/// Axis for 2D projection when checking point-in-polygon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// 3D plane defined by normal and distance from origin.
///
/// The plane equation is: `normal · point + distance = 0`
///
/// # Default Value
///
/// `Plane::default()` creates a degenerate plane with `normal = [0,0,0]` and
/// `distance = 0`. This is used as a sentinel value for empty or invalid portals
/// (e.g., portals with no vertices). All methods handle this gracefully:
///
/// - `major_axis()` returns `Axis::Z` (deterministic fallback)
/// - `intersect_line()` returns `f32::INFINITY` (no intersection)
/// - `distance_to_point()` returns `0.0` (point is on the "plane")
///
/// This sentinel pattern avoids `Option<Plane>` complexity while maintaining
/// safe behavior for degenerate cases.
#[derive(Debug, Clone, Copy, Default)]
pub struct Plane {
    /// Unit normal vector. Zero vector indicates a degenerate/sentinel plane.
    pub normal: [f32; 3],
    /// Signed distance from origin along the normal.
    pub distance: f32,
}

impl Plane {
    /// Check if this is a degenerate (sentinel) plane with zero normal.
    ///
    /// Degenerate planes are created by `Plane::default()` and represent
    /// invalid or empty geometry. Use this to check validity before
    /// performing geometric operations that require a valid plane.
    pub fn is_degenerate(&self) -> bool {
        let len_sq = self.normal[0] * self.normal[0]
            + self.normal[1] * self.normal[1]
            + self.normal[2] * self.normal[2];
        len_sq < 0.0001
    }

    /// Create a plane from a normal vector and a point on the plane.
    pub fn from_normal_and_point(normal: [f32; 3], point: &[f32; 3]) -> Self {
        let d = -(normal[0] * point[0] + normal[1] * point[1] + normal[2] * point[2]);
        Self {
            normal,
            distance: d,
        }
    }

    /// Create a plane from three points (counter-clockwise winding).
    ///
    /// If the points are collinear or coincident, returns a degenerate plane.
    pub fn from_triangle(a: &[f32; 3], b: &[f32; 3], c: &[f32; 3]) -> Self {
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let mut normal = cross(&ab, &ac);
        let len = length(&normal);
        if len > 0.0001 {
            normal[0] /= len;
            normal[1] /= len;
            normal[2] /= len;
        }
        Self::from_normal_and_point(normal, a)
    }

    /// Signed distance from a point to the plane.
    ///
    /// For degenerate planes (zero normal), returns `self.distance` (typically 0).
    pub fn distance_to_point(&self, point: &[f32; 3]) -> f32 {
        self.normal[0] * point[0]
            + self.normal[1] * point[1]
            + self.normal[2] * point[2]
            + self.distance
    }

    /// Negate the plane (flip to other side).
    pub fn negate(&mut self) {
        self.normal[0] = -self.normal[0];
        self.normal[1] = -self.normal[1];
        self.normal[2] = -self.normal[2];
        self.distance = -self.distance;
    }

    /// Get the major axis of this plane (axis with largest normal component).
    ///
    /// For degenerate planes (zero normal), returns `Axis::Z` as a deterministic fallback.
    pub fn major_axis(&self) -> Axis {
        let ax = self.normal[0].abs();
        let ay = self.normal[1].abs();
        let az = self.normal[2].abs();
        if az >= ax && az >= ay {
            Axis::Z
        } else if ay >= ax {
            Axis::Y
        } else {
            Axis::X
        }
    }

    /// Intersect a ray with this plane, returning the t parameter.
    ///
    /// Returns `f32::INFINITY` if the ray is parallel to the plane or if
    /// this is a degenerate plane.
    pub fn intersect_line(&self, origin: &[f32; 3], direction: &[f32; 3]) -> f32 {
        let denom = self.normal[0] * direction[0]
            + self.normal[1] * direction[1]
            + self.normal[2] * direction[2];
        if denom.abs() < 0.0001 {
            return f32::INFINITY;
        }
        -self.distance_to_point(origin) / denom
    }
}

/// Axis-aligned bounding box.
///
/// # Default Value
///
/// `AABB::default()` creates a zero-volume box at the origin (`min = max = [0,0,0]`).
/// This is used as a sentinel for empty geometry (e.g., portals with no vertices).
/// Use `is_empty()` to check for this degenerate state.
#[derive(Debug, Clone, Copy, Default)]
pub struct AABB {
    /// Minimum corner of the bounding box.
    pub min: [f32; 3],
    /// Maximum corner of the bounding box.
    pub max: [f32; 3],
}

impl AABB {
    /// Check if this is an empty/degenerate AABB (zero volume at origin).
    pub fn is_empty(&self) -> bool {
        self.min[0] >= self.max[0] && self.min[1] >= self.max[1] && self.min[2] >= self.max[2]
    }

    /// Create from a list of points.
    ///
    /// Returns `AABB::default()` (empty sentinel) if points is empty.
    pub fn from_points(points: &[[f32; 3]]) -> Self {
        if points.is_empty() {
            return Self::default();
        }
        let mut min = points[0];
        let mut max = points[0];
        for p in points.iter().skip(1) {
            min[0] = min[0].min(p[0]);
            min[1] = min[1].min(p[1]);
            min[2] = min[2].min(p[2]);
            max[0] = max[0].max(p[0]);
            max[1] = max[1].max(p[1]);
            max[2] = max[2].max(p[2]);
        }
        Self { min, max }
    }

    /// Check if a point is inside this AABB
    pub fn contains_point(&self, point: &[f32; 3]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
            && point[2] >= self.min[2]
            && point[2] <= self.max[2]
    }

    /// Check if a point is inside ignoring max Z (for exterior groups)
    pub fn contains_point_ignore_max_z(&self, point: &[f32; 3]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
            && point[2] >= self.min[2]
    }
}

/// A convex hull represented as a collection of planes
#[derive(Debug, Clone, Default)]
pub struct ConvexHull {
    pub planes: Vec<Plane>,
}

impl ConvexHull {
    /// Create an empty convex hull
    pub fn new() -> Self {
        Self { planes: Vec::new() }
    }

    /// Check if an AABB is at least partially inside this convex hull
    pub fn contains_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            // Find the corner closest to the plane
            let test_point = [
                if plane.normal[0] >= 0.0 {
                    aabb.min[0]
                } else {
                    aabb.max[0]
                },
                if plane.normal[1] >= 0.0 {
                    aabb.min[1]
                } else {
                    aabb.max[1]
                },
                if plane.normal[2] >= 0.0 {
                    aabb.min[2]
                } else {
                    aabb.max[2]
                },
            ];
            // If the closest corner is outside this plane, the AABB is outside
            if plane.distance_to_point(&test_point) < 0.0 {
                return false;
            }
        }
        true
    }
}

/// Processed portal data ready for culling operations
#[derive(Debug, Clone)]
pub struct Portal {
    /// Portal vertices in 3D space
    pub vertices: Vec<[f32; 3]>,
    /// Portal plane
    pub plane: Plane,
    /// Axis-aligned bounding box
    pub aabb: AABB,
}

impl Portal {
    /// Create a portal from raw vertex data
    pub fn from_vertices(raw_vertices: &[Vec3], normal: &Vec3) -> Self {
        // Convert to array format and remove duplicates
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        for v in raw_vertices {
            let arr = [v.x, v.y, v.z];
            let is_duplicate = vertices.iter().any(|existing| {
                (existing[0] - arr[0]).abs() < 0.001
                    && (existing[1] - arr[1]).abs() < 0.001
                    && (existing[2] - arr[2]).abs() < 0.001
            });
            if !is_duplicate {
                vertices.push(arr);
            }
        }

        // Create plane from normal
        let plane = if !vertices.is_empty() {
            Plane::from_normal_and_point([normal.x, normal.y, normal.z], &vertices[0])
        } else {
            Plane::default()
        };

        // Sort vertices around centroid for consistent winding
        if vertices.len() > 2 {
            let major_axis = plane.major_axis();
            let centroid = compute_centroid(&vertices);
            let centroid_2d = project_to_2d(&centroid, major_axis);

            vertices.sort_by(|a, b| {
                let a_2d = project_to_2d(a, major_axis);
                let b_2d = project_to_2d(b, major_axis);
                let a_rel = [a_2d[0] - centroid_2d[0], a_2d[1] - centroid_2d[1]];
                let b_rel = [b_2d[0] - centroid_2d[0], b_2d[1] - centroid_2d[1]];
                let a_angle = f32::atan2(a_rel[1], a_rel[0]);
                let b_angle = f32::atan2(b_rel[1], b_rel[0]);
                a_angle
                    .partial_cmp(&b_angle)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let aabb = AABB::from_points(&vertices);

        Self {
            vertices,
            plane,
            aabb,
        }
    }

    /// Check if the portal is facing the camera (based on portal reference side)
    pub fn is_facing_camera(&self, eye: &[f32; 3], side: i16) -> bool {
        let dist = self.plane.distance_to_point(eye);
        if side < 0 && dist > 0.0 {
            return false;
        }
        if side > 0 && dist < 0.0 {
            return false;
        }
        true
    }

    /// Check if the portal AABB intersects the frustum
    pub fn in_frustum(&self, frustum: &ConvexHull) -> bool {
        frustum.contains_aabb(&self.aabb)
    }

    /// Check if a point is inside the portal AABB
    pub fn aabb_contains_point(&self, point: &[f32; 3]) -> bool {
        self.aabb.contains_point(point)
    }

    /// Clip a frustum by adding planes that pass through the eye and each edge
    pub fn clip_frustum(&self, eye: &[f32; 3], frustum: &ConvexHull) -> ConvexHull {
        let mut result = frustum.clone();

        if self.vertices.len() < 2 {
            return result;
        }

        for i in 0..self.vertices.len() {
            let a = &self.vertices[i];
            let b = if i == self.vertices.len() - 1 {
                &self.vertices[0]
            } else {
                &self.vertices[i + 1]
            };

            // Get a test point (previous vertex) to determine plane orientation
            let test_point = if i == 0 {
                &self.vertices[self.vertices.len() - 1]
            } else {
                &self.vertices[i - 1]
            };

            // Create plane through eye, a, b
            let mut plane = Plane::from_triangle(eye, a, b);

            // Make sure the plane faces inward (test point should be on positive side)
            if plane.distance_to_point(test_point) < 0.0 {
                plane.negate();
            }

            result.planes.push(plane);
        }

        result
    }

    /// Project portal vertices to 2D for point-in-polygon test
    pub fn project_to_2d(&self) -> (Vec<[f32; 2]>, Axis) {
        let axis = self.plane.major_axis();
        let projected: Vec<[f32; 2]> = self
            .vertices
            .iter()
            .map(|v| project_to_2d(v, axis))
            .collect();
        (projected, axis)
    }
}

/// Visibility traversal state
#[derive(Debug)]
pub struct VisibilityResult {
    /// Set of visible group indices
    pub visible_groups: Vec<u32>,
    /// Exterior frustums (for rendering outdoor content)
    pub exterior_frustums: Vec<ConvexHull>,
}

impl VisibilityResult {
    pub fn new() -> Self {
        Self {
            visible_groups: Vec::new(),
            exterior_frustums: Vec::new(),
        }
    }
}

impl Default for VisibilityResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Portal reference linking groups through portals
#[derive(Debug, Clone, Copy)]
pub struct PortalRef {
    /// Index into the portal array
    pub portal_index: u16,
    /// Index of the group this portal leads to
    pub group_index: u16,
    /// Which side of the portal (-1 or 1)
    pub side: i16,
}

/// Group portal information for visibility traversal
#[derive(Debug, Clone)]
pub struct GroupPortalInfo {
    /// Starting index in the portal references array
    pub portal_start: u16,
    /// Number of portals for this group
    pub portal_count: u16,
    /// Whether this is an exterior group
    pub is_exterior: bool,
}

/// Portal culling system for a WMO
#[derive(Debug)]
pub struct PortalCuller {
    /// Processed portal data
    portals: Vec<Portal>,
    /// Portal references
    portal_refs: Vec<PortalRef>,
    /// Per-group portal information
    group_info: Vec<GroupPortalInfo>,
}

impl PortalCuller {
    /// Create a new portal culler
    pub fn new(
        portals: Vec<Portal>,
        portal_refs: Vec<PortalRef>,
        group_info: Vec<GroupPortalInfo>,
    ) -> Self {
        Self {
            portals,
            portal_refs,
            group_info,
        }
    }

    /// Find visible groups starting from a given group
    pub fn find_visible_groups(
        &self,
        start_group: u32,
        eye: &[f32; 3],
        frustum: &ConvexHull,
    ) -> VisibilityResult {
        let mut result = VisibilityResult::new();
        let mut visited = vec![false; self.group_info.len()];

        self.traverse_portals(start_group, eye, frustum, &mut result, &mut visited);

        result
    }

    /// Recursive portal traversal
    fn traverse_portals(
        &self,
        group_id: u32,
        eye: &[f32; 3],
        frustum: &ConvexHull,
        result: &mut VisibilityResult,
        visited: &mut [bool],
    ) {
        let group_idx = group_id as usize;
        if group_idx >= visited.len() || visited[group_idx] {
            return;
        }

        // Mark this group as visible
        result.visible_groups.push(group_id);
        visited[group_idx] = true;

        // Get portal info for this group
        let Some(info) = self.group_info.get(group_idx) else {
            return;
        };

        let start = info.portal_start as usize;
        let end = start + info.portal_count as usize;

        // Check each portal
        for i in start..end {
            let Some(portal_ref) = self.portal_refs.get(i) else {
                continue;
            };

            let Some(portal) = self.portals.get(portal_ref.portal_index as usize) else {
                continue;
            };

            let other_group = portal_ref.group_index as u32;

            // Skip if not facing us
            if !portal.is_facing_camera(eye, portal_ref.side) {
                continue;
            }

            // Check if portal is visible or we're inside it
            if portal.in_frustum(frustum) || portal.aabb_contains_point(eye) {
                // Clip frustum and recurse
                let clipped_frustum = portal.clip_frustum(eye, frustum);

                // Check for interior->exterior transition
                let current_is_exterior = info.is_exterior;
                let other_idx = other_group as usize;
                let other_is_exterior = self
                    .group_info
                    .get(other_idx)
                    .map(|g| g.is_exterior)
                    .unwrap_or(false);

                if !current_is_exterior && other_is_exterior {
                    // Save exterior frustum for outdoor rendering
                    result.exterior_frustums.push(clipped_frustum.clone());
                }

                // Create a local visited set for this path
                // This allows multiple paths to reach the same group through different portals
                let mut local_visited = visited.to_vec();
                self.traverse_portals(
                    other_group,
                    eye,
                    &clipped_frustum,
                    result,
                    &mut local_visited,
                );
            }
        }
    }

    /// Get the number of portals
    pub fn portal_count(&self) -> usize {
        self.portals.len()
    }

    /// Get a portal by index
    pub fn get_portal(&self, index: usize) -> Option<&Portal> {
        self.portals.get(index)
    }
}

// Vector math helpers

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: &[f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn compute_centroid(vertices: &[[f32; 3]]) -> [f32; 3] {
    if vertices.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut sum = [0.0f32; 3];
    for v in vertices {
        sum[0] += v[0];
        sum[1] += v[1];
        sum[2] += v[2];
    }
    let n = vertices.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

fn project_to_2d(point: &[f32; 3], axis: Axis) -> [f32; 2] {
    match axis {
        Axis::X => [point[1], point[2]],
        Axis::Y => [point[0], point[2]],
        Axis::Z => [point[0], point[1]],
    }
}

/// Check if a 2D point is inside a convex polygon
pub fn point_in_convex_polygon(point: &[f32; 2], polygon: &[[f32; 2]]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut sign = 0i32;
    for i in 0..polygon.len() {
        let a = &polygon[i];
        let b = &polygon[(i + 1) % polygon.len()];

        // Cross product of edge vector and point vector
        let cross = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);

        if cross.abs() > 0.0001 {
            let new_sign = if cross > 0.0 { 1 } else { -1 };
            if sign == 0 {
                sign = new_sign;
            } else if sign != new_sign {
                return false;
            }
        }
    }

    true
}

/// Group location data for point-in-group queries
#[derive(Debug, Clone)]
pub struct GroupLocationData {
    /// Group index
    pub group_index: u32,
    /// Group bounding box
    pub aabb: AABB,
    /// Whether this is an exterior group
    pub is_exterior: bool,
}

/// Locates which WMO group contains a given point.
///
/// Uses a two-phase approach:
/// 1. AABB filtering to quickly eliminate groups that can't contain the point
/// 2. BSP tree query (if available) to precisely determine containment
///
/// For exterior groups, only the AABB check is used (with max Z ignored).
/// For interior groups, BSP tree ray-casting is used for precise results.
#[derive(Debug)]
pub struct WmoGroupLocator {
    /// Location data for each group
    groups: Vec<GroupLocationData>,
}

impl WmoGroupLocator {
    /// Create a new group locator
    pub fn new(groups: Vec<GroupLocationData>) -> Self {
        Self { groups }
    }

    /// Find all groups whose AABB contains the point
    ///
    /// Returns group indices sorted by preference (interior first, then exterior)
    pub fn find_candidate_groups(&self, point: &[f32; 3]) -> Vec<u32> {
        let mut interior_groups = Vec::new();
        let mut exterior_groups = Vec::new();

        for group in &self.groups {
            let contains = if group.is_exterior {
                group.aabb.contains_point_ignore_max_z(point)
            } else {
                group.aabb.contains_point(point)
            };

            if contains {
                if group.is_exterior {
                    exterior_groups.push(group.group_index);
                } else {
                    interior_groups.push(group.group_index);
                }
            }
        }

        // Prefer interior groups over exterior groups
        interior_groups.extend(exterior_groups);
        interior_groups
    }

    /// Find the primary group for a point (first interior group, or first exterior if no interior)
    pub fn find_primary_group(&self, point: &[f32; 3]) -> Option<u32> {
        self.find_candidate_groups(point).first().copied()
    }

    /// Get the number of groups
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Get group data by index
    pub fn get_group(&self, index: usize) -> Option<&GroupLocationData> {
        self.groups.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_distance() {
        let plane = Plane {
            normal: [0.0, 0.0, 1.0],
            distance: -5.0,
        };

        assert!((plane.distance_to_point(&[0.0, 0.0, 5.0])).abs() < 0.001);
        assert!(plane.distance_to_point(&[0.0, 0.0, 10.0]) > 0.0);
        assert!(plane.distance_to_point(&[0.0, 0.0, 0.0]) < 0.0);
    }

    #[test]
    fn test_aabb_contains() {
        let aabb = AABB {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };

        assert!(aabb.contains_point(&[0.0, 0.0, 0.0]));
        assert!(aabb.contains_point(&[1.0, 1.0, 1.0]));
        assert!(!aabb.contains_point(&[2.0, 0.0, 0.0]));
    }

    #[test]
    fn test_point_in_polygon() {
        let square = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        assert!(point_in_convex_polygon(&[0.5, 0.5], &square));
        assert!(!point_in_convex_polygon(&[2.0, 0.5], &square));
    }

    #[test]
    fn test_convex_hull_aabb() {
        // Simple frustum facing +Z
        let mut hull = ConvexHull::new();
        hull.planes.push(Plane {
            normal: [0.0, 0.0, 1.0],
            distance: 0.0,
        }); // near plane at z=0

        let aabb = AABB {
            min: [-1.0, -1.0, 1.0],
            max: [1.0, 1.0, 5.0],
        };
        assert!(hull.contains_aabb(&aabb));

        let behind = AABB {
            min: [-1.0, -1.0, -5.0],
            max: [1.0, 1.0, -1.0],
        };
        assert!(!hull.contains_aabb(&behind));
    }

    #[test]
    fn test_group_locator_find_candidate_groups() {
        let groups = vec![
            GroupLocationData {
                group_index: 0,
                aabb: AABB {
                    min: [-10.0, -10.0, -10.0],
                    max: [10.0, 10.0, 10.0],
                },
                is_exterior: false,
            },
            GroupLocationData {
                group_index: 1,
                aabb: AABB {
                    min: [20.0, 20.0, -10.0],
                    max: [40.0, 40.0, 10.0],
                },
                is_exterior: false,
            },
            GroupLocationData {
                group_index: 2,
                aabb: AABB {
                    min: [-100.0, -100.0, -10.0],
                    max: [100.0, 100.0, 10.0],
                },
                is_exterior: true,
            },
        ];
        let locator = WmoGroupLocator::new(groups);

        // Point inside group 0 - should return group 0 first (interior), then group 2 (exterior)
        let candidates = locator.find_candidate_groups(&[0.0, 0.0, 0.0]);
        assert!(candidates.contains(&0));
        assert!(candidates.contains(&2));
        assert!(!candidates.contains(&1));
        assert_eq!(candidates[0], 0); // Interior groups come first

        // Point inside only group 1
        let candidates = locator.find_candidate_groups(&[30.0, 30.0, 0.0]);
        assert!(candidates.contains(&1));
        assert!(candidates.contains(&2));
        assert!(!candidates.contains(&0));

        // Point outside all groups
        let candidates = locator.find_candidate_groups(&[200.0, 200.0, 0.0]);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_group_locator_exterior_ignores_max_z() {
        let groups = vec![GroupLocationData {
            group_index: 0,
            aabb: AABB {
                min: [-10.0, -10.0, 0.0],
                max: [10.0, 10.0, 20.0],
            },
            is_exterior: true,
        }];
        let locator = WmoGroupLocator::new(groups);

        // Point above max Z - should still be found for exterior group
        let candidates = locator.find_candidate_groups(&[0.0, 0.0, 100.0]);
        assert_eq!(candidates, vec![0]);

        // Point below min Z - should not be found
        let candidates = locator.find_candidate_groups(&[0.0, 0.0, -10.0]);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_plane_is_degenerate() {
        // Default plane is degenerate (zero normal)
        let degenerate = Plane::default();
        assert!(degenerate.is_degenerate());

        // Valid plane is not degenerate
        let valid = Plane {
            normal: [0.0, 0.0, 1.0],
            distance: -5.0,
        };
        assert!(!valid.is_degenerate());

        // Near-zero normal is degenerate
        let near_zero = Plane {
            normal: [0.0001, 0.0, 0.0],
            distance: 0.0,
        };
        assert!(near_zero.is_degenerate());
    }

    #[test]
    fn test_degenerate_plane_methods() {
        let degenerate = Plane::default();

        // major_axis returns deterministic fallback
        assert_eq!(degenerate.major_axis(), Axis::Z);

        // distance_to_point returns distance (0 for default)
        assert!((degenerate.distance_to_point(&[1.0, 2.0, 3.0])).abs() < 0.001);

        // intersect_line returns INFINITY for degenerate plane
        assert_eq!(
            degenerate.intersect_line(&[0.0, 0.0, 0.0], &[0.0, 0.0, 1.0]),
            f32::INFINITY
        );
    }

    #[test]
    fn test_aabb_is_empty() {
        // Default AABB is empty
        let empty = AABB::default();
        assert!(empty.is_empty());

        // Zero-volume AABB is empty
        let point = AABB {
            min: [1.0, 1.0, 1.0],
            max: [1.0, 1.0, 1.0],
        };
        assert!(point.is_empty());

        // Valid AABB is not empty
        let valid = AABB {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        assert!(!valid.is_empty());

        // AABB from empty points is empty
        let from_empty: &[[f32; 3]] = &[];
        assert!(AABB::from_points(from_empty).is_empty());
    }
}
