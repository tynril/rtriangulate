rtriangulate
============

A Rust implementation of the Delaunay triangulation algorithm presented by
[Paul Bourke](http://paulbourke.net/papers/triangulate/).

Example
-------

```rust
extern crate rtriangulate;

use rtriangulate::{Point, triangulate};

fn main() {
    let points = [Point::new(10.0, 50.0), Point::new(30.0, 40.0), Point::new(25.0, 40.0)];
    let triangles = triangulate(&points);

    println!("{:?}", triangles); // [Triangle(0, 1, 2)]
}
```