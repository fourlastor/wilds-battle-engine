use std::{env, fs, path::PathBuf};

fn main() {
    let moves_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../moves");
    println!("cargo:rerun-if-changed={}", moves_dir.display());
    let mut files: Vec<_> = fs::read_dir(&moves_dir)
        .expect("moves directory")
        .map(|entry| entry.expect("move entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lua"))
        .collect();
    files.sort();
    let mut source = String::from("return {\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let text = fs::read_to_string(&path).expect("move file");
        // Each file returns one move; wrap it to keep its local helper functions local.
        source.push_str("(function()\n");
        source.push_str(&text);
        source.push_str("\nend)(),\n");
    }
    source.push_str("}\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("builtin_moves.lua"),
        source,
    )
    .expect("generated built-in catalog");
}
