# Move catalog

Each move has a standalone `<Name>.lua` file in this directory. `Name` is the
canonical slug from `pokewilds-next/pokemon_data/assets/moves/<ID>.json`;
`HumanName` supplies its display name. All 588 slugs in that source are unique.
Moves without local source JSON use PokeAPI names and parameters, checked against
the pinned Gen 8 behavior reference; individual references are in the manifest.
The move cutoff is Sword/Shield, including The Isle of Armor and The Crown Tundra.
Legends: Arceus moves and battle rules are outside the project scope. Older moves
remain included even when they cannot be used in Sword/Shield itself.

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
186 further moves through Sword/Shield and the nine original moves outside the sheet,
the runtime catalog has 754 moves: 753 files here and Struggle in
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
Z-Moves and Max Moves remain excluded, including from the older Lua API examples.

Voluntary switches and faint replacements now use host-selected bench choices.
Source-linked traps now prevent switches and end when their source leaves or faints.
Circle Throw and Dragon Tail now force a random eligible bench replacement.
U-turn, Volt Switch, Flip Turn and Parting Shot pause for a host-selected replacement.
Pursuit intercepts voluntary and pivot switches before withdrawal, except Baton Pass.
Baton Pass transfers stages and eligible volatile effects to the host-selected
replacement before entry hazards and abilities. Transform copies battle traits and
move slots temporarily, restoring originals on withdrawal or fainting.
Sky Drop links both participants in the air, blocks carrying-related switches and
actions, and releases them on the drop, interruption, fainting or grounding.
Flying targets are carried without taking drop damage, targets at least 200 kg
cannot be lifted, and Power Herb does not skip the carry turn.
These definitions use the current engine's general damage, status and rounding rules;
the coverage count does not imply full official battle parity.
The [Sword/Shield status audit](status-rules.md) records status rules which
supersede the original PokeWilds behavior and their regression coverage.

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

Run the repeatable move inventory against the local PokeAPI clone and PokeWilds:

```sh
python3 scripts/audit_moves.py --output reports/move-coverage.csv
python3 -m unittest discover -s scripts -p 'test_*.py'
```

The audit reads PokeAPI's `data/v2/csv` files directly (the same source used to
build its API database), without Docker or network access. Defaults are
`~/Documents/pokeapi`, the sibling PokeWilds `pokemon_data/assets/moves` directory,
and this checkout. Override them with `--pokeapi-dir`, `--pokewilds-moves`, and
`--engine-root`. No source data is updated.

Numeric IDs join the sources. Existing production Lua files count as `PRESENT`,
including Struggle; manifest entries without files do not. Files absent here but
present in PokeWilds are `TO_BE_PORTED`; moves only in PokeAPI are `MISSING`.
The union retains local moves absent from an older PokeAPI snapshot. PokeWilds
names take precedence where available. Unknown Lua identities fail the audit.

The CSV includes excluded moves and their reasons: all Z-Moves (including both
physical/special variants), Max Moves (including Max Guard), Legends: Arceus,
later moves, and the Colosseum/XD Shadow moves. Older moves unavailable in
Sword/Shield, Let's Go moves, and moves using existing item mechanics remain in
scope. This is file coverage, not proof of battle-behavior correctness.
`reports/move-coverage.csv` is a generated snapshot; rerun the command after
updating any of the three sources.

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
