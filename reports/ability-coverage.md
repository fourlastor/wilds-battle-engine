# Sword/Shield ability coverage

`ability-coverage.csv` inventories the 267 main-series abilities in the local PokeAPI CSV with generation <= 8. It includes older abilities even if their species is unavailable in Sword/Shield. It excludes later generations and non-main-series abilities.

- `IMPLEMENTED`: native core battle behavior exists. Source-inventoried entries are not a claim of complete cartridge parity.
- `PARTIAL`: native behavior exists, but a known component is missing.
- `MISSING`: no native behavior; references limited to copy/suppression metadata or move eligibility do not count.

The verification column records targeted regression tests as the implementation proceeds. Generic host-supplied rules do not count as native implementations. Overworld effects, Z-Moves, Dynamax and new item content are outside this battle-engine task; abilities depending on these remain explicitly incomplete rather than receiving placeholder behavior.

Rules reference: [Pokemon Showdown abilities at 942cda71](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/data/abilities.ts), with Generation 8 overrides where applicable. Identity source: `~/Documents/pokeapi/data/v2/csv/abilities.csv`.
