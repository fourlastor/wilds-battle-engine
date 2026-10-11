# Move catalog

Each move has a standalone `<Name>.lua` file in this directory. `Name` is the
canonical slug from `pokewilds-next/pokemon_data/assets/moves/<ID>.json`;
`HumanName` supplies its display name. All 588 slugs in that source are unique.

The 572 entries in the [Saved moves sheet](https://docs.google.com/spreadsheets/d/1e9lPFCqyuwpX6R6s69HXoFUXedoIAD8SdtSbswY48Q8/edit?gid=1664051581)
comprise 539 newly added files, 32 existing files, and the engine-provided Struggle.
Together with the nine existing moves outside the sheet, the runtime catalog has
581 moves: 580 files here and Struggle in `src/system_moves.lua` inside the crate.
The 22 original filenames containing underscores have been renamed with hyphens.

New move IDs equal their filename slugs. Original runtime IDs retain underscores
where applicable (for example, `solar_beam` lives in `solar-beam.lua`) so existing
setups continue to work. Original move behavior and parameters are preserved.
Move Lab replaces and downloads edited files using the canonical filenames.

## Behavior and incomplete mechanics

New files keep the source JSON's names and numeric parameters. Actual move behavior
takes precedence over conflicting sheet descriptions. PokeAPI metadata and effects
were checked by numeric ID; special behavior was cross-checked against the pinned
Pokémon Showdown reference recorded in [catalog.json](catalog.json).
For example, Snipe Shot has an increased critical-hit rate, Jaw Lock traps both
participants, and No Retreat raises five stats and records its one-use trap.

Of the 539 new definitions, 445 implement their move-specific behavior through the
available API and 94 are partial. Each partial file has an `INCOMPLETE` comment,
an `incomplete` flag, and specific limitations in the manifest. Sky Drop and Stuff
Cheeks fail explicitly because their defining mechanics cannot run through this API.
Other partial moves execute the supported portion of their effects.

The main gaps are switching and benches, held-item activation, ability execution,
Z-Move and Dynamax eligibility, conditional action ordering, and complete terrain
interactions. Trapping can be recorded but cannot cause or prevent an actual switch.
These definitions use the current engine's general damage, status and rounding rules;
the coverage count does not imply full official battle parity. No Rust mechanics
were added for these gaps.

The manifest maps every sheet ID and row to its slug, display name, runtime ID,
file, status and limitations. To list partial moves:

```sh
jq -r '.moves[] | select(.status == "partial") | "\(.id)\t\(.slug)\t\(.limitations | join("; "))"' moves/catalog.json
```

## Validation

`cargo test --workspace --offline` loads both embedded and disk catalogs, checks
all sheet identities and filenames, and executes every shipped move over several
turns with multiple participants, items and statuses. Focused regression cases
check representative special behaviors and explicit failures. They do not prove
every interaction with every other move.

Move Lab builds the same files. Its block parity suite covers the definitions it
can reconstruct as blocks; Lua-only moves remain available to the engine and are
covered by the Rust runtime tests.
