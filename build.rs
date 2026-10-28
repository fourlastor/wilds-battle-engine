use std::{env, fs, path::PathBuf};

fn main() {
    let moves_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../moves");
    println!("cargo:rerun-if-changed={}", moves_dir.display());
    let mut files: Vec<_> = fs::read_dir(&moves_dir)
        .expect("moves directory")
        .map(|entry| entry.expect("move entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lua"))
        .collect();
    files.sort_by(|a, b| a.file_stem().cmp(&b.file_stem()));
    // Each file is one move named after its ID; the catalog looks them up by binary search.
    let mut source = String::from("pub(super) static MOVES: &[(&str, &str)] = &[\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let id = path
            .file_stem()
            .unwrap()
            .to_str()
            .expect("UTF-8 move file name");
        let path = fs::canonicalize(&path).expect("move file");
        source.push_str(&format!(
            "    ({id:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("builtin_moves.rs"),
        source,
    )
    .expect("generated built-in catalog");
}
