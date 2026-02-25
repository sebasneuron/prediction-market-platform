use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from("./src/proto_types");

    fs::create_dir_all(&out_dir)?;

    let mut config = prost_build::Config::new();

    config
        .protoc_arg("--proto_path=proto")
        .protoc_arg("--experimental_allow_proto3_optional")
        .out_dir(&out_dir)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "proto/ws_server/market_price.proto",
                "proto/ws_server/order_book.proto",
                "proto/ws_server/common.proto",
            ],
            &["proto"],
        )?;

    let entries = fs::read_dir(&out_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false));

    // for creating mod.rs with all modules (for allowing access to all crates)
    let mut mod_rs = File::create(out_dir.join("mod.rs"))?;
    for entry in entries {
        let file_name = entry.file_name();
        let module_name = file_name
            .to_string_lossy()
            .trim_end_matches(".rs")
            .to_string();
        if module_name == "mod" {
            continue;
        }
        writeln!(mod_rs, "pub mod {};", module_name)?;
    }

    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
