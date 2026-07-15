use copy_to_output::copy_to_output;
use std::env;

fn main() {
    const ASSETS: &str = "assets";

    // Re-runs script if any files in res are changed
    println!("cargo:rerun-if-changed={ASSETS}/*");
    copy_to_output(ASSETS, &env::var("PROFILE").unwrap()).expect("Could not copy");
}
