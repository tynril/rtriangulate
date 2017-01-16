// Copyright 2017 Samuel Loretan <tynril@gmail.com>

//! Triangulate a set of points

/// A two dimentional point.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// Makes a new point from xy coordinates.
    pub fn new(x: f64, y: f64) -> Point {
        Point { x: x, y: y }
    }
}

/// A triangle, represented by indexes into a vertice list.
#[derive(Debug, PartialEq)]
pub struct Triangle(usize, usize, usize);

/// An edge, represented by indexes into a vertice list.
#[derive(Debug, Clone)]
struct Edge(usize, usize);

impl PartialEq for Edge {
    /// Compare edges regardless of directionality.
    fn eq(&self, other: &Edge) -> bool {
        (self.0 == other.1 && self.1 == other.0) || (self.0 == other.0 && self.1 == other.1)
    }
}

/// An indexable view over two slices.
struct TwoSlices<'a, T: 'a>(&'a [T], &'a [T]);

use std::ops::Index;
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

/// Triangulate a given set of points. The returned triangles are indices into the list of points.
pub fn triangulate(points: &[Point]) -> Vec<Triangle> {
    // Make sure we have enough points to do a triangulation.
    let points_count = points.len();
    if points_count < 3 {
        panic!("Can't triangulate less than three points.")
    }

    // Find the bounds of the space that contains our points.
    let (min_point, max_point) = points.iter().fold((points[0], points[0]), |acc, &p| {
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
    for i in 0..points_count {
        // Storage for the edges
        let mut edges = Vec::<Edge>::new();
        triangles.retain(|ref t| {
            if in_circle(all_points[i],
                         all_points[t.0],
                         all_points[t.1],
                         all_points[t.2]) {
                edges.extend([Edge(t.0, t.1), Edge(t.1, t.2), Edge(t.2, t.0)].iter().cloned());
                false
            } else {
                true
            }
        });

        // Remove duplicate edges.
        let mut to_remove = Vec::<usize>::new();
        let edges_count = edges.len();
        for (j, ref e1) in edges.iter().enumerate().rev().skip(1) {
            for (k, ref e2) in edges.iter().enumerate().rev().take(edges_count - j - 1) {
                if e1 == e2 {
                    to_remove.extend([j, k].iter().cloned());
                    break;
                }
            }
        }
        to_remove.sort();
        for j in to_remove.iter().rev() {
            edges.remove(*j);
        }

        // Form new triangles from the remaining edges. Edges are added in clockwise order.
        triangles.extend(edges.iter().map(|ref e| Triangle(e.0, e.1, i)));
    }

    // Remove triangles with supertriangle vertices
    triangles.retain(|ref t| t.0 < points_count && t.1 < points_count && t.2 < points_count);

    triangles
}

/// Returns true if the first point lies inside (or on the edge of) the circumcircle made from the
/// three other points.
fn in_circle(p: Point, pc1: Point, pc2: Point, pc3: Point) -> bool {
    // Handle coincident points.
    if (pc1.y - pc2.y).abs() < std::f64::EPSILON && (pc2.y - pc3.y).abs() < std::f64::EPSILON {
        return false;
    }

    let m1: f64;
    let m2: f64;
    let mx1: f64;
    let mx2: f64;
    let my1: f64;
    let my2: f64;
    let xc: f64;
    let yc: f64;

    if (pc2.y - pc1.y).abs() < std::f64::EPSILON {
        m2 = 0.0 - (pc3.x - pc2.x) / (pc3.y - pc2.y);
        mx2 = (pc2.x + pc3.x) * 0.5;
        my2 = (pc2.y + pc3.y) * 0.5;
        xc = (pc2.x + pc1.x) * 0.5;
        yc = m2 * (xc - mx2) + my2;
    } else if (pc3.y - pc2.y).abs() < std::f64::EPSILON {
        m1 = 0.0 - (pc2.x - pc1.x) / (pc2.y - pc1.y);
        mx1 = (pc1.x + pc2.x) * 0.5;
        my1 = (pc1.y + pc2.y) * 0.5;
        xc = (pc3.x + pc2.x) * 0.5;
        yc = m1 * (xc - mx1) + my1;
    } else {
        m1 = 0.0 - (pc2.x - pc1.x) / (pc2.y - pc1.y);
        m2 = 0.0 - (pc3.x - pc2.x) / (pc3.y - pc2.y);
        mx1 = (pc1.x + pc2.x) * 0.5;
        mx2 = (pc2.x + pc3.x) * 0.5;
        my1 = (pc1.y + pc2.y) * 0.5;
        my2 = (pc2.y + pc3.y) * 0.5;
        xc = (m1 * mx1 - m2 * mx2 + my2 - my1) / (m1 - m2);
        yc = m1 * (xc - mx1) + my1;
    }

    let rsqr = (pc2.x - xc).powf(2.0) + (pc2.y - yc).powf(2.0);
    let drsqr = (p.x - xc).powf(2.0) + (p.y - yc).powf(2.0);

    drsqr <= rsqr
}

#[cfg(test)]
mod test {
    use super::{Point, Triangle, triangulate};

    #[test]
    fn test_simple() {
        let points = vec![Point::new(10.0, 10.0), Point::new(25.0, 15.0), Point::new(15.0, 25.0)];

        let tris: Vec<Triangle> = triangulate(&points);

        assert_eq!(tris.len(), 1);
        assert_eq!(tris[..], [Triangle(1, 0, 2)][..]);
    }

    #[test]
    fn test_four_triangles() {
        let points = vec![Point::new(10.0, 10.0),
                          Point::new(25.0, 15.0),
                          Point::new(15.0, 25.0),
                          Point::new(30.0, 25.0),
                          Point::new(40.0, 15.0)];

        let tris: Vec<Triangle> = triangulate(&points);
        assert_eq!(tris.len(), 4);

        let expected_tris =
            [Triangle(1, 0, 2), Triangle(1, 2, 3), Triangle(0, 1, 4), Triangle(1, 3, 4)];
        assert_eq!(tris[..], expected_tris[..]);
    }

    #[test]
    fn test_overlapping() {
        let points = vec![Point::new(10.0, 10.0), Point::new(25.0, 15.0), Point::new(25.0, 15.0)];

        let tris: Vec<Triangle> = triangulate(&points);

        assert_eq!(tris.len(), 1);
        assert_eq!(tris[..], [Triangle(0, 1, 2)][..]);
    }

    #[test]
    fn test_complex() {
        let points = vec![Point::new(601.0, 535.0),
                          Point::new(895.0, 666.0),
                          Point::new(876.0, 110.0),
                          Point::new(448.0, 36.0),
                          Point::new(829.0, 512.0),
                          Point::new(742.0, 363.0),
                          Point::new(267.0, 152.0),
                          Point::new(331.0, 244.0),
                          Point::new(623.0, 335.0),
                          Point::new(245.0, 119.0),
                          Point::new(104.0, 522.0),
                          Point::new(285.0, 561.0),
                          Point::new(282.0, 17.0),
                          Point::new(836.0, 20.0),
                          Point::new(667.0, 462.0),
                          Point::new(65.0, 216.0),
                          Point::new(839.0, 178.0),
                          Point::new(11.0, 264.0),
                          Point::new(181.0, 479.0),
                          Point::new(168.0, 90.0),
                          Point::new(348.0, 504.0),
                          Point::new(688.0, 605.0),
                          Point::new(329.0, 432.0),
                          Point::new(627.0, 461.0),
                          Point::new(450.0, 514.0)];

        let tris: Vec<Triangle> = triangulate(&points);

        let expected_tris = [Triangle(1, 2, 4),
                             Triangle(3, 6, 7),
                             Triangle(3, 7, 8),
                             Triangle(6, 3, 12),
                             Triangle(9, 6, 12),
                             Triangle(12, 3, 13),
                             Triangle(4, 5, 14),
                             Triangle(5, 8, 14),
                             Triangle(7, 6, 15),
                             Triangle(4, 2, 16),
                             Triangle(5, 4, 16),
                             Triangle(8, 5, 16),
                             Triangle(3, 8, 16),
                             Triangle(13, 3, 16),
                             Triangle(2, 13, 16),
                             Triangle(10, 11, 18),
                             Triangle(7, 15, 18),
                             Triangle(15, 17, 18),
                             Triangle(17, 10, 18),
                             Triangle(6, 9, 19),
                             Triangle(15, 6, 19),
                             Triangle(9, 12, 19),
                             Triangle(17, 15, 19),
                             Triangle(1, 4, 21),
                             Triangle(4, 14, 21),
                             Triangle(14, 0, 21),
                             Triangle(8, 7, 22),
                             Triangle(7, 18, 22),
                             Triangle(18, 11, 22),
                             Triangle(11, 20, 22),
                             Triangle(0, 14, 23),
                             Triangle(14, 8, 23),
                             Triangle(20, 11, 24),
                             Triangle(11, 21, 24),
                             Triangle(21, 0, 24),
                             Triangle(8, 22, 24),
                             Triangle(23, 8, 24),
                             Triangle(0, 23, 24),
                             Triangle(22, 20, 24)];

        assert_eq!(tris.len(), 39);
        assert_eq!(tris[..], expected_tris[..]);
    }
}
