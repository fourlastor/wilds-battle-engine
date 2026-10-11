# Move catalog

Each move has a standalone `<Name>.lua` file in this directory. `Name` is the
canonical slug from `pokewilds-next/pokemon_data/assets/moves/<ID>.json`;
`HumanName` supplies its display name. All 588 slugs in that source are unique.
Gravity and Stockpile have no local source JSON; their names and parameters come
from PokeAPI, with the individual references recorded in the manifest.

Of the 572 entries in the [Saved moves sheet](https://docs.google.com/spreadsheets/d/1e9lPFCqyuwpX6R6s69HXoFUXedoIAD8SdtSbswY48Q8/edit?gid=1664051581),
538 remain in the active catalog. The 16 Z-Moves and 18 Max Moves are deliberately
excluded while Z-Moves and Dynamax are outside the project scope. They are listed
under `excluded_moves` in the manifest and have no loadable Lua files here.
Dynamax Cannon remains an ordinary attack; its Dynamax-only bonus is out of scope.

With Gravity, Stockpile and the nine original moves outside the sheet, the runtime
catalog has 549 moves: 548 files here and Struggle in `src/system_moves.lua`
inside the crate.
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

Of the 507 added definitions currently included, 448 implement their move-specific
behavior through the available API and 59 are partial. Each partial file has an
`INCOMPLETE` comment, an `incomplete` flag, and specific limitations in the manifest. Sky Drop and Stuff
Cheeks fail explicitly because their defining mechanics cannot run through this API.
Other partial moves execute the supported portion of their effects.

The main gaps are switching and benches, held-item activation, ability execution,
conditional action ordering, and complete terrain interactions. Trapping can be
recorded but cannot cause or prevent an actual switch.
These definitions use the current engine's general damage, status and rounding rules;
the coverage count does not imply full official battle parity. No Rust mechanics
were added for these gaps.

Stockpile stores up to three charges and tracks each actual Defense and Special
Defense increase, including stat-cap cases. Swallow consumes those charges,
heals the corresponding amount and removes only the recorded boosts.

Gravity grounds everyone for five turns, cancels airborne charging, and activates
the existing Gravity checks in Fly, Bounce, the jumping kicks, Flying Press and
Grav Apple. It leaves Dig, Dive and disappearing moves alone. Its global accuracy
bonus and preventing affected moves from being selected still require engine hooks;
affected moves currently fail when executed under Gravity.

The manifest maps included sheet IDs and rows to their slugs, display names,
runtime IDs, files, statuses and limitations; excluded entries retain their IDs,
rows and exclusion reasons. Additional moves have a null `sheet_row`.
To list partial moves:

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
