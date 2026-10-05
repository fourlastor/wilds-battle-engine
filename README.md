# Wilds battle engine

`wilds-battle-engine` is the Rust battle simulation. Move definitions are loaded
from Lua; `MoveCatalog::builtin()` embeds the 41 editable files under `moves/`,
and `MoveCatalog::from_directory(path)` reads them from disk at runtime.
The host supplies choices for both sides.
The Rust API uses enums for sides, Pokémon types, stats, statuses, and weather.
Host-facing prompts contain `Choice` enum values. The only variant today is
`Choice::UseMove { id, move_id, target }`; item, switch, and run variants can
be added later without changing the prompt container.
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
Solar Beam now uses a Lua callback too, while retaining its current C# values.
Thrash is included as a sixth additional move, using its Generation V onward
duration, interruption, and confusion rules.
Charge and Thunder Shock come from the sheet as a next-move effect and a simple
Electric attack. Charge's Lua script raises Special Defense and asks Rust to
double the power of the next Electric attack; any other next move consumes the
boost without applying it.
Each callback receives read-only `user` and `target` snapshots (`hp`, plus the
user's name and target's status) and a battle API. Calls such as `ctx:damage` yield to Rust;
Rust applies the operation and resumes Lua with its result. Damage accepts
`accuracy` for an individual hit, `drain`, and `min_target_hp` for moves that
cannot KO. `typeless=true` applies damage without type effectiveness or STAB.
A hit result has `hit` and `damage` fields.
For example:

```lua
script=function(ctx)
  if ctx.target.status ~= ctx.Status.Asleep then return ctx:fail() end
  local result = ctx:damage(100, {drain=0.5})
  if not result.hit then return end
end
```

Scripts can inspect `ctx.weather`, `ctx.turn`, and `ctx.total_turns`. A move
can call `ctx:force_move(total_turns, ctx.TargetPolicy.SameTarget)` or
`ctx.TargetPolicy.RandomOpponent` to lock subsequent turns. Rust owns the lock,
chooses random targets, and spends PP only when the move is first selected.
With `RandomOpponent` each forced turn is aimed at a random living opponent;
the turn the move is selected on keeps the chosen target.
`on_interrupt` is a Lua callback for forced turns stopped before the main
script runs; the move can call `ctx:break_sequence()` and
`ctx:confuse_self()`. Scripts also have `ctx:random_int`, `ctx:message`, and
`ctx:announce` for move flow and messages.
`ctx:change_self_stat(ctx.Stat.SpDefense, 1)` and
`ctx:boost_next_move(ctx.Type.Electric, 2)` expose typed stat changes and a
one-use power modifier. Rust stores and consumes the modifier on the next move.
Rage uses `ctx:watch_hits_until_next_action()` and an `on_hit` Lua callback.
Rust runs the callback after each damaging move hit, even when one move hits
several times. The reaction expires when the user next acts; poison and weather
damage do not trigger it. Hit callbacks currently support messages and stat
changes.
Facade, Morning Sun, and Snore are additional sheet moves. Script callbacks
can inspect `ctx.user.status`, heal with `ctx:heal_self(fraction)`, and call
`ctx:flinch_target(chance)` after a hit. Snore uses the
`usable_while_asleep=true` move property.
Struggle is a catalog-provided system move. When all learned moves have 0 PP,
Rust selects it automatically. Its metadata is Normal type, while its scripted
damage is typeless; it always passes accuracy checks, targets a random opponent,
spends no PP, and recoils for one quarter of the user's maximum HP after a hit.
`advance()` collects events in order across automatic turns and stops at the
next host prompt or battle outcome.

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
                prompt_id: prompt.id, choice_id: choice.id(),
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
each move and selected interactions. The local parity suite compares one-turn
Dragon Rage, Tackle, Toxic, and Recover battles against the running C# engine,
including different damage rolls, a miss, a critical hit, status, poison damage,
healing, fainting, and battle outcome.
Broader C# event parity remains to be verified.
`wilds-battle-engine-ffi` builds as a C ABI dynamic library. Its declarations
are in `include/wilds_battle_engine.h`. `wbe_create` accepts UTF-8 JSON setup
and a move directory; `wbe_advance` returns events, the next prompt or winner,
and participant state as JSON. `wbe_respond` supplies a choice, while
`wbe_catalog` validates and lists disk move files. Each returned string must
be released with `wbe_free_string`. The PokeWilds playtest host demonstrates
the complete call sequence and the editable setup format.

`Battle::with_rng(rng, sides, catalog)` accepts a `BattleRng` implementation
for controlled draws. `Battle::new(seed, sides, catalog)` retains the default
`SeededRng`. The `damage_roll` hook accepts Godot's actual damage multiplier
without converting it through Rust's default random-float calculation. Run
`python3 parity/run.py` to compare messages, damage, status, and final HP
against the built PokeWilds C# assembly. This requires the sibling
`pokewilds-next` checkout with a current Debug assembly, Godot, and the .NET
SDK. The fixtures compare presented events and resulting state; they do not
compare every internal C# event.

Run `cargo test --workspace --offline` and
`cargo clippy --workspace --all-targets --offline -- -D warnings`.
