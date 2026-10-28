# Sword/Shield ability coverage

`ability-coverage.csv` inventories the 267 main-series abilities in the local PokeAPI CSV with generation <= 8. It includes older abilities even if their species is unavailable in Sword/Shield. It excludes later generations and non-main-series abilities.

- `IMPLEMENTED`: native core battle behavior exists. Source-inventoried entries are not a claim of complete cartridge parity.
- `PARTIAL`: native behavior exists, but a known component is missing.
- `MISSING`: no native behavior; references limited to copy/suppression metadata or move eligibility do not count.

The verification column records targeted regression tests as the implementation proceeds. Generic host-supplied rules do not count as native implementations. Overworld effects, Z-Moves, Dynamax and new item content are outside this battle-engine task; abilities depending on these remain explicitly incomplete rather than receiving placeholder behavior.

Rules reference: [Pokemon Showdown abilities at 942cda71](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/data/abilities.ts), with Generation 8 overrides where applicable. Identity source: `~/Documents/pokeapi/data/v2/csv/abilities.csv`.

## Current result

| Status | Count |
| --- | ---: |
| IMPLEMENTED | 265 |
| PARTIAL | 0 |
| MISSING | 2 |
| Total | 267 |

The initial audit found 67 implemented, 25 partial and 175 missing abilities.
This implementation pass completed the 25 partial entries and added native battle
behavior for 172 missing entries. Each ability change was tested and committed
before moving on. The CSV contains an individual verification reference per entry.

The remaining entries are retained in the denominator rather than counted as
implemented through names or placeholders:

| Ability | Remaining requirement |
| --- | --- |
| Illuminate | Gen8 overworld encounter-rate behavior; it has no Gen8 battle effect. |
| Honey Gather | Post-battle Honey generation, outside battle resolution and the no-new-items scope. |

## Integration requirements and verification

- Form abilities use canonical form slugs and **host-computed** `Pokemon.forms`
  presets. The host must provide those presets; the engine does not contain a
  species-stat database. Power Construct also requires `max_hp` on
  `zygarde-complete`. Missing presets emit an explicit diagnostic and do not
  silently invent stats.
- Illusion supplies separate appearance data and reveal events. Hosts must render
  that appearance and filter full battle state for opponents as appropriate.
- Run Away requires the host to enable wild encounters with
  `Battle::set_wild_battle(true)` or JSON `wild_battle: true` and handle the
  distinct `Escaped` / `escaped` terminal outcome. Its guarantee works through
  traps and is disabled by ability suppression. Ordinary escape currently uses
  the classic main-series speed formula; exact Sword/Shield ordinary escape odds
  remain unverified (see README).
- Item-dependent abilities operate on the existing held-item and berry system.
  This pass did not add item content, Z-Moves, Dynamax, Arceus rules or Gen9 rules.
- Lua moves can use `hit_index`, `hit_count`, `last_hit` and `move_state` for
  Parental Bond timing. Custom scripts with one-time setup or cleanup need the
  same distinction; catalog scripts with those operations were updated.
- All **935 workspace tests** pass, including **207 ability regression tests**,
  **17 escape regression tests**, **8 damage regression tests**, **13 targeting/spread regression tests**,
  the complete type-chart matrix and FFI serialization coverage. `cargo clippy --workspace --all-targets -- -D warnings`
  also passes. Inventory IDs/slugs exactly match the 267 local PokeAPI entries,
  and all test names cited by the CSV resolve to actual tests.

`IMPLEMENTED` records native core mechanics with regression coverage or existing
source inventory, not exhaustive validation of every multi-ability interaction
or complete cartridge parity. Host form data and presentation integration remain
host responsibilities.
