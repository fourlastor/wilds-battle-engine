use std::{env, fs, path::PathBuf};

/// Embeds every `<id>.lua` file of `directory` as `(id, source)` pairs sorted by ID, which the
/// catalog looks up by binary search.
fn embed(directory: &str, output: &str) {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(directory);
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.expect("script entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lua"))
        .collect();
    files.sort_by(|a, b| a.file_stem().cmp(&b.file_stem()));
    let mut source = String::from("pub(super) static SCRIPTS: &[(&str, &str)] = &[\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let id = path
            .file_stem()
            .unwrap()
            .to_str()
            .expect("UTF-8 script file name");
        let path = fs::canonicalize(&path).expect("script file");
        source.push_str(&format!(
            "    ({id:?}, include_str!({:?})),\n",
            path.display().to_string()
        ));
    }
    source.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join(output),
        source,
    )
    .expect("generated built-in catalog");
}

fn main() {
    embed("../../moves", "builtin_moves.rs");
    embed("../../items", "builtin_items.rs");
}
