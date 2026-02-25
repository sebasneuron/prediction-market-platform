use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from("./src/generated");
    fs::create_dir_all(&out_dir)?;

    tonic_build::configure()
        .protoc_arg("--proto_path=proto")
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .out_dir(&out_dir)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_client(false)
        .compile_protos(&["proto/markets.proto", "proto/price.proto"], &["proto"])?;

    // building common mod.rs file with all module names

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
            // skip the mod.rs file itself
            continue;
        }
        writeln!(mod_rs, "pub mod {};", module_name)?;
    }

    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
