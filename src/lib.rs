// Copyright 2017 Samuel Loretan <tynril@gmail.com> -- See LICENSE file

//! A Rust implementation of the Delaunay triangulation algorithm presented by
//! [Paul Bourke](http://paulbourke.net/papers/triangulate/).
//!
//! ## Usage
//!
//! Add the rtriangulate dependency to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rtriangulate = "0.2"
//! ```
//!
//! And use the crate as such:
//!
//! ```rust
//! extern crate rtriangulate;
//!
//! use rtriangulate::{Point, Triangle, triangulate};
//!
//! # fn main() {
//! // A list of points (which has to be sorted on x).
//! let points = [Point::new(10.0, 50.0), Point::new(30.0, 40.0), Point::new(25.0, 40.0)];
//! let triangles = triangulate(&points).unwrap();
//!
//! assert_eq!(triangles, [Triangle(0, 1, 2)]);
//! # }
//! ```

use std::cmp::Ordering;
use std::ops::Index;

pub type Result<T> = std::result::Result<T, TriangulateError>;

/// Possible triangulation errors.
#[derive(Debug)]
pub enum TriangulateError {
    /// At least three points are necessary to triangulate.
    NotEnoughPoints,
}

/// A two-dimensional point.
///
/// Compares so that it can easily be sorted in ascending `x` order, as required by the
/// `triangulate` function.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Makes a new point from xy coordinates.
    pub fn new(x: f64, y: f64) -> Point {
        Point { x: x, y: y }
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(o) => o,
            None => Ordering::Greater,
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.x.partial_cmp(&other.x) {
            Some(Ordering::Equal) => self.y.partial_cmp(&other.y),
            other => other,
        }
    }
}

/// A triangle, represented by indexes into a list of points.
#[derive(Debug, PartialEq)]
pub struct Triangle(pub usize, pub usize, pub usize);

/// An edge, represented by indexes into a list of points.
///
/// When compared, ignore the directionality of the edge, such as:
///
/// ```rust
/// use rtriangulate::Edge;
/// assert_eq!(Edge(0, 1), Edge(1, 0));
/// ```
#[derive(Debug, Clone)]
pub struct Edge(pub usize, pub usize);

impl PartialEq for Edge {
    /// Compare edges regardless of directionality.
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.1 && self.1 == other.0) || (self.0 == other.0 && self.1 == other.1)
    }
}

/// A view over two slices that can be indexed seamlessly across both.
///
/// This is used internally by the `triangulate` function as a way to treat the supertriangle
/// vertices as any other vertice, but without having to modify the input list of vertices.
struct TwoSlices<'a, T: 'a>(&'a [T], &'a [T]);

impl<'a, T: 'a> Index<usize> for TwoSlices<'a, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        let first_slice_len = self.0.len();
        if index < first_slice_len {
            &self.0[index]
        } else {
            &self.1[index - first_slice_len]
        }
    }
}

/// Generate the Delaunay triangulation of given set of points.
///
/// It takes a slice of points, and returns a vector of triangles arranged in clockwise order. The
/// list of points must have a least three entries, otherwise this function will panic. The points
/// *needs* to be sorted by increasing `x` value. The comparison operators on `Point` will respect
/// this, so you can simply call `sort()` on your points list if necessary.
///
/// The returned triangles are indices into the input slice of points.
///
/// Example:
///
/// ```rust
/// use rtriangulate::{Point, Triangle, triangulate};
///
/// // Note that the points are sorted in ascending order of x.
/// let points = [
///     Point::new(10.0, 10.0), Point::new(15.0, 25.0), Point::new(25.0, 15.0),
///     Point::new(30.0, 25.0), Point::new(40.0, 15.0)
/// ];
///
/// let triangles = triangulate(&points).unwrap();
/// assert_eq!(
///     triangles,
///     [Triangle(0, 1, 2), Triangle(2, 1, 3), Triangle(0, 2, 4), Triangle(2, 3, 4)]
/// );
/// ```
pub fn triangulate(points: &[Point]) -> Result<Vec<Triangle>> {
    // Make sure we have enough points to do a triangulation.
    let points_count = points.len();
    if points_count < 3 {
        return Err(TriangulateError::NotEnoughPoints);
    }

    // Find the bounds of the space that contains our points.
    let (min_point, max_point) = points.iter().fold((points[0], points[0]), |acc, ref p| {
        (Point::new(acc.0.x.min(p.x), acc.0.y.min(p.y)),
         Point::new(acc.1.x.max(p.x), acc.1.y.max(p.y)))
    });
    let delta_point = Point::new(max_point.x - min_point.x, max_point.y - min_point.y);
    let delta_max = delta_point.x.max(delta_point.y);
    let mid_point = Point::new((max_point.x + min_point.x) * 0.5,
                               (max_point.y + min_point.y) * 0.5);

    // Compute the supertriangle, which encompasses all the input points.
    let supertriangle = [Point::new(mid_point.x - 2.0 * delta_max, mid_point.y - delta_max),
                         Point::new(mid_point.x, mid_point.y + 2.0 * delta_max),
                         Point::new(mid_point.x + 2.0 * delta_max, mid_point.y - delta_max)];

    // Make an iterable slice of our points and the supertriangle.
    let all_points = TwoSlices(points, &supertriangle);

    // The list of triangles we're gonna fill, initialized with the super-triangle.
    let mut triangles = vec![Triangle(points_count, points_count + 1, points_count + 2)];

    // Include each of the input point into the mesh.
    let mut edges = Vec::<Edge>::with_capacity(18);
    let mut to_remove = Vec::<usize>::with_capacity(10);
    for i in 0..points_count {
        // Adding relevant edges.
        triangles.retain(|ref t| {
            if in_circumcircle(all_points[i],
                               (all_points[t.0], all_points[t.1], all_points[t.2])) {
                edges.extend_from_slice(&[Edge(t.0, t.1), Edge(t.1, t.2), Edge(t.2, t.0)]);
                false
            } else {
                true
            }
        });

        // Remove duplicate edges (both pairs).
        let edges_count = edges.len();
        for (j, ref e1) in edges.iter().enumerate().rev().skip(1) {
            for (k, ref e2) in edges.iter().enumerate().rev().take(edges_count - j - 1) {
                if e1 == e2 {
                    to_remove.extend_from_slice(&[j, k]);
                    break;
                }
            }
        }
        to_remove.sort();
        for j in to_remove.iter().rev() {
            edges.remove(*j);
        }
        to_remove.clear();

        // Form new triangles from the remaining edges. Edges are added in clockwise order.
        triangles.extend(edges.iter().map(|ref e| Triangle(e.0, e.1, i)));
        edges.clear();
    }

    // Remove triangles with supertriangle vertices
    triangles.retain(|ref t| t.0 < points_count && t.1 < points_count && t.2 < points_count);

    Ok(triangles)
}

/// Returns true if the point lies inside (or on the edge of) the circumcircle made from the
/// triangle (t0, t1, t2).
fn in_circumcircle(point: Point, (t0, t1, t2): (Point, Point, Point)) -> bool {
    // Handle coincident points in the input triangle.
    if (t0.y - t1.y).abs() < std::f64::EPSILON && (t1.y - t2.y).abs() < std::f64::EPSILON {
        return false;
    }

    // Compute the center of the triangle's circumcircle.
    let mut circumcircle_center = Point::new(0.0, 0.0);
    if (t1.y - t0.y).abs() < std::f64::EPSILON {
        let mid = 0.0 - (t2.x - t1.x) / (t2.y - t1.y);
        let mid_point = Point::new((t1.x + t2.x) * 0.5, (t1.y + t2.y) * 0.5);
        circumcircle_center.x = (t1.x + t0.x) * 0.5;
        circumcircle_center.y = mid * (circumcircle_center.x - mid_point.x) + mid_point.y;
    } else if (t2.y - t1.y).abs() < std::f64::EPSILON {
        let mid = 0.0 - (t1.x - t0.x) / (t1.y - t0.y);
        let mid_point = Point::new((t0.x + t1.x) * 0.5, (t0.y + t1.y) * 0.5);
        circumcircle_center.x = (t2.x + t1.x) * 0.5;
        circumcircle_center.y = mid * (circumcircle_center.x - mid_point.x) + mid_point.y;
    } else {
        let mid1 = 0.0 - (t1.x - t0.x) / (t1.y - t0.y);
        let mid2 = 0.0 - (t2.x - t1.x) / (t2.y - t1.y);
        let mid_point1 = Point::new((t0.x + t1.x) * 0.5, (t0.y + t1.y) * 0.5);
        let mid_point2 = Point::new((t1.x + t2.x) * 0.5, (t1.y + t2.y) * 0.5);
        circumcircle_center.x = (mid1 * mid_point1.x - mid2 * mid_point2.x + mid_point2.y -
                                 mid_point1.y) / (mid1 - mid2);
        circumcircle_center.y = mid1 * (circumcircle_center.x - mid_point1.x) + mid_point1.y;
    }

    // Check the radius of the circumcircle against the point's distance from its center.
    let circumcircle_radius_sq = (t1.x - circumcircle_center.x).powf(2.0) +
                                 (t1.y - circumcircle_center.y).powf(2.0);
    let point_distance_sq = (point.x - circumcircle_center.x).powf(2.0) +
                            (point.y - circumcircle_center.y).powf(2.0);

    point_distance_sq <= circumcircle_radius_sq
}

#[cfg(test)]
mod tests {
    use super::{Point, Triangle, triangulate};

    #[test]
    fn test_simple() {
        let points = [Point::new(10.0, 10.0), Point::new(15.0, 25.0), Point::new(25.0, 15.0)];
        let tris: Vec<Triangle> = triangulate(&points).unwrap();

        assert_eq!(tris.len(), 1);
        assert_eq!(tris[..], [Triangle(0, 1, 2)][..]);
    }

    #[test]
    fn test_four_triangles() {
        let points = [Point::new(10.0, 10.0),
                      Point::new(15.0, 25.0),
                      Point::new(25.0, 15.0),
                      Point::new(30.0, 25.0),
                      Point::new(40.0, 15.0)];

        let tris: Vec<Triangle> = triangulate(&points).unwrap();
        assert_eq!(tris.len(), 4);

        let expected_tris =
            [Triangle(0, 1, 2), Triangle(2, 1, 3), Triangle(0, 2, 4), Triangle(2, 3, 4)];
        assert_eq!(tris[..], expected_tris[..]);
    }

    #[test]
    fn test_overlapping() {
        let points = vec![Point::new(10.0, 10.0), Point::new(25.0, 15.0), Point::new(25.0, 15.0)];

        let tris: Vec<Triangle> = triangulate(&points).unwrap();

        assert_eq!(tris.len(), 1);
        assert_eq!(tris[..], [Triangle(0, 1, 2)][..]);
    }

    #[test]
    fn test_complex() {
        let points = [Point::new(11.0, 264.0),
                      Point::new(65.0, 216.0),
                      Point::new(104.0, 522.0),
                      Point::new(168.0, 90.0),
                      Point::new(181.0, 479.0),
                      Point::new(245.0, 119.0),
                      Point::new(267.0, 152.0),
                      Point::new(282.0, 17.0),
                      Point::new(285.0, 561.0),
                      Point::new(329.0, 432.0),
                      Point::new(331.0, 244.0),
                      Point::new(348.0, 504.0),
                      Point::new(448.0, 36.0),
                      Point::new(450.0, 514.0),
                      Point::new(601.0, 535.0),
                      Point::new(623.0, 335.0),
                      Point::new(627.0, 461.0),
                      Point::new(667.0, 462.0),
                      Point::new(688.0, 605.0),
                      Point::new(742.0, 363.0),
                      Point::new(829.0, 512.0),
                      Point::new(836.0, 20.0),
                      Point::new(839.0, 178.0),
                      Point::new(876.0, 110.0),
                      Point::new(895.0, 666.0)];

        println!("{:?}", points);

        let tris: Vec<Triangle> = triangulate(&points).unwrap();

        let expected_tris = [Triangle(0, 1, 3),
                             Triangle(1, 0, 4),
                             Triangle(0, 2, 4),
                             Triangle(3, 1, 6),
                             Triangle(5, 3, 6),
                             Triangle(3, 5, 7),
                             Triangle(5, 6, 7),
                             Triangle(4, 2, 8),
                             Triangle(4, 8, 9),
                             Triangle(1, 4, 10),
                             Triangle(6, 1, 10),
                             Triangle(4, 9, 10),
                             Triangle(9, 8, 11),
                             Triangle(7, 6, 12),
                             Triangle(6, 10, 12),
                             Triangle(11, 8, 13),
                             Triangle(9, 11, 13),
                             Triangle(10, 9, 15),
                             Triangle(9, 13, 15),
                             Triangle(12, 10, 15),
                             Triangle(13, 14, 16),
                             Triangle(15, 13, 16),
                             Triangle(16, 14, 17),
                             Triangle(15, 16, 17),
                             Triangle(13, 8, 18),
                             Triangle(14, 13, 18),
                             Triangle(17, 14, 18),
                             Triangle(15, 17, 19),
                             Triangle(17, 18, 20),
                             Triangle(19, 17, 20),
                             Triangle(7, 12, 21),
                             Triangle(12, 15, 22),
                             Triangle(21, 12, 22),
                             Triangle(15, 19, 22),
                             Triangle(19, 20, 22),
                             Triangle(22, 20, 23),
                             Triangle(21, 22, 23),
                             Triangle(20, 18, 24),
                             Triangle(23, 20, 24)];

        assert_eq!(tris.len(), 39);
        assert_eq!(tris[..], expected_tris[..]);
    }

    #[test]
    #[should_panic]
    fn test_less_than_three_points() {
        let points = [Point::new(10.0, 10.0)];
        triangulate(&points).unwrap();
    }

    #[test]
    fn test_points_ordering() {
        let mut unsorted_points =
            [Point::new(10.0, 7.5), Point::new(5.0, 5.0), Point::new(10.0, 5.0)];
        unsorted_points.sort();

        assert_eq!(unsorted_points,
                   [Point::new(5.0, 5.0), Point::new(10.0, 5.0), Point::new(10.0, 7.5)]);
    }
}
