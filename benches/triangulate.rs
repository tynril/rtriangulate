#[macro_use]
extern crate bencher;
use bencher::Bencher;

extern crate rtriangulate;
use rtriangulate::{Point, triangulate};

#[cfg_attr(rustfmt, rustfmt_skip)]
const POINTS: [Point; 100] = [
    Point {x:   1.0, y: 117.0}, Point {x:   3.0, y: 438.0}, Point {x:   3.0, y: 524.0},
    Point {x:  10.0, y: 253.0}, Point {x:  10.0, y: 515.0}, Point {x:  14.0, y: 479.0},
    Point {x:  27.0, y: 257.0}, Point {x:  28.0, y:  16.0}, Point {x:  34.0, y: 452.0},
    Point {x:  48.0, y: 201.0}, Point {x:  55.0, y: 501.0}, Point {x:  71.0, y: 216.0},
    Point {x:  83.0, y: 304.0}, Point {x:  85.0, y: 657.0}, Point {x:  93.0, y:  57.0},
    Point {x: 104.0, y: 564.0}, Point {x: 123.0, y: 163.0}, Point {x: 145.0, y: 460.0},
    Point {x: 147.0, y: 343.0}, Point {x: 149.0, y: 624.0}, Point {x: 151.0, y: 550.0},
    Point {x: 169.0, y: 480.0}, Point {x: 177.0, y: 397.0}, Point {x: 188.0, y:  18.0},
    Point {x: 192.0, y: 358.0}, Point {x: 196.0, y: 270.0}, Point {x: 208.0, y: 392.0},
    Point {x: 216.0, y: 315.0}, Point {x: 230.0, y: 616.0}, Point {x: 269.0, y:  76.0},
    Point {x: 273.0, y: 333.0}, Point {x: 278.0, y: 644.0}, Point {x: 286.0, y: 420.0},
    Point {x: 321.0, y: 161.0}, Point {x: 349.0, y: 365.0}, Point {x: 354.0, y:  51.0},
    Point {x: 362.0, y: 123.0}, Point {x: 376.0, y: 660.0}, Point {x: 385.0, y: 352.0},
    Point {x: 391.0, y: 160.0}, Point {x: 392.0, y: 413.0}, Point {x: 400.0, y: 611.0},
    Point {x: 409.0, y: 380.0}, Point {x: 420.0, y: 354.0}, Point {x: 442.0, y: 545.0},
    Point {x: 449.0, y: 209.0}, Point {x: 459.0, y: 327.0}, Point {x: 463.0, y: 458.0},
    Point {x: 467.0, y: 593.0}, Point {x: 474.0, y: 254.0}, Point {x: 478.0, y: 469.0},
    Point {x: 478.0, y: 602.0}, Point {x: 491.0, y: 221.0}, Point {x: 491.0, y: 493.0},
    Point {x: 503.0, y: 142.0}, Point {x: 503.0, y: 635.0}, Point {x: 521.0, y: 488.0},
    Point {x: 527.0, y: 335.0}, Point {x: 534.0, y: 269.0}, Point {x: 535.0, y: 423.0},
    Point {x: 556.0, y: 570.0}, Point {x: 574.0, y: 410.0}, Point {x: 579.0, y: 393.0},
    Point {x: 591.0, y: 439.0}, Point {x: 607.0, y: 266.0}, Point {x: 620.0, y:  18.0},
    Point {x: 631.0, y: 221.0}, Point {x: 635.0, y: 206.0}, Point {x: 637.0, y: 598.0},
    Point {x: 650.0, y: 243.0}, Point {x: 662.0, y: 598.0}, Point {x: 662.0, y: 622.0},
    Point {x: 681.0, y: 230.0}, Point {x: 686.0, y: 241.0}, Point {x: 699.0, y: 576.0},
    Point {x: 702.0, y: 647.0}, Point {x: 703.0, y:  14.0}, Point {x: 706.0, y: 383.0},
    Point {x: 712.0, y:  70.0}, Point {x: 717.0, y: 443.0}, Point {x: 726.0, y: 349.0},
    Point {x: 745.0, y: 616.0}, Point {x: 749.0, y: 282.0}, Point {x: 756.0, y: 310.0},
    Point {x: 761.0, y:  88.0}, Point {x: 791.0, y:   4.0}, Point {x: 800.0, y:  72.0},
    Point {x: 813.0, y: 565.0}, Point {x: 817.0, y: 100.0}, Point {x: 834.0, y: 196.0},
    Point {x: 844.0, y: 247.0}, Point {x: 847.0, y:   4.0}, Point {x: 856.0, y: 299.0},
    Point {x: 867.0, y:  94.0}, Point {x: 871.0, y: 509.0}, Point {x: 873.0, y: 111.0},
    Point {x: 875.0, y: 468.0}, Point {x: 877.0, y:  86.0}, Point {x: 878.0, y: 301.0},
    Point {x: 891.0, y:  23.0},
];

fn bench_three_points(bench: &mut Bencher) {
    let points = &POINTS[..3];
    bench.iter(|| triangulate(points));
}

fn bench_ten_points(bench: &mut Bencher) {
    let points = &POINTS[..10];
    bench.iter(|| triangulate(points));
}

fn bench_twenty_points(bench: &mut Bencher) {
    let points = &POINTS[..20];
    bench.iter(|| triangulate(points));
}

fn bench_thirty_points(bench: &mut Bencher) {
    let points = &POINTS[..30];
    bench.iter(|| triangulate(points));
}

fn bench_forty_points(bench: &mut Bencher) {
    let points = &POINTS[..40];
    bench.iter(|| triangulate(points));
}

fn bench_fifty_points(bench: &mut Bencher) {
    let points = &POINTS[..50];
    bench.iter(|| triangulate(points));
}

fn bench_sixty_points(bench: &mut Bencher) {
    let points = &POINTS[..60];
    bench.iter(|| triangulate(points));
}
fn bench_seventy_points(bench: &mut Bencher) {
    let points = &POINTS[..70];
    bench.iter(|| triangulate(points));
}

fn bench_eighty_points(bench: &mut Bencher) {
    let points = &POINTS[..80];
    bench.iter(|| triangulate(points));
}

fn bench_ninety_points(bench: &mut Bencher) {
    let points = &POINTS[..90];
    bench.iter(|| triangulate(points));
}

fn bench_one_hundred_points(bench: &mut Bencher) {
    let points = &POINTS[..100];
    bench.iter(|| triangulate(points));
}

benchmark_group!(benches,
                 bench_three_points,
                 bench_ten_points,
                 bench_twenty_points,
                 bench_thirty_points,
                 bench_forty_points,
                 bench_fifty_points,
                 bench_sixty_points,
                 bench_seventy_points,
                 bench_eighty_points,
                 bench_ninety_points,
                 bench_one_hundred_points);
benchmark_main!(benches);
