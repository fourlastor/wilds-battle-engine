# Move catalog

Each move has a standalone `<Name>.lua` file in this directory. `Name` is the
canonical slug from `pokewilds-next/pokemon_data/assets/moves/<ID>.json`;
`HumanName` supplies its display name. All 588 slugs in that source are unique.
Moves without local source JSON use PokeAPI names and parameters, checked against
the pinned Gen 8 behavior reference; individual references are in the manifest.

Of the 572 entries in the [Saved moves sheet](https://docs.google.com/spreadsheets/d/1e9lPFCqyuwpX6R6s69HXoFUXedoIAD8SdtSbswY48Q8/edit?gid=1664051581),
538 remain in the active catalog. The 16 Z-Moves and 18 Max Moves are deliberately
excluded while Z-Moves and Dynamax are outside the project scope. They are listed
under `excluded_moves` in the manifest and have no loadable Lua files here.
Dynamax Cannon remains an ordinary attack; its Dynamax-only bonus is out of scope.
Behemoth Blade and Behemoth Bash also implement their ordinary attacks without
the Dynamax-only damage bonus.
Aura Wheel requires Morpeko and uses Dark typing in the `morpeko-hangry` form,
otherwise Electric. The host supplies its current form; automatic Hunger Switch
form changes remain part of the unfinished ability catalog.

With seven additional support moves, 14 further moves from the local JSON data,
61 further Gen 1–8 moves and the nine original moves outside the sheet,
the runtime catalog has 629 moves: 628 files here and Struggle in
`src/system_moves.lua` inside the crate.
The 22 original filenames containing underscores have been renamed with hyphens.

New move IDs equal their filename slugs. Original runtime IDs retain underscores
where applicable (for example, `solar_beam` lives in `solar-beam.lua`) so existing
setups continue to work. Original move behavior and parameters are preserved.
Move Lab replaces and downloads edited files using the canonical filenames.

## Behavior and coverage

New files keep the source JSON's names and numeric parameters. Actual move behavior
takes precedence over conflicting sheet descriptions. PokeAPI metadata and effects
were checked by numeric ID; special behavior was cross-checked against the pinned
Pokémon Showdown reference recorded in [catalog.json](catalog.json).
For example, Snipe Shot has an increased critical-hit rate, Jaw Lock traps both
participants, and No Retreat raises five stats and records its one-use trap.

All added definitions implement their move-specific behavior through the
available API. No included move remains flagged `incomplete` in the manifest.
Z-Moves and Max Moves remain excluded.

Voluntary switches and faint replacements now use host-selected bench choices.
Source-linked traps now prevent switches and end when their source leaves or faints.
Circle Throw and Dragon Tail now force a random eligible bench replacement.
U-turn, Volt Switch, Flip Turn and Parting Shot pause for a host-selected replacement.
Pursuit intercepts voluntary and pivot switches before withdrawal.
Sky Drop links both participants in the air, blocks carrying-related switches and
actions, and releases them on the drop, interruption, fainting or grounding.
Flying targets are carried without taking drop damage, targets at least 200 kg
cannot be lifted, and Power Herb does not skip the carry turn.
These definitions use the current engine's general damage, status and rounding rules;
the coverage count does not imply full official battle parity.

Stockpile stores up to three charges and counts each successful Defense and Special
Defense change, including stat-cap cases. Contrary and Simple modify both the
initial change and its repayment. Swallow consumes those charges,
heals the corresponding amount and removes only the recorded boosts.

Gravity grounds everyone for five turns, cancels airborne charging, and activates
the existing Gravity checks in Fly, Bounce, the jumping kicks, Flying Press and
Grav Apple. It leaves Dig, Dive and disappearing moves alone. A timed engine rule
boosts ordinary accuracy checks by about 1.67 without changing stat stages or OHKO
accuracy. A flag-based engine rule prevents selecting or executing affected moves,
including Splash and Sky Drop. If every move is blocked, the engine uses Struggle.

The manifest maps included sheet IDs and rows to their slugs, display names,
runtime IDs, files, statuses and limitations; excluded entries retain their IDs,
rows and exclusion reasons. Additional moves have a null `sheet_row`.
To check for partial moves (currently no results):

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

Spikes, Toxic Spikes, Stealth Rock and Sticky Web are additional support moves.
Their parameters come from the pinned Showdown reference because no local JSON
definitions exist. Their side conditions activate on entry and Rapid Spin removes them.

Substitute is an additional support move with parameters from the same pinned
reference. Its HP pool, bypass flags and secondary-effect protection run in Rust.
