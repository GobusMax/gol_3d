use pollster::FutureExt;
use wgpu_test::run;
fn main() {
    run().block_on();
}
