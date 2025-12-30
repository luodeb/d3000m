// build.rs
fn main() {
    println!("cargo:rustc-link-arg=-T");
    println!("cargo:rustc-link-arg=target/aarch64-unknown-none-softfloat/release/linker_axplat-aarch64-d3000m-n80-laptop.lds");
    println!("cargo:rustc-codegen-options=-C target-cpu=cortex-a76");
}
