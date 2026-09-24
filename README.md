# Wilds battle engine

`wilds-battle-engine` is the Rust battle simulation. Move definitions are loaded
from Lua; `MoveCatalog::builtin()` loads the 29 moves currently defined under
`pokewilds-next/battle/moves`. The host supplies choices for both sides.
The Rust API uses enums for sides, Pokémon types, stats, statuses, and weather.
Lua uses names for these values; unknown names are rejected when the catalog loads.

Five additional moves from the [Saved moves sheet](https://docs.google.com/spreadsheets/d/1e9lPFCqyuwpX6R6s69HXoFUXedoIAD8SdtSbswY48Q8/edit?gid=1664051581)
use Lua callbacks: Dream Eater, False Swipe, Triple Kick, Explosion, and Hyper Beam.
Each callback receives read-only `user` and `target` snapshots (`hp`, plus the
target's status) and returns an ordered list of actions. Rust validates and
executes those actions, including damage, drain, self fainting, and recharge.
Damage actions may set `accuracy` for an individual hit, `stop_on_miss` for
sequences, and `min_target_hp` for moves that cannot KO. Scripts run before
state changes for that move; their returned actions execute in order.
For example:

```lua
script=function(ctx)
  if ctx.target.status ~= "asleep" then return {{kind="fail"}} end
  return {{kind="damage", power=100, drain=0.5}}
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
