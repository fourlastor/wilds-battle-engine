# Move Lab

A web page for making battle moves without writing code. You stack blocks, Move Lab writes the
`moves/<id>.lua` file, and a test battle beside the blocks runs it in the real battle engine,
compiled to WebAssembly.

Move Lab is a separate project that happens to live in this repository. It depends on the engine
by path; the engine does not know it exists, and nothing under `crates/` was changed for it.

## Run it

You need Node 24+, Rust (stable, via rustup) and git.

```bash
cd move-lab
npm install
npm run setup:emsdk    # once: downloads Emscripten into .emsdk/ (git-ignored, about 1.7 GB)
npm run build:engine   # compiles the engine to public/engine/wbe.wasm
npm run dev
```

Run `npm run build:engine` again whenever the engine changes. Move files are picked up without
rebuilding: the page bundles `../moves/*.lua` as text.

`npm run build` writes a static site to `dist/` that can be hosted anywhere (it uses relative
paths). `npm test` runs the checks described below.

## How it works

**The engine in the browser.** `engine-wasm/` is a small Rust crate, outside the engine's Cargo
workspace, that builds for `wasm32-unknown-emscripten`. It compiles the engine's own C interface
(`crates/wilds-battle-engine-ffi/src/lib.rs`) from source, so the page calls the same `wbe_create`,
`wbe_advance` and `wbe_respond` the game does. Move files are written to Emscripten's in-memory
file system and loaded with the engine's normal directory loader. Two extra read-only functions,
`mlab_catalog` and `mlab_state`, expose what the public Rust API already offers (move metadata,
stat stages, weather) so the editor can list moves and draw the battle.

**Blocks to Lua.** `src/blocks/definitions.ts` defines the blocks and `src/blocks/generate.ts` turns
a workspace into a move file. The engine accepts either an `effects` list or a `script` function,
never both, so:

- a stack made only of blocks the effects list can express becomes a plain move (`effects = {…}`);
- as soon as one block needs a script (logic, messages, multi-turn control and so on), the whole
  move becomes one;
- blocks the chosen form cannot express are flagged on the block instead of being dropped, and the
  message names what made the move a script. Status, weather, protect, trapping, multi-hit, fixed,
  level and one-hit-KO damage, and stat changes on the target exist only as effects today, so they
  cannot be combined with logic until the engine gains script calls for them.

The engine applies a plain move's effects to the one Pokémon the move is used on, so two cases are
written as scripts to keep the blocks true to their words: a move used on "everyone else" (only a
script's `ctx:damage` reaches them all), and "heal the user" in a move aimed at someone else.
"protect the user" is only accepted in a move used on the user.

**All blocks.** The "All blocks" link in the top bar (address `#blocks`) opens a page listing every
block with what it does and the line it writes in each form. Nothing on it is written twice: the
explanations are the blocks' tooltips (`TIPS` in `src/blocks/definitions.ts`), the lines come from
the generator, and the pictures are the real blocks, drawn once by Blockly off screen and copied
in. `src/blocks/reference.ts` only adds the order, fuller examples and a few longer notes. The
tooltips describe engine behaviour, so check them against `crates/wilds-battle-engine/src/engine.rs`
when the engine changes.

**Opening built-in moves.** Plain built-ins are converted to blocks from the engine's own
description of them. The twelve scripted ones are rebuilt by hand in `src/blocks/library.ts`.

**The test battle.** `src/bench/bench.ts` drives a battle and builds the log. The copy of the move
that runs in the test battle carries invisible marker messages (`ctx:message("\1…")`) before each
block, which is how the log and the glowing blocks know what ran. The downloaded file has no
markers. Because the engine's dice are seeded, editing a move restarts the battle on the same seed
and repeats your choices, so only the edit changes the outcome.

## Checks

- `npm run test:engine` loads the wasm engine in Node and plays a short battle.
- `npm run test:blocks` rebuilds all 41 built-in moves from blocks and plays about 5,900 seeded
  battles, comparing every event and the final state against the original Lua files, for both the
  clean and the marked-up output. `PARITY_BREAK=tackle npm run test:blocks` proves the check bites.
- `npm run test:reference` checks that every block of the palette is on the "All blocks" page with
  an explanation and a line, and plays short battles to confirm that blocks which say "the user"
  act on the user whoever the move is aimed at.

## Publishing to Cloudflare

The site is static, so it is published as a Cloudflare Worker that only serves files
(`wrangler.jsonc` points at `dist/`; there is no Worker script). The build needs Rust and
Emscripten, so it runs in GitHub Actions, where both are cached, and the result is handed to
Cloudflare.

`.github/workflows/move-lab.yml` installs Rust and the pinned Emscripten (`emsdk-version`), builds
the engine, runs the checks and builds the site, keeping `dist/` as an artifact. Pushes to `main`
are then published, once the repository has these two secrets (Settings > Secrets and variables >
Actions):

- `CLOUDFLARE_ACCOUNT_ID`: your account ID, from the Cloudflare dashboard;
- `CLOUDFLARE_API_TOKEN`: an API token made from the "Edit Cloudflare Workers" template.

Until both exist, the workflow only builds and tests. The first publish creates a Worker called
`move-lab`, reachable at `https://move-lab.<your-subdomain>.workers.dev`. Rename it or attach your
own domain in `wrangler.jsonc`.

To publish from your own machine instead, log in once with `npx wrangler login`, then run
`npm run deploy`.

`public/_headers` tells Cloudflare to cache the hashed files under `/assets/` permanently. The page
and the engine are revalidated on every load, so a new build shows up straight away.

## Layout

```
engine-wasm/      Rust crate that builds the engine for the browser
scripts/          emsdk setup, engine build, and the checks
src/engine/       loads the wasm module and wraps the C interface
src/blocks/       block definitions, palette, Lua generator, built-in moves as blocks, block reference
src/bench/        battle runner, log and trace
src/ui/           the page: editor, battle panel, setup dialog, "All blocks" page
src/data/         bundled move files and Pokémon presets for the setup form
```

## Known gaps

- The battle cannot start in a chosen weather: the engine's setup has no weather field. Use Sunny
  Day or Sandstorm in the battle instead.
- A script may yield 64 operations per turn, and each marker is one. If markers alone push a move
  over the limit, Move Lab runs it without them and turns block highlighting off for that move.
- Moves and the battle setup are saved in the browser's local storage only.
- Sprites are placeholders.
