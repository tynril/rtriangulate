# rtriangulate

[![crates.io](https://img.shields.io/crates/v/rtriangulate.svg)](https://crates.io/crates/rtriangulate)
[![Build Status](https://travis-ci.org/tynril/rtriangulate.svg?branch=master)](https://travis-ci.org/tynril/rtriangulate)
[![Coverage Status](https://coveralls.io/repos/github/tynril/rtriangulate/badge.svg?branch=master)](https://coveralls.io/github/tynril/rtriangulate?branch=master)

A Rust implementation of the Delaunay triangulation algorithm presented by
[Paul Bourke](http://paulbourke.net/papers/triangulate/).

Find the crate documentation on [docs.rs](https://docs.rs/rtriangulate), or
[here on Github](https://tynril.github.io/rtriangulate).

This was developed as an exercise to get more used to Rust. As far as I know, it works, but it
might not. Also, this is a O(n<sup>1.5</sup>) (approximatively) algorithm, it's not parallelized,
and it doesn't use the GPU at all.

## Usage

Add the rtriangulate dependency to `Cargo.toml`:

```toml
[dependencies]
rtriangulate = "0.3"
```

And use the crate as such:

```rust
extern crate rtriangulate;

use rtriangulate::{TriangulationPoint, triangulate};

fn main() {
    // A list of points (which has to be sorted on x).
    // Note that you can use your own point type, just implement the rtriangulate::Point trait.
    let points = [
        TriangulationPoint::new(10.0, 50.0),
        TriangulationPoint::new(25.0, 40.0),
        TriangulationPoint::new(30.0, 40.0)
    ];

    // In case you need to sort your points:
    // points.sort_unstable_by(rtriangulate::sort_points);

    // Do the triangulation.
    let triangles = triangulate(&points).unwrap();

    println!("{:?}", triangles); // [Triangle(1, 0, 2)]
}
```

## License

MIT - See `LICENSE` file.
