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
Each callback receives `ctx`: tables describing the battle (`ctx.user`,
`ctx.target`, and more) and a battle API. Calls such as `ctx:damage` yield to Rust;
Rust applies the operation, refreshes the tables, and resumes Lua with the result,
so what a script reads is always current. Damage accepts
`accuracy` for an individual hit, `drain`, and `min_target_hp` for moves that
cannot KO. `typeless=true` applies damage without type effectiveness or STAB.
A hit result has `hit` and `damage` fields, among others. The full list of what
a script can read and do is under [Script reference](#script-reference).
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
damage do not trigger it. Hit callbacks can show messages and change stats,
statuses, HP, marks and effects, but cannot start attacks of their own.
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

## Script reference

A scripted move is a table with `script = function(ctx) … end` instead of `effects`.
It can also have `on_hit`, `on_interrupt` and `on_turn_start` callbacks.
Everything a plain `effects` list can do, a script can do too.

For worked examples, `crates/wilds-battle-engine/tests/sheet_moves.lua` has about 120
moves written with what follows, from Fly and Gravity to Stockpile, Shell Trap and the
pledges; `tests/sheet_moves.rs` plays them.

### Move fields

| Field | Meaning |
|---|---|
| `target` | `Target.Selected` (a chosen foe, the default), `User`, `Ally`, `UserOrAlly`, `AnyOther` (a chosen foe or ally), `RandomOpponent`, `AllOpponents`, `AllAllies`, `UserAndAllies`, `AllOthers`, `All`, `Field`. A plain `effects` list acts on one Pokémon; the kinds that reach several need a script. |
| `flags` | A list of words such as `{"contact", "sound"}`. The engine only passes them on (`ctx.move.flags.contact`, `ctx.hit.contact`). |
| `hits_hidden` | Hidden places the move still reaches, such as `{Hidden.Air}`. |
| `usable_while_asleep`, `usable_while_frozen` | The second also thaws the user. |
| `on_turn_start` | Runs before anyone acts, on a turn where the move was chosen (Focus Punch, Beak Blast). |
| `on_hit` | Runs each time a move damages the user after `ctx:watch_hits_until_next_action()`. `ctx.target` is the attacker and `ctx.hit` describes the hit. |

### What a script reads

Pokémon tables: `ctx.user`, `ctx.target`, and the lists `ctx.targets` (everyone the
move is aimed at), `ctx.allies`, `ctx.foes`, `ctx.others`, `ctx.everyone`. Fainted
Pokémon are left out of the lists.

| On a Pokémon | |
|---|---|
| `name`, `species`, `level`, `hp`, `max_hp`, `gender`, `weight`, `happiness`, `ivs` | `ivs` has `hp`, `attack`, `defense`, `sp_attack`, `sp_defense`, `speed`, or is nil. |
| `status`, `confused`, `flinched`, `bound`, `trapped`, `protected`, `recharging`, `fainted` | `status` is a `Status` value or nil. |
| `types`, `:has_type(Type.X)`, `grounded` | Types as they count right now. |
| `attack`, `defense`, `sp_attack`, `sp_defense`, `speed` | With stat stages and effects applied. |
| `stages[Stat.X]`, `:stage(Stat.X)` | −6 to +6. |
| `hidden` | A `Hidden` value (`Air`, `Underground`, `Underwater`, `Vanished`) or nil. |
| `item`, `ability` | Text or nil. The engine stores them and does not know what they do. |
| `marks[name]`, `effects[name]` | See below. |
| `acted`, `selected_move` | This turn. `selected_move` has `id`, `name`, `type`, `category`, `damaging`, `priority`. |
| `damage_taken`, `hurt_this_turn`, `last_attacker`, `last_hit` | This turn. `last_hit` has `damage`, `move`, `category`, `type`, `contact`. |
| `turns_on_field`, `moves`, `used_moves[id]`, `pp[id]`, `last_move`, `last_move_failed`, `streak` | `streak` counts the turns in a row this move was used without failing. |
| `side`, `team`, `is_user`, `is_ally`, `is_foe` | `team` is the side table. |

Also: `ctx.weather`, `ctx.weather_turns`, `ctx.turn` and `ctx.total_turns` (turns of a
forced move), `ctx.battle_turn`, `ctx.environment`, `ctx.move` (`id`, `name`, `type`,
`category`, `priority`, `flags`), `ctx.field`, `ctx.user_side`, `ctx.foe_side`. A side
table has `marks`, `effects`, `pokemon` and `fainted_last_turn`.

### What a script does

`who` is a Pokémon table. Options are a table and can be left out.

| Call | Result |
|---|---|
| `ctx:damage(power, options)` | A hit on everyone the move is aimed at. Returns `hit`, `damage`, `fainted`, `critical`, `effectiveness`, `missed`, `blocked`, `immune`, `hits`. |
| `ctx:multi_hit(power, min, max, options)` | Several hits on one Pokémon, as `Effect.MultiHit`. |
| `ctx:direct_damage(who, amount, {typeless, min_target_hp})` | An exact amount, as fixed and level damage. |
| `ctx:try_hit(who, options)` | The usual protection, hiding and accuracy checks without damage. Returns true or false. The next `damage` or `multi_hit` on that Pokémon follows the answer instead of checking again. |
| `ctx:hurt(who, amount)`, `ctx:heal(who, amount)` | `amount` is HP or `{fraction = 1/8}` of max HP. Returns the HP that changed. |
| `ctx:apply_status(who, Status.X, {chance, replace, announce_failure})`, `ctx:cure_status(who, Status.X)` | True if it took. |
| `ctx:confuse(who, {chance})`, `ctx:flinch(who, {chance})` | |
| `ctx:change_stat(who, Stat.X, stages, {chance})`, `ctx:reset_stats(who)` | `change_stat` returns how far the stage really moved. |
| `ctx:protect(who)`, `ctx:break_protect(who)`, `ctx:bind(who, {min_turns, max_turns})`, `ctx:free(who)` | A Pokémon that protects itself with the same move turn after turn succeeds a third as often each time, as with `Effect.Protect`. |
| `ctx:hide(Hidden.X)`, `ctx:unhide(who)` | The user hides until it next acts. `unhide` also calls off what the Pokémon was doing. |
| `ctx:set_weather(Weather.X, turns)`, `ctx:clear_weather()` | `Sun`, `Sandstorm`, `Rain`, `Hail`. |
| `ctx:set_types(who, {Type.X})`, `ctx:set_weight(who, kg)`, `ctx:take_item(who)`, `ctx:give_item(who, item)`, `ctx:suppress_ability(who)` | |
| `ctx:chance(probability)`, `ctx:effect_chance(probability)`, `ctx:random_int(min, max)` | `effect_chance` is for a move's extra effects: an `EffectChance` rule can multiply it. |
| `ctx:reached(who)` | Whether this use of the move got through to a Pokémon (the target if left out): the result of the hit already made on it, or else `try_hit` now. Use it to hold an effect back until the move connects. |
| `ctx:steal_boosts(from, to)`, `ctx:steal_item(from, to)`, `ctx:lose_type(who, Type.X)` | Shorthands built from the calls above. |
| `ctx:opposite_genders(a, b)`, `ctx:share_type(a, b)`, `ctx:type_number(0..15)` | `type_number` uses Hidden Power's order. |
| `who:raised_stages()`, `who:used_all_other_moves(id)` | Read-only helpers on a Pokémon. |
| `ctx:add_payout(amount)` | Money for the user's side, read by the host with `Battle::payout`. |
| `ctx:hurry()` | Everyone else who chose this move acts next (Round). |

Damage options: `accuracy`, `never_miss`, `ignore_protect`, `ignore_evasion`,
`hits_hidden` (a list of `Hidden` values, or `true`), `drain`, `recoil`,
`min_target_hp`, `typeless`, `high_crit`, `always_crit`, `target` (one Pokémon),
`type`, `also_type`, `effective` (`{[Type.Water] = 2}` replaces the type chart
against the listed types), `category`, `attack_from` (whose attacking stat is used),
`attack_stat`, `defense_stat`, `ignore_stages`. `try_hit` takes the first five.

Power and amounts of HP drop any fraction, so `ctx:damage(150 * ctx.user.hp / ctx.user.max_hp)`
works as it does in the games. A hook may run up to 256 operations.

### Marks and timed effects

A mark is a named number that one move leaves for another, on a Pokémon, a side or
the field. Marks on a Pokémon are gone when it faints.

```lua
ctx:mark(ctx.user, "minimized")                             -- Minimize
if ctx.target.marks.minimized then power = power * 2 end    -- Stomp
ctx:mark(ctx.field, "echo", chain + 1, 2)                   -- a counter that lasts through next turn
```

An effect is a mark with a duration and rules that the engine enforces. `turns`
counts the turn it starts on; without it the effect lasts until `ctx:end_effect`.
Starting an effect ends any other effect of the same `group` in the same place.

```lua
ctx:start_effect(ctx.user.team, "reflect", {turns = 5, end_message = "Reflect wore off!",
  rules = {{kind = Rule.DamageTaken, category = Category.Physical, factor = 0.5, not_on_crit = true}}})
ctx:end_effect(ctx.target.team, "reflect")                  -- Brick Break
```

| Rule | Options |
|---|---|
| `DamageEachTurn`, `HealEachTurn` | `fraction` of max HP, `message` (`{name}` is the Pokémon); `except_types` for damage |
| `DrainEachTurn` | `fraction`, `to` (who gets the HP), `message` |
| `DamageTaken`, `DamageDealt` | `factor`, and optionally only for a `category` or `type`; `not_on_crit` for damage taken |
| `StatMultiplier` | `stat`, `factor` |
| `BlockStatus` | `statuses` (all if left out), `confusion`. Does not stop what a Pokémon does to itself. |
| `BlockStatDrops`, `Grounded`, `Trapped`, `Endure` | |
| `EffectChance` | `factor` for the `chance` of the holder's extra effects |
| `MoveType` | `from`, `to` |
| `AlwaysHit` | `against` one Pokémon, or anyone |
| `WithoutType` | `type` |

A rule on a side applies to every Pokémon of that side, and a rule on the field to
everyone.

The engine has no switching and no bench: `Trapped` is only reported
(`Battle::trapped`), and nothing leaves the field except by fainting. Items and
abilities are data that moves can read, take, give and suppress; the engine does
not act on them.

### Host setup

Besides stats and moves, each Pokémon in the setup JSON can carry `species`,
`gender` (`"Male"`, `"Female"`, `"Genderless"`), `weight` in kilograms, `item`,
`ability`, `ivs` (six numbers) and `happiness`. The setup can name an
`environment`. `wbe_advance` also reports `weather`, the marks and effects on the
`field` and on both `sides`, and the `payout` earned by each side.
