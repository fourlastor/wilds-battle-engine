# Wilds battle engine

`wilds-battle-engine` is the Rust battle simulation. Move definitions are loaded
from Lua; `MoveCatalog::builtin()` loads the 29 moves currently defined under
`pokewilds-next/battle/moves`. The host supplies choices for both sides.
The Rust API uses enums for sides, Pokémon types, stats, statuses, and weather.
Rust exposes typed Lua constants for Pokémon types, stats, statuses, weather,
categories, targets, effects, and special accuracy modes. Move IDs and display
text remain strings. `MoveCatalog::from_lua` installs these constants before
evaluating move definitions; values from the wrong enum family are rejected.

```lua
{ id="toxic", name="Toxic", type=Type.Poison,
  category=Category.Status, pp=10, accuracy=0.9,
  effects={{kind=Effect.Status, status=Status.BadlyPoisoned}} }
```

Five additional moves from the [Saved moves sheet](https://docs.google.com/spreadsheets/d/1e9lPFCqyuwpX6R6s69HXoFUXedoIAD8SdtSbswY48Q8/edit?gid=1664051581)
use Lua callbacks: Dream Eater, False Swipe, Triple Kick, Explosion, and Hyper Beam.
Each callback receives read-only `user` and `target` snapshots (`hp`, plus the
target's status) and a battle API. Calls such as `ctx:damage` yield to Rust;
Rust applies the operation and resumes Lua with its result. Damage accepts
`accuracy` for an individual hit, `drain`, and `min_target_hp` for moves that
cannot KO. A hit result has `hit` and `damage` fields.
For example:

```lua
script=function(ctx)
  if ctx.target.status ~= ctx.Status.Asleep then return ctx:fail() end
  local result = ctx:damage(100, {drain=0.5})
  if not result.hit then return end
end
```

```rust
use wilds_battle_engine::{ActionSelection, AdvanceStatus, Battle, MoveCatalog, Pokemon};

let catalog = MoveCatalog::builtin()?;
let mut battle = Battle::new(7, [
    vec![Pokemon::new("Ally", vec!["tackle".into()])],
    vec![Pokemon::new("Foe", vec!["splash".into()])],
], catalog)?;
loop {
    let result = battle.advance()?;
    for event in result.events { /* present the event */ }
    match result.status {
        AdvanceStatus::Awaiting(prompt) => {
            let choice = &prompt.choices[0]; // supplied by the host's player or AI
            battle.set_response(ActionSelection {
                prompt_id: prompt.id, choice_id: choice.id,
            })?;
        }
        AdvanceStatus::End { .. } => break,
    }
}
# Ok::<(), wilds_battle_engine::BattleError>(())
```

The current port covers move metadata and battle mechanics for the original 29 moves,
including turn order, accuracy, damage, status, weather, charging, repeated hits,
consecutive moves, and outcome resolution. The Rust RNG is seeded, so a Rust
run is reproducible; it does not reproduce Godot's RNG stream. Tests exercise
each move and selected interactions. This is not yet a verified event-for-event
match against a running C# battle. The FFI crate remains a placeholder while
the Rust API stabilizes.

Run `cargo test --workspace --offline` and
`cargo clippy --workspace --all-targets --offline -- -D warnings`.
