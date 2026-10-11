# Sword/Shield status rules

The status audit targets Sword/Shield (including its expansions). These rules
supersede the original PokeWilds compatibility behavior where it differs.
They do not change the source move names, PP, power or accuracy parameters.

References: Pokémon Showdown's pinned [status conditions](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/data/conditions.ts)
and [abilities](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/data/abilities.ts),
with [Gen 8 overrides](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/data/mods/gen8/abilities.ts).

## Freeze

- A damaging Fire-type hit thaws its surviving target before its pending action.
  The actual move type is used, including Weather Ball. Misses, immunity,
  Substitute interception and Fire-type status moves do not thaw.
- Scald, Steam Eruption and Scorching Sands also thaw their targets.
- Ice types, active Magma Armor and effective sunshine prevent new freezes.
  Sunshine does not cure an existing freeze; Cloud Nine and Air Lock suppress
  its freeze prevention. Magma Armor cures existing freeze when active.
- Frozen Pokémon have a 20% chance to thaw when attempting an ordinary move.
  Flame Wheel, Sacred Fire, Flare Blitz, Fusion Flare, Scald, Steam Eruption,
  Burn Up, Pyro Ball and Scorching Sands thaw their user before execution.
  Burn Up requires its user to still be Fire-type for this automatic thaw.

Regression tests are in `crates/wilds-battle-engine/tests/status_rules.rs`.

## Burn

- Residual damage is 1/16 maximum HP, rounded down with a minimum of 1.
  Heatproof halves that damage; Magic Guard prevents it.
- Physical move damage is halved, except Facade or an active Guts user.
  Guts boosts Attack for any major status; Flare Boost increases special move
  power while burned.
- Fire types cannot be burned. Water Veil and Water Bubble prevent and cure
  burns while active, including after acquiring the ability.
- This audit implements status interactions of these abilities; their unrelated
  type-based damage modifiers are outside this status audit.

## Poison and toxic poison

- Ordinary poison deals floor(maximum HP / 8), minimum 1. Toxic deals
  floor(maximum HP / 16), minimum 1, multiplied by a stage from 1 to 15.
  The counter resets on switching; the status remains toxic poison.
- Magic Guard prevents damage. Poison Heal replaces it with 1/8-HP healing,
  blocked by Heal Block; Toxic's counter still advances with either ability.
- Immunity and Pastel Veil prevent and cure poison; Pastel Veil also protects
  active allies. Corrosion bypasses Poison/Steel status immunity, but cannot
  make Poison-type attacks damage Steel or make Toxic Spikes poison Steel.
- A Poison-type Toxic user bypasses accuracy and semi-invulnerability.
- Toxic Boost increases physical move power while poisoned. Merciless forces
  critical hits against poisoned targets unless critical-hit immunity applies.

## Paralysis

- Speed is halved after other Speed modifiers. Quick Feet instead grants its
  1.5× Speed boost, including with other major statuses.
- The 25% chance to lose an action remains with Quick Feet, and is checked
  after flinch, confusion and infatuation. Blocked actions spend no PP.
- Electric types are immune; Ground types resist Electric paralysis moves.
  Active Limber prevents and cures paralysis.
