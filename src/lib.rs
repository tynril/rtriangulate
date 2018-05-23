// Copyright 2017-2018 Samuel Loretan <tynril@gmail.com> -- See LICENSE file

//! A Rust implementation of the Delaunay triangulation algorithm presented by
//! [Paul Bourke](http://paulbourke.net/papers/triangulate/).
//!
//! ## Usage
//!
//! Add the rtriangulate dependency to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rtriangulate = "0.3"
//! ```
//!
//! And use the crate as such:
//!
//! ```rust
//! extern crate rtriangulate;
//!
//! use rtriangulate::{TriangulationPoint, Triangle, triangulate};
//!
//! # fn main() {
//! // A list of points (which has to be sorted on x).
//! let points = [
//!     TriangulationPoint::new(10.0, 50.0),
//!     TriangulationPoint::new(25.0, 40.0),
//!     TriangulationPoint::new(30.0, 40.0)
//! ];
//! let triangles = triangulate(&points).unwrap();
//!
//! assert_eq!(triangles, [Triangle(1, 0, 2)]);
//! # }
//! ```

extern crate num_traits;

use num_traits::float::FloatCore;
use std::marker::PhantomData;

pub type Result<T> = std::result::Result<T, TriangulateError>;

/// Possible triangulation errors.
#[derive(Debug)]
pub enum TriangulateError {
    /// At least three points are necessary to triangulate.
    NotEnoughPoints,
}

/// A trait for two-dimensional points.
///
/// This is the trait your point type needs to implement to be able to be passed to the
/// `triangulate` function.
///
/// For your convenience, a `sort_points` function is provided. This function can be used in
/// the `sort_by` or `sort_unstable_by` functions on slices, and will order the points in the
/// order necessary for `triangulate` to work (which is in ascending `x` order).
pub trait Point<T>
where
    T: FloatCore,
{
    /// Returns the `x` component of this point.
    fn x(&self) -> T;

    /// Returns the `y` component of this point.
    fn y(&self) -> T;
}

/// A utility function to sort points.
///
/// Use this function by passing it to `sort_by` or `sort_unstable_by` on your slice of points.
/// The ordering this function applies is what the `triangulate` function expects, which is an
/// ascending `x` order.
pub fn sort_points<T, P1, P2>(a: &P1, b: &P2) -> std::cmp::Ordering
where
    T: FloatCore,
    P1: Point<T>,
    P2: Point<T>,
{
    match a.x().partial_cmp(&b.x()) {
        Some(std::cmp::Ordering::Equal) => a.y().partial_cmp(&b.y()),
        other => other,
    }.unwrap_or(std::cmp::Ordering::Greater)
}

/// A two-dimensional point of generic precision, which implements the `Point` trait.
///
/// If you're not using your own type implementing the `Point` trait, feel free to use this one.
/// It is very simple, and doesn't provide much features, but it is functional. Internally, this
/// type is used for temporary storage of points that need to be created during triangulation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TriangulationPoint<T>
where
    T: FloatCore,
{
    pub x: T,
    pub y: T,
}

impl<T> TriangulationPoint<T>
where
    T: FloatCore,
{
    /// Makes a new point from xy coordinates.
    #[inline(always)]
    pub fn new(x: T, y: T) -> Self {
        TriangulationPoint { x, y }
    }

    /// A point located at the infinity.
    #[inline(always)]
    fn infinity() -> Self {
        TriangulationPoint {
            x: T::infinity(),
            y: T::infinity(),
        }
    }

    /// A point located at the negative infinity.
    #[inline(always)]
    fn neg_infinity() -> Self {
        TriangulationPoint {
            x: T::neg_infinity(),
            y: T::neg_infinity(),
        }
    }
}

impl<T> Point<T> for TriangulationPoint<T>
where
    T: FloatCore,
{
    /// The `x` component of this triangulation point.
    #[inline(always)]
    fn x(&self) -> T {
        self.x
    }

    /// The `y` component of this triangulation point.
    #[inline(always)]
    fn y(&self) -> T {
        self.y
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
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.1 && self.1 == other.0) || (self.0 == other.0 && self.1 == other.1)
    }
}

/// A view over two slices that can be indexed seamlessly across both.
///
/// This is used internally by the `triangulate` function as a way to treat the supertriangle
/// vertices as any other vertice, but without having to modify the input list of vertices.
struct TwoPointsSlices<'a, S1: 'a, S2: 'a, T>(&'a [S1], &'a [S2], PhantomData<T>);

impl<'a, S1, S2, T> TwoPointsSlices<'a, S1, S2, T>
where
    T: FloatCore,
    S1: Point<T>,
    S2: Point<T>,
{
    /// Make a new view over two slices of points.
    #[inline(always)]
    fn new(first: &'a [S1], second: &'a [S2]) -> Self {
        TwoPointsSlices(first, second, PhantomData)
    }

    /// Get the point at a given index across both slices of points.
    #[inline(always)]
    fn get(&self, index: usize) -> &Point<T> {
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
/// *needs* to be sorted by increasing `x` value. The `sort_points` function provided in this
/// module can be used, in conjunction with `sort_by` or `sort_unstable_by`, to order your slice
/// of points as necessary.
///
/// The returned triangles are indices into the input slice of points.
///
/// Example:
///
/// ```rust
/// use rtriangulate::{TriangulationPoint, Triangle, triangulate};
///
/// // Note that the points are sorted in ascending order of x.
/// let points = [
///     TriangulationPoint::new(10.0, 10.0),
///     TriangulationPoint::new(15.0, 25.0),
///     TriangulationPoint::new(25.0, 15.0),
///     TriangulationPoint::new(30.0, 25.0),
///     TriangulationPoint::new(40.0, 15.0)
/// ];
///
/// // If needed, you can sort your points like this:
/// // points.sort_unstable_by(rtriangulate::sort_points);
///
/// let triangles = triangulate(&points).unwrap();
/// assert_eq!(
///     triangles,
///     [Triangle(0, 1, 2), Triangle(2, 1, 3), Triangle(0, 2, 4), Triangle(2, 3, 4)]
/// );
/// ```
pub fn triangulate<T, P>(points: &[P]) -> Result<Vec<Triangle>>
where
    T: FloatCore,
    P: Point<T>,
{
    // Make sure we have enough points to do a triangulation.
    let points_count = points.len();
    if points_count < 3 {
        return Err(TriangulateError::NotEnoughPoints);
    }

    // Compute a constant we'll need later.
    let half = T::from(0.5).unwrap();
    let two = T::from(2.0).unwrap();

    // Find the bounds of the space that contains our points.
    let (min_point, max_point) = points.iter().fold(
        (
            TriangulationPoint::<T>::infinity(),
            TriangulationPoint::<T>::neg_infinity(),
        ),
        |acc, p| {
            (
                TriangulationPoint::<T>::new(acc.0.x().min(p.x()), acc.0.y().min(p.y())),
                TriangulationPoint::<T>::new(acc.1.x().max(p.x()), acc.1.y().max(p.y())),
            )
        },
    );
    let delta_point =
        TriangulationPoint::new(max_point.x() - min_point.x(), max_point.y() - min_point.y());
    let delta_max = delta_point.x.max(delta_point.y);
    let mid_point = TriangulationPoint::new(
        (max_point.x() + min_point.x()) * half,
        (max_point.y() + min_point.y()) * half,
    );

    // Compute the supertriangle, which encompasses all the input points.
    let supertriangle = [
        TriangulationPoint::<T>::new(mid_point.x - two * delta_max, mid_point.y - delta_max),
        TriangulationPoint::<T>::new(mid_point.x, mid_point.y + two * delta_max),
        TriangulationPoint::<T>::new(mid_point.x + two * delta_max, mid_point.y - delta_max),
    ];

    // Make an iterable slice of our points and the supertriangle.
    let all_points = TwoPointsSlices::new(points, &supertriangle);

    // The list of triangles we're gonna fill, initialized with the super-triangle.
    let mut triangles = vec![Triangle(points_count, points_count + 1, points_count + 2)];

    // Include each of the input point into the mesh.
    let mut edges = Vec::<Edge>::with_capacity(18);
    let mut to_remove = Vec::<usize>::with_capacity(10);
    for i in 0..points_count {
        triangles.retain(|t| {
            if in_circumcircle(
                all_points.get(i),
                all_points.get(t.0),
                all_points.get(t.1),
                all_points.get(t.2),
            ) {
                edges.extend_from_slice(&[Edge(t.0, t.1), Edge(t.1, t.2), Edge(t.2, t.0)]);
                false
            } else {
                true
            }
        });

        // Remove duplicate edges (both pairs).
        let edges_count = edges.len();
        for (j, e1) in edges.iter().enumerate().rev().skip(1) {
            for (k, e2) in edges.iter().enumerate().rev().take(edges_count - j - 1) {
                if e1 == e2 {
                    to_remove.extend_from_slice(&[j, k]);
                    break;
                }
            }
        }
        to_remove.sort();
        to_remove.dedup();
        for j in to_remove.iter().rev() {
            edges.remove(*j);
        }
        to_remove.clear();

        // Form new triangles from the remaining edges. Edges are added in clockwise order.
        triangles.extend(edges.iter().map(|e| Triangle(e.0, e.1, i)));
        edges.clear();
    }

    // Remove triangles with supertriangle vertices
    triangles.retain(|t| t.0 < points_count && t.1 < points_count && t.2 < points_count);

    Ok(triangles)
}

/// Returns true if the point lies inside (or on the edge of) the circumcircle made from the
/// triangle made off of points t0, t1, and t2.
#[inline(always)]
fn in_circumcircle<T>(point: &Point<T>, t0: &Point<T>, t1: &Point<T>, t2: &Point<T>) -> bool
where
    T: FloatCore,
{
    // Handle coincident points in the input triangle.
    if (t0.y() - t1.y()).abs() < T::epsilon() && (t1.y() - t2.y()).abs() < T::epsilon() {
        return false;
    }

    let half = T::from(0.5).unwrap();

    // Compute the center of the triangle's circumcircle.
    let (circ_x, circ_y) = if (t1.y() - t0.y()).abs() < T::epsilon() {
        let mid = T::zero() - (t2.x() - t1.x()) / (t2.y() - t1.y());
        let mid_point = TriangulationPoint::new((t1.x() + t2.x()) * half, (t1.y() + t2.y()) * half);
        let x = (t1.x() + t0.x()) * half;
        (x, mid * (x - mid_point.x) + mid_point.y)
    } else if (t2.y() - t1.y()).abs() < T::epsilon() {
        let mid = T::zero() - (t1.x() - t0.x()) / (t1.y() - t0.y());
        let mid_point = TriangulationPoint::new((t0.x() + t1.x()) * half, (t0.y() + t1.y()) * half);
        let x = (t2.x() + t1.x()) * half;
        (x, mid * (x - mid_point.x) + mid_point.y)
    } else {
        let mid1 = T::zero() - (t1.x() - t0.x()) / (t1.y() - t0.y());
        let mid2 = T::zero() - (t2.x() - t1.x()) / (t2.y() - t1.y());
        let mid_point1 =
            TriangulationPoint::new((t0.x() + t1.x()) * half, (t0.y() + t1.y()) * half);
        let mid_point2 =
            TriangulationPoint::new((t1.x() + t2.x()) * half, (t1.y() + t2.y()) * half);
        let x = (mid1 * mid_point1.x - mid2 * mid_point2.x + mid_point2.y - mid_point1.y)
            / (mid1 - mid2);
        (x, mid1 * (x - mid_point1.x) + mid_point1.y)
    };

    // Check the radius of the circumcircle against the point's distance from its center.
    let circumcircle_radius_sq = (t1.x() - circ_x).powi(2) + (t1.y() - circ_y).powi(2);
    let point_distance_sq = (point.x() - circ_x).powi(2) + (point.y() - circ_y).powi(2);

    point_distance_sq <= circumcircle_radius_sq
}

#[cfg(test)]
mod tests {
    use super::{sort_points, triangulate, Triangle, TriangulationPoint};

    #[test]
    fn test_simple() {
        let points = [
            TriangulationPoint::new(10.0, 10.0),
            TriangulationPoint::new(15.0, 25.0),
            TriangulationPoint::new(25.0, 15.0),
        ];
        let tris: Vec<Triangle> = triangulate(&points).unwrap();

        assert_eq!(tris.len(), 1);
        assert_eq!(tris[..], [Triangle(0, 1, 2)][..]);
    }

    #[test]
    fn test_four_triangles() {
        let points = [
            TriangulationPoint::new(10.0, 10.0),
            TriangulationPoint::new(15.0, 25.0),
            TriangulationPoint::new(25.0, 15.0),
            TriangulationPoint::new(30.0, 25.0),
            TriangulationPoint::new(40.0, 15.0),
        ];

        let tris: Vec<Triangle> = triangulate(&points).unwrap();
        assert_eq!(tris.len(), 4);

        let expected_tris = [
            Triangle(0, 1, 2),
            Triangle(2, 1, 3),
            Triangle(0, 2, 4),
            Triangle(2, 3, 4),
        ];
        assert_eq!(tris[..], expected_tris[..]);
    }

    #[test]
    fn test_overlapping() {
        let points = vec![
            TriangulationPoint::new(10.0, 10.0),
            TriangulationPoint::new(25.0, 15.0),
            TriangulationPoint::new(25.0, 15.0),
        ];

        let tris: Vec<Triangle> = triangulate(&points).unwrap();

        assert_eq!(tris.len(), 1);
        assert_eq!(tris[..], [Triangle(0, 1, 2)][..]);
    }

    #[test]
    fn test_complex() {
        let points = [
            TriangulationPoint::new(11.0, 264.0),
            TriangulationPoint::new(65.0, 216.0),
            TriangulationPoint::new(104.0, 522.0),
            TriangulationPoint::new(168.0, 90.0),
            TriangulationPoint::new(181.0, 479.0),
            TriangulationPoint::new(245.0, 119.0),
            TriangulationPoint::new(267.0, 152.0),
            TriangulationPoint::new(282.0, 17.0),
            TriangulationPoint::new(285.0, 561.0),
            TriangulationPoint::new(329.0, 432.0),
            TriangulationPoint::new(331.0, 244.0),
            TriangulationPoint::new(348.0, 504.0),
            TriangulationPoint::new(448.0, 36.0),
            TriangulationPoint::new(450.0, 514.0),
            TriangulationPoint::new(601.0, 535.0),
            TriangulationPoint::new(623.0, 335.0),
            TriangulationPoint::new(627.0, 461.0),
            TriangulationPoint::new(667.0, 462.0),
            TriangulationPoint::new(688.0, 605.0),
            TriangulationPoint::new(742.0, 363.0),
            TriangulationPoint::new(829.0, 512.0),
            TriangulationPoint::new(836.0, 20.0),
            TriangulationPoint::new(839.0, 178.0),
            TriangulationPoint::new(876.0, 110.0),
            TriangulationPoint::new(895.0, 666.0),
        ];

        let tris: Vec<Triangle> = triangulate(&points).unwrap();

        let expected_tris = [
            Triangle(0, 1, 3),
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
            Triangle(23, 20, 24),
        ];

        assert_eq!(tris.len(), 39);
        assert_eq!(tris[..], expected_tris[..]);
    }

    #[test]
    fn test_coincident_points() {
        let points = [
            TriangulationPoint::new(10.0, 10.0),
            TriangulationPoint::new(10.0, 10.0),
            TriangulationPoint::new(11.0, 10.0),
            TriangulationPoint::new(11.0, 10.0),
        ];

        let tris: Vec<Triangle> = triangulate(&points).unwrap();
        assert_eq!(tris.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_less_than_three_points() {
        let points = [TriangulationPoint::new(10.0, 10.0)];
        triangulate(&points).unwrap();
    }

    #[test]
    fn test_points_ordering() {
        let mut unsorted_points = [
            TriangulationPoint::new(10.0, 7.5),
            TriangulationPoint::new(5.0, 5.0),
            TriangulationPoint::new(10.0, 5.0),
        ];
        unsorted_points.sort_by(sort_points);

        assert_eq!(
            unsorted_points,
            [
                TriangulationPoint::new(5.0, 5.0),
                TriangulationPoint::new(10.0, 5.0),
                TriangulationPoint::new(10.0, 7.5)
            ]
        );
    }
}
