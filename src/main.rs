use pollster::FutureExt;
use wgpu_test::run;
fn main() {
    println!("Hello, world!");
    run().block_on();
}
