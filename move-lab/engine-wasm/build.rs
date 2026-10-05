// Emscripten link settings for the browser build. Other targets get a plain binary.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("emscripten") {
        return;
    }
    let exported = [
        "_main",
        "_malloc",
        "_free",
        // The engine's own C ABI, unchanged.
        "_wbe_create",
        "_wbe_destroy",
        "_wbe_advance",
        "_wbe_respond",
        "_wbe_catalog",
        "_wbe_last_error",
        "_wbe_free_string",
        // Read-only extras for the editor.
        "_mlab_catalog",
        "_mlab_state",
    ]
    .join(",");
    for arg in [
        "-sMODULARIZE=1",
        "-sEXPORT_ES6=1",
        "-sENVIRONMENT=web,worker,node",
        "-sALLOW_MEMORY_GROWTH=1",
        "-sFORCE_FILESYSTEM=1",
        "-sEXPORTED_RUNTIME_METHODS=FS,UTF8ToString,stringToNewUTF8",
    ] {
        println!("cargo:rustc-link-arg-bins={arg}");
    }
    println!("cargo:rustc-link-arg-bins=-sEXPORTED_FUNCTIONS={exported}");
}
