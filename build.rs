fn main() {
    embuild::espidf::sysenv::output();

    // Copy partitions.csv to the ESP-IDF build output directory so that the
    // CMake build can find it when CONFIG_PARTITION_TABLE_CUSTOM is enabled.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let src = std::path::Path::new("partitions.csv");
    let dst = std::path::Path::new(&out_dir).join("partitions.csv");
    if src.exists() {
        std::fs::copy(src, dst).expect("Failed to copy partitions.csv to OUT_DIR");
    }

    println!("cargo:rerun-if-changed=partitions.csv");
}
