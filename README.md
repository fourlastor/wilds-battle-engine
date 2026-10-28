# Wilds battle engine

`wilds-battle-engine` is the Rust battle simulation. Move definitions are Lua files
named after their move ID. The library embeds the 753 files under `moves/`, plus the
system move Struggle, for 754 moves in total. Moves are loaded on first use:
`MoveCatalog::builtin()` uses only the embedded moves, while
`MoveCatalog::with_source(|id| ...)` asks a host function for each ID first and falls back
to them, so a host can add or replace moves from any number of folders without the
engine knowing. `MoveCatalog::from_directory(path)` is such a source over one folder.
A file must declare the ID it is named after; IDs are lowercase slugs.
The [move catalog](moves/README.md) includes 538 entries from the Saved moves sheet,
plus support moves, additional local JSON moves and the original moves outside the sheet. Its manifest
records the 34 excluded Z/Max Moves from the sheet. The move cutoff is Sword/Shield
and its expansions; Legends: Arceus moves and rules are excluded.
Files use canonical slugs and source display names.
Type effectiveness follows the Gen 8 chart, including Ice being super effective
against Grass and Grass resisting itself. A checked-in PokeAPI fixture verifies
all 324 single-type matchups; dual types multiply and immunities take precedence.
[catalog.json](moves/catalog.json) records per-move coverage and the
remaining general engine limitations.
The host supplies choices for both sides.
The Rust API uses enums for sides, Pokémon types, stats, statuses, and weather.
Host-facing prompts contain `Choice::UseMove { id, move_id, target }` and
`Choice::Switch { id, to }`, plus `Escape` and `Continue` for opted-in wild
encounters. `PromptReason` distinguishes action selection, faint replacement and
move-triggered pivot choices; the host selects an offered choice in each case.
Each active Pokémon gets its own action prompt. Once a side answered one this
turn, its later action prompts end with `Choice::Undo { id }` (JSON kind `undo`):
it takes back that side's most recent answer, releases a bench member the answer
reserved, and prompts for it again. Undo is offered to both sides, so a host can
let any player back out; automated players should never select it. Replacement
and pivot prompts do not offer it.
Rust exposes typed Lua constants for Pokémon types, stats, statuses, weather,
categories, targets, effects, and special accuracy modes. Move IDs and display
text remain strings. `MoveCatalog::from_lua` installs these constants before
evaluating move definitions; values from the wrong enum family are rejected.

```lua
{ id="toxic", name="Toxic", type=Type.Poison,
  category=Category.Status, pp=10, accuracy=0.9,
  effects={{kind=Effect.Status, status=Status.BadlyPoisoned}} }
```

Moves such as Dream Eater, False Swipe, Triple Kick, Explosion, and Hyper Beam
use Lua callbacks. Every move's runtime ID is its filename slug, the PokeAPI
move name (`solar-beam`). Charge's Lua script raises Special Defense and asks Rust to
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
        AdvanceStatus::End { .. } | AdvanceStatus::Escaped { .. } => break,
    }
}
# Ok::<(), wilds_battle_engine::BattleError>(())
```

For wild encounters, call `battle.set_wild_battle(true)?` before the first
`advance()`, or include `"wild_battle": true` in the JSON setup. The default is
false, so trainer battles and scripted encounters do not offer escape. This enables
`Choice::Escape { id }` for living active allies. It is a party command: selecting
it replaces other allied actions that turn, while the host still supplies opposing
actions. A failed attempt consumes the party's turn; a successful attempt stops
queued actions and residual effects. Bench members and replacement/pivot prompts
never offer escape. Recharging and scripted continuations offer a
`Choice::Continue { id }` alternative when escape is possible.

Active Run Away guarantees escape through move and ability traps; suppression
removes that guarantee. Ghost types also escape automatically. Neither changes
whether Run Away users can switch, and Shed Shell only affects switching. Ordinary
escape checks raw Speed against the opposing active slot (or another living foe),
ignoring stages and battle modifiers. Equal or greater Speed guarantees escape.
For slower users, the current compatibility formula is
`(floor(128 * speed / foe_speed) + 30 * previous_attempts) mod 256`, compared
strictly against a uniform integer from 0 to 255. The counter starts at zero and
survives moves and switches. This formula follows
[the Emerald implementation](https://github.com/pret/pokeemerald/blob/master/src/battle_util.c);
**exact Sword/Shield ordinary escape odds have not been verified**. Run Away's
unconditional wild escape is also described in the local PokeAPI ability data.

`BattleEvent::EscapeAttempt { actor, success }` reports the attempt, and
`AdvanceStatus::Escaped { side }` is a persistent terminal outcome distinct from a
win or draw. JSON uses choice kinds `escape` and `continue`, event kind
`escape_attempt` with `actor` and `success`, and terminal status
`{"kind":"escaped","side":"allies"}`. Hosts must handle this status in addition
to `end`; repeated `advance()` calls return it without repeating events.

Damage uses the Gen 8 integer calculation: floored base damage, 4096-based
modifier chains, half-down modifier application, and the 16 damage rolls from
85% through 100%. Weather, critical hits, random damage, STAB, effectiveness,
burn and final modifiers are applied in that order. Parental Bond scales its
second hit before weather. Attack-, power- and final-damage ability effects use
their respective stages, including the Gen 8 Heatproof override. Fixed damage
continues to bypass this formula. Exact regression vectors cover rounding and
modifier interactions; these rules supersede the original C# damage arithmetic.

In doubles, ordinary selected moves can target either opponent or the adjacent
ally, excluding the user and bench. Prompts list opponents first. Attacks follow
the chosen field slot through switches and Ally Switch; a fainted foe redirects
to another eligible foe, while a fainted ally makes the attack fail and spend PP.

Scripted spread moves apply the Gen 8 **0.75 damage modifier before weather**
when more than one living active recipient exists at the start of the move.
Protected, immune, hidden and missed targets still count. That count stays fixed
across Lua per-target loops and knockouts during the move. A single remaining
recipient takes full damage. `AllOthers` includes adjacent allies and foes;
`AllOpponents` includes adjacent foes only. Fixed damage, delayed single-target
attacks and Dragon Darts do not receive the spread penalty. Called moves have
their own recipient count. These rules follow the targeting and spread-damage
stages in [Pokémon Showdown's battle actions](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/sim/battle-actions.ts)
and [target selection](https://github.com/smogon/pokemon-showdown/blob/942cda71d14bf0618bd57991708a2e2734b6daba/sim/battle.ts).

The original compatibility port covers metadata and battle mechanics for 29 C# moves,
including turn order, accuracy, damage, status, weather, charging, repeated hits,
consecutive moves, and outcome resolution. The Rust RNG is seeded, so a Rust
run is reproducible; it does not reproduce Godot's RNG stream. Tests exercise
each move and selected interactions. The local parity suite compares one-turn
Dragon Rage, Tackle, Toxic, and Recover battles against the running C# engine,
including different damage rolls, a miss, a critical hit, status, poison damage,
healing, fainting, and battle outcome.
Broader C# event parity remains to be verified.
`wilds-battle-engine-ffi` builds as a C ABI dynamic library. Its declarations
are in `include/wilds_battle_engine.h`. `wbe_catalog_create` takes a host callback
that supplies move sources by ID (see `wbe_move_source`); `wbe_catalog_move` looks a
move up and `wbe_create_with_catalog` starts a battle from UTF-8 JSON setup.
`wbe_create` does the same with an optional move directory instead of a callback.
`wbe_advance` returns events, the next prompt or winner,
and participant state as JSON. `wbe_respond` supplies a choice, while
`wbe_catalog` validates and lists disk move files. Each returned string must
be released with `wbe_free_string`. The PokeWilds playtest host demonstrates
the complete call sequence and the editable setup format.

`Battle::with_rng(rng, sides, catalog)` accepts a `BattleRng` implementation
for controlled draws. `Battle::new(seed, sides, catalog)` retains the default
`SeededRng`. The `damage_roll` replay hook supplies a multiplier, normalized to an integer
percentage from 85 through 100. The C# parity runner can report intentional
differences after the Gen 8 type-chart and damage corrections. Run
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

Production definitions live in `moves/`. The older API examples in
`crates/wilds-battle-engine/tests/sheet_moves.lua` exercise the script API, including
deliberately altered move names and prototypes. They are not the production catalog
or an authoritative source of move behavior. `tests/move_catalog.rs` exercises the
shipped files, including identity, spread targeting, critical hits, multi-hit attacks,
switching, source-linked traps and linked carrying.

### Move fields

| Field | Meaning |
|---|---|
| `target` | `Target.Selected` (a chosen adjacent foe or ally, the default), `User`, `Ally`, `UserOrAlly`, `AnyOther` (a chosen foe or ally without adjacency restrictions), `RandomOpponent`, `AllOpponents`, `AllAllies`, `UserAndAllies`, `AllOthers`, `All`, `Field`. A plain `effects` list acts on one Pokémon; the kinds that reach several need a script. |
| `flags` | A list of words such as `{"contact", "sound"}`. The engine only passes them on (`ctx.move.flags.contact`, `ctx.hit.contact`). |
| `hits_hidden` | Hidden places the move still reaches, such as `{Hidden.Air}`. |
| `usable_while_asleep`, `usable_while_frozen` | The second also thaws the user. |
| `on_turn_start` | Runs before anyone acts, on a turn where the move was chosen (Focus Punch, Beak Blast). |
| `on_hit` | After `ctx:watch_hits_until_next_action()`, reacts to HP loss while surviving. After `ctx:watch_contacts_until_next_action()`, reacts to successful contact, including zero HP loss and knockout. Misses, protection and immunity do not make contact. `ctx.target` is the attacker and `ctx.hit` describes the current hit. |

### What a script reads

Pokémon tables: `ctx.user`, `ctx.target`, and the lists `ctx.targets` (everyone the
move is aimed at), `ctx.allies`, `ctx.foes`, `ctx.others`, `ctx.everyone`. Fainted
Pokémon are left out of the lists. `ctx.pending` lists Pokémon with a pending move
after the current action, in queue order. `force_stab = true` applies the usual
STAB bonus even when the attacker lacks the hit's type, without doubling an existing bonus.

| On a Pokémon | |
|---|---|
| `name`, `species`, `level`, `hp`, `max_hp`, `gender`, `weight`, `happiness`, `ivs` | `ivs` has `hp`, `attack`, `defense`, `sp_attack`, `sp_defense`, `speed`, or is nil. |
| `status`, `confused`, `flinched`, `bound`, `trapped`, `protected`, `recharging`, `fainted` | `status` is a `Status` value or nil. |
| `types`, `:has_type(Type.X)`, `grounded` | Types as they count right now. |
| `attack`, `defense`, `sp_attack`, `sp_defense`, `speed` | With stat stages and effects applied. |
| `stages[Stat.X]`, `:stage(Stat.X)` | −6 to +6. |
| `carrying`, `carried_by` | Linked Pokémon tables, or nil. Both host state APIs report the corresponding participant references. |
| `hidden` | A `Hidden` value (`Air`, `Underground`, `Underwater`, `Vanished`) or nil. |
| `item`, `ability` | Item/ability IDs or nil. Power Herb activates on charging; supported berries activate or can be eaten by move scripts. |
| `marks[name]`, `effects[name]` | See below. |
| `acted`, `selected_move`, `switching` | This turn. `selected_move` has `id`, `name`, `type`, `category`, `damaging`, `priority`. |
| `damage_taken`, `hurt_this_turn`, `last_attacker`, `last_hit` | This turn. `last_hit` has `damage`, `move`, `category`, `type`, `contact`. |
| `turns_on_field`, `moves`, `used_moves[id]`, `pp[id]`, `last_move`, `last_move_failed`, `streak` | `streak` counts the turns in a row this move was used without failing. |
| `side`, `team`, `is_user`, `is_ally`, `is_foe` | `team` is the side table. |

Also: `ctx.weather`, `ctx.weather_turns`, `ctx.turn` and `ctx.total_turns` (turns of a
forced move), `ctx.battle_turn`, `ctx.environment`, `ctx.move` (`id`, `name`, `type`,
`category`, `priority`, `flags`), `ctx.field`, `ctx.user_side`, `ctx.foe_side`. A side
table has `marks`, `effects`, `pokemon` and `fainted_last_turn`.

`who.stats_raised_this_turn` and `who.stats_lowered_this_turn` record actual stat
stage changes during the current turn, including self-inflicted changes. They reset
before turn-start hooks; capped changes and resetting stages do not trigger them.

### What a script does

`who` is a Pokémon table. Options are a table and can be left out.

| Call | Result |
|---|---|
| `ctx:damage(power, options)` | A hit on everyone the move is aimed at. Returns `hit`, `damage`, `fainted`, `critical`, `effectiveness`, `missed`, `blocked`, `immune`, `hits`. |
| `ctx:multi_hit(power, min, max, options)` | Several hits on one Pokémon, as `Effect.MultiHit`. |
| `ctx:direct_damage(who, amount, {typeless, min_target_hp})` | An exact amount, as fixed and level damage. |
| `ctx:try_hit(who, options)` | The usual protection, hiding and accuracy checks without damage. Returns true or false. The next `damage` or `multi_hit` on that Pokémon follows the answer instead of checking again. |
| `ctx:healing_hit(who, hp)` | Healing through an attack (Present): checks accuracy, protection and type/ability immunity, bypasses Substitute and respects healing prevention. Returns HP restored. |
| `ctx:hurt(who, amount)`, `ctx:heal(who, amount)` | `amount` is HP or `{fraction = 1/8}` of max HP. Returns the HP that changed. |
| `ctx:apply_status(who, Status.X, {chance, replace, announce_failure})`, `ctx:cure_status(who, Status.X)` | True if it took. |
| `ctx:confuse(who, {chance})`, `ctx:flinch(who, {chance})` | |
| `ctx:change_stat(who, Stat.X, stages, {chance})`, `ctx:reset_stats(who)` | `change_stat` accepts nonzero changes from -12 to +12, applies abilities, clamps the final stage to -6…+6 and returns how far it moved. |
| `ctx:protect(who)`, `ctx:break_protect(who)`, `ctx:bind(who, {min_turns, max_turns})`, `ctx:free(who)` | Self-protection shares a consecutive success counter across move names, with a one-third multiplier per success, as with `Effect.Protect`. |
| `ctx:hide(Hidden.X)`, `ctx:unhide(who)` | The user hides until it next acts. `unhide` also calls off what the Pokémon was doing. |
| `ctx:set_weather(Weather.X, turns)`, `ctx:clear_weather()` | `Sun`, `Sandstorm`, `Rain`, `Hail`. |
| `ctx:set_types(who, {Type.X})`, `ctx:set_weight(who, kg)`, `ctx:take_item(who)`, `ctx:give_item(who, item)`, `ctx:suppress_ability(who)` | Suppression returns false for absent, already suppressed, fainted, or protected abilities. Active-ability checks immediately respect suppression. |
| `ctx:carry(who)`, `ctx:release()` | Link the user and a foe in the air, then release the pair. Carrying requires a living, active, unhidden foe under 200 kg. Pair state blocks switching and suppresses the carried member's action while it remains held. Interruption, fainting and `unhide` release both and cancel the carrier's continuation. |
| `ctx:baton_pass({effect_names})` | Requests a host-selected pivot and transfers stages, Substitute, confusion, binding, critical-hit and next-move boosts, ability suppression and the named volatile effects. Transfer precedes entry hazards and abilities; Power Trick swaps the recipient's own stats. Gen 8 Baton Pass skips Pursuit interception. |
| `ctx:switch_out()` | Requests a host-selected bench replacement for the user after this move and its hit reactions. Bypasses ordinary traps. A `Pivot` prompt pauses the turn; responding resumes the remaining actions without rerolling order or repeating moves. |
| `ctx:force_switch(who)` | Requests a random living bench replacement after this move and its hit reactions. Returns false when unavailable or blocked by Suction Cups, Guard Dog or `Rule.BlockForcedSwitch`. Cancels the outgoing member's pending action. |
| `ctx:redirect_target(who)` | Resolves redirection for a target chosen during the script, such as Counter's last attacker. Returns the Pokémon to aim at. |
| `ctx:adjacent(a, b)` | Whether two Pokémon occupy neighboring field slots. `who.position` is its zero-based slot from its trainer's perspective; opposing rows face each other. Fainting does not collapse slots. |
| `ctx:item_removable(who)` | Checks species-bound items independently of Sticky Hold, for damage bonuses such as Knock Off. Removal and atomic theft enforce both restrictions; fainted or suppressed Sticky Hold does not protect an item. |
| `ctx:eat_berry(who, forced)`, `ctx:eat_stolen_berry(from, to)` | Apply berry consumption effects and record history. Forced eating bypasses Unnerve and Klutz; stealing a berry respects removal restrictions and preserves the eater's own item. |
| `ctx:transform(who, target)` | Copies battle identity, non-HP stats, types, stages, ability and learned moves (at most five PP each). Preserves personal HP, level, status, item and original species base Attack. Restores originals on withdrawal or fainting. `who.transformed` and both FFI/WASM snapshots expose the state. |
| `ctx:replace_move(who, old, new, permanent)` | Replaces a learned slot with base PP. Temporary replacements restore the original slot and its spent PP on withdrawal; permanent replacements persist in the battle roster for the host to save. |
| `ctx:receive_berry(who, item)` | Applies an external berry's eating effect, preserving the recipient's held item and Recycle slot, for Fling. |
| `ctx:change_form(who, id)`, `ctx:restore_form(who)` | Apply a host-supplied form or restore the original one; return whether it changed. HP, status, stages, and PP are preserved. |
| `ctx:check_protection()` | Rolls and records the shared consecutive protection chance without applying Protect, for Endure and other guards. |
| `ctx:swap_items(a, b)` | Exchanges held items atomically; rejects an opposing Sticky Hold or species binding at either destination. Does not consume either item. |
| `ctx:recycle_item(who)` | Restores the last consumed held item into an empty slot once. Stolen berries do not change which item can be recovered; consumption history persists. |
| `ctx:consume_item(who, eaten)` | Removes a held item; pass false for `eaten` to spend a berry without enabling Belch. It records it in `who.consumed_items` and `who.last_consumed_item`, and returns its ID (or nil). Taking an item does not record consumption. |
| `who.last_move_type` | Actual type of the last attempted move, including Weather Ball and other dynamic damage types; cleared on withdrawal. |
| `who.team.roster` | The complete read-only party, including fainted members; `team.pokemon` continues to contain living members. |
| `ctx:instruct(who)` | Queues an immediate extra action using the target's previous move and target slot. It spends PP, runs status checks and preserves the original pending action; turn-end effects still run once. Lua checks move eligibility. |
| `ctx:call_move(id, target)` | Executes another catalog move immediately with its targeting, hit checks and effects, spending only the caller's PP. An omitted target is selected by the engine; nesting is bounded. |
| `ctx:prepare_replacement_heal(who, restore_pp)` | Reserves full healing and status curing for a slot, with optional PP restoration. Requires a living bench member, activates before entry hazards, and remains pending through a healthy entrant. |
| `Rule.Snatch {moves={...}}` | Steals the next eligible move or one flagged `snatch`, using the recipient's stats and side without spending their PP. The fastest waiting snatcher wins, and a stolen move cannot be stolen again. |
| `ctx:schedule_attack(target, power, delay)` | Reserves a delayed hit on a field position, using the original user's stats at impact even after withdrawal or fainting. Future attacks bypass protection, check accuracy and immunity at impact, and do not trigger Destiny Bond or Grudge. |
| `ctx:schedule_heal(who, amount, delay)` | Schedules one pending heal for a field slot after the given number of turns, before residual damage; follows replacements and respects Heal Block. |
| `who.base_attack` | Optional host-supplied original species base Attack, also accepted by the FFI setup. Beat Up requires it for eligible party members and fails explicitly when missing. |
| `ctx:multi_hit(power, min, max, {powers={...}})` | A fixed-length hit sequence with separate base power per hit, sharing one accuracy check and applying reactions between hits. |
| `Rule.OnlyMove {id}` | Restricts choices and overrides an already queued move while the required slot has PP, preserving that action's priority. The effect expires when its slot is gone or depleted. |
| `ctx.last_move` | The battle's last attempted move before the current action, including the actual move produced by a caller. Actions prevented by sleep, flinching, etc. leave it unchanged. |
| `ctx:move_ids()` | Reads every ID in the loaded catalog, sorted for deterministic scripted selection. |
| `ctx:move_info(id)` | Reads catalog identity, base type, category, priority, PP and flags; returns nil for an unknown ID. |
| `ctx:reduce_pp(who, move_id, amount)` | Returns PP removed, clamped to what remains; an absent move loses zero. Refreshed `who.pp` reflects the change. |
| `ctx:chance(probability)`, `ctx:effect_chance(probability)`, `ctx:random_int(min, max)` | `effect_chance` is for a move's extra effects: an `EffectChance` rule can multiply it. |
| `ctx:reached(who)` | Whether this use of the move got through to a Pokémon (the target if left out): the result of the hit already made on it, or else `try_hit` now. Use it to hold an effect back until the move connects. |
| `ctx:steal_boosts(from, to)`, `ctx:steal_item(from, to)`, `ctx:lose_type(who, Type.X)` | Shorthands built from the calls above. |
| `ctx:opposite_genders(a, b)`, `ctx:share_type(a, b)`, `ctx:type_number(0..15)` | `type_number` uses Hidden Power's order. |
| `who:raised_stages()`, `who:used_all_other_moves(id)` | Read-only helpers on a Pokémon. |
| `ctx:swap_side_effects(names)` | Exchanges the named side conditions, preserving their durations, counters and rules. Returns whether any existed. |
| `ctx:swap_positions(a, b)` | Swaps two active teammates without withdrawing or resetting them. Pending attacks keep targeting their chosen slots; emits `PositionsSwapped` / `swap_positions`. |
| `ctx:set_payout_multiplier(factor)` | Sets a persistent side multiplier without stacking. `Battle::payout` includes it for Pay Day; `Battle::payout_multiplier` and JSON `payout_multiplier` let the host apply it to trainer prizes. |
| `ctx:add_payout(amount)` | Money for the user's side, read by the host with `Battle::payout`. |
| `ctx:set_hp(who, hp)` | Sets a living Pokémon's HP to an integer between 1 and its maximum, without damage, healing or hit reactions. Returns its HP and emits `hp_set`. |
| `ctx:succeed()` | Records a successful effect, even if another target was missed or protected during the same move. |
| `ctx:hurry(who)` | Promotes that Pokémon's pending action to immediately after the current action. Usable in hit reactions. Omitting `who` promotes all remaining users of this move (Round). Actors that have already acted do not act again. |

Damage options: `ohko_accuracy`, `accuracy`, `never_miss`, `ignore_protect`, `ignore_evasion`,
`hits_hidden` (a list of `Hidden` values, or `true`), `drain`, `recoil`,
`min_target_hp`, `typeless`, `high_crit`, `always_crit`, `force_stab`, `target` (one Pokémon),
`type`, `also_type`, `effective` (`{[Type.Water] = 2}` replaces the type chart
against the listed types), `category`, `attack_from` (whose attacking stat is used),
`attack_stat`, `defense_stat`, `ignore_stages`. `try_hit` takes the first six. `ohko_accuracy` is an OHKO base probability plus
1% per level above the target; lower-level users fail even with guaranteed hits.
It ignores accuracy/evasion stages and ordinary accuracy multipliers.

A damage call can set `contact = true` or `contact = false` for that hit. If omitted,
contact follows the move's flags. This applies to multi-hit attacks too and controls
hit reactions and the contact value in `last_hit`.

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
| `ExplodeOnType` | `type`, `fraction`, optional `message`; cancels moves of that type after PP is spent and deals rounded residual damage. Magic Guard prevents that damage. |
| Screen damage rules | `DamageTaken` accepts `screen=true` to allow Infiltrator and `unless_side_effect` to avoid stacking with another named screen. |
| `BlockPriority` | Blocks opposing moves with positive effective priority; `grounded_only` restricts protection to grounded, visible targets. |
| Terrain recovery and priority | `HealEachTurn` accepts `grounded_only`. A move's `priority_terrain` adds one priority on that terrain while grounded; pending action order updates after terrain changes. |
| `IncomingBasePower` | `moves` and/or `type`, `factor`, optional `grounded_only`; modifies incoming base power before the damage formula. |
| `BlockGroundedStatus` | `statuses` (empty means all), `confusion`; protects grounded Pokémon, including from self-inflicted status. `BasePower` and `IncomingBasePower` accept `grounded_only` for terrain bonuses. |
| `IncomingAlwaysHit` | Ordinary incoming moves pass accuracy checks; OHKO accuracy, Protect and hiding remain separate. |
| `Airborne` | Grants Ground immunity unless grounded. Read `pokemon.airborne` for this effect or `pokemon.on_ground` for terrain/hazard eligibility. Moves flagged `hits_airborne` ignore levitation immunity. |
| `BlockHealing` | Blocks healing, including drain and passive recovery; combine with `BlockMoves` for the `heal` flag. Direct HP assignments and Regenerator on withdrawal bypass it. |
| `ReflectMoves` | Reflects eligible incoming moves, including side hazards, once. Independent of ability suppression. |
| `SwapDefenses` | Field rule exchanging raw Defense and Sp. Defense for calculations, retaining their stage modifiers; defensive attacking-stat overrides follow Wonder Room rules. |
| `ReverseSpeed` | Field rule reversing action Speed within each priority bracket, including mid-turn queue updates. |
| `FaintOnExpiry` | Faints the Pokémon at the end of a timed countdown, with count messages; switching cancels it. |
| `StatusOnExpiry` | `status`, `by`; inflicts status when a Pokémon condition times out, checking current immunities. Switching or explicit removal cancels it. |
| `StatEachTurn` | `stat`, `stages`, `by`; changes a stage at turn end, respecting stat abilities and prevention. Ends when its source leaves or faints. |
| `DamageEachTurn`, `HealEachTurn` | `fraction` of max HP, `message` (`{name}` is the Pokémon); `except_types` for damage |
| `DrainEachTurn` | `fraction`, `to` (who gets the HP), `message` |
| `BasePower` | `factor` and optional move `type`; modifies base power before the damage formula, rounding halves down |
| `DamageTaken`, `DamageDealt` | `factor`, and optionally only for a `category` or `type`; `not_on_crit` and optional `multi_battle_factor` for damage taken (alternate factor when either side has multiple active positions) |
| `AccuracyMultiplier` | `factor`; ordinary hit checks only, excludes OHKO and always-hit moves |
| `StatMultiplier` | `stat`, `factor` |
| `Redirect` | `to`, optional `type` and `priority` (-7 to 7). Redirects single-target moves to a living, adjacent teammate of the original target. Higher priority wins; later rules break ties. `ignore_redirection` moves bypass it. Use a shared condition group for mutually exclusive redirection effects. |
| `Attention` | `to`, `priority` (default 1), optional `powder` and `allow_carried`. Draws opposing moves only; powder fails against Grass types and Overcoat. A carried user cannot draw attention unless allowed. |
| `MovePriority` | `amount` (-7 to 7); adds to priority. Applied after turn-start hooks, retaining existing speed/tie ordering. |
| `SealMoves` | On a Pokémon, seals its currently learned moves for all opponents while it is alive and active; Struggle is exempt |
| `NoRepeat` | Prevents selecting the previous move again, except Struggle and forced sequences; does not cancel an already selected action |
| `BlockMove` | Move `id` and optional `message`; blocks selection and execution |
| `BlockCategory` | `category`, optional `except` move IDs and `message`; blocks selection and execution |
| `BlockMoves` | nonempty `flags` list, optional `message`; prevents selection and execution of matching moves |
| `BlockStatus` | `statuses` (all if left out), `confusion`. Does not stop what a Pokémon does to itself. |
| `BlockStatDrops`, `BlockFlinch`, `BlockCritical`, `Grounded`, `Endure`, `BlockForcedSwitch` | |
| `CriticalStage` | `stages` (0–3), added to the holder's critical-hit stage while the condition lasts |
| `Trapped` | Optional `by` Pokémon; the trap ends when that source leaves or faints. Without it, a self trap lasts until removed or the holder leaves. |
| `EffectChance` | `factor` for the `chance` of the holder's extra effects |
| `MoveType` | `from`, `to` |
| `AlwaysHit` | `against` one Pokémon, or anyone |
| `NegateTypeImmunity` | `defending` type and `incoming` move types; ignores that type only for those incoming matchups |
| `TypeEffectiveness` | Incoming `type` and `factor`; changes the matchup seen by damage, defensive abilities and resistance berries |
| `Infatuated` | `by` Pokémon; blocks half of attempted actions, ends when the source leaves, and is cleared by Oblivious |
| `RequiresStatus` | `status`; removes a Pokémon condition when that status ends. Comatose also satisfies `Status.Asleep`. |
| `IgnoreEvasionBoost` | Disregards positive Evasion stages when checking incoming accuracy; negative stages still count |
| `WithoutType` | `type` |

A rule on a side applies to every Pokémon of that side, and a rule on the field to
everyone.

The host can configure active slots with `Battle::set_active([allies, foes])`
before advancing, or JSON `"active":{"allies":[0],"foes":[0]}`. These arrays
contain party indices in field order; other party members are on the bench.
Omitting them preserves the original all-active setup. `ParticipantId.index`
always identifies the same party member; `Battle::position` returns its current
field slot, and `active_indices` maps slots back to party members. Both JSON
state APIs expose this map under `active`.

Voluntary switches resolve before ordinary moves. Attacks aim at field slots,
so they hit the incoming occupant. Fainted slots prompt for a replacement before
the next turn. Switching resets stages and volatile effects, restores temporary
forms/types, and preserves HP, nonvolatile status, PP, items and consumption history.
Pending actions refresh their speed after each action and resumed pivot, including
stat-stage and paralysis changes. Priority brackets and the original tie rolls are
preserved, as are Custap ordering and explicit promotions such as Pledge combinations.
Bench members do not act, receive residual effects or activate abilities/items.
A pending move flagged `intercepts_switch` runs once before an adjacent foe
withdraws voluntarily or pivots. Its target has `switching = true` during that
action; Pursuit uses this for doubled power and guaranteed accuracy. A knockout
cancels withdrawal and the host chooses a faint replacement. Forced switches do
not trigger interception.
Lua `pokemon.team.pokemon` includes the living party, while `ctx.allies`, `foes`,
`everyone`, and targets contain living active members. `pokemon.active` and
`pokemon.position` distinguish the bench (`false`/`nil`) from a field slot.

`Trapped` prevents voluntary switches. Ghost types and holders of an active
Shed Shell can switch out. `pokemon.linked_trap` reports source-linked trapping
independently of that switch permission. Binding also ends when its source leaves
or faints. Move-triggered switches use separate mechanics. Items and abilities are data that moves can
read, take, give and suppress. Power Herb now
activates when a move charges: `Effect.TwoTurn`, or a scripted move flagged `charge`
that calls `force_move(2, ...)`, unless flagged `no_power_herb` (Sky Drop).
Its second phase runs immediately, paying PP once.
Weather shortcuts preserve the item. Berry and gem activation also run in the engine. Other item and ability effects
remain separate mechanics.

### Host setup

Besides stats and moves, each Pokémon in the setup JSON can carry `species`,
`gender` (`"Male"`, `"Female"`, `"Genderless"`), `weight` in kilograms, `item`,
`ability`, `ivs` (six numbers) and `happiness`. The setup can name an
`environment`. `wbe_advance` also reports `weather`, the marks and effects on the
`field` and on both `sides`, and the `payout` earned by each side.

Berry consumption supports the generation 3–8 berries. Health and status berries
activate between actions and before choices. Resistance berries activate before
damage; reactive berries run after their hit or after all hits, as appropriate.
Multi-hit attacks resolve each hit against current state, including healing
between hits. `item_effect_on_hit` lets scripts remove or eat a target's item
before its later reactions (Pluck, Bug Bite, Incinerate). Type gems boost the
entire matching attack, including every target and hit, and are consumed once.
Custap changes ordering within the existing priority bracket; Micle grants its
next eligible accuracy check a bonus and then expires. `who.ate_berry` persists for the battle and gates Belch in both move
selection and execution. Stuff Cheeks requires a held berry and consumes it after
a successful Defense increase. Hosts may supply `nature_down` (a `CombatStat` in
Rust or `"Attack"`, `"Defense"`, `"SpAttack"`, `"SpDefense"`, `"Speed"` in setup JSON)
for flavor-based confusion; absent means a neutral flavor preference. This does
not recalculate host-provided stats. Berry effects account for Ripen, Cheek Pouch,
Gluttony, Klutz and Unnerve when those abilities are active.

Hosts supply form presets through `Pokemon.forms` (`PokemonForm` values), or the
setup JSON's `forms` object keyed by form ID. Each entry contains `attack`,
`defense`, `sp_attack`, `sp_defense`, `speed`, `types`, and optional `weight`.
An optional `max_hp` changes the HP ceiling while preserving missing HP (it never
revives a fainted Pokémon). Power Construct requires this field on the
`zygarde-complete` preset. Transform copies neither current nor maximum HP.
These are computed battle stats, just like the initial Pokémon's stats; the engine
does not infer IVs, EVs or nature modifiers. Optional `form` identifies an initial
alternate form. Form changes emit `BattleEvent::FormChanged` (`kind: "form"` in
JSON), and `who.form` reports the current ID in Lua. The battle preserves its
original stats and types for restoration, including on fainting.

Relic Song uses the `meloetta-pirouette` preset for Meloetta (species ID 648), then
restores the original form on its next successful use. If starting in Pirouette
Forme, provide `form: "meloetta-pirouette"` and a `meloetta-aria` preset. Missing
form data produces an explicit message and leaves the stats intact.

Ability-owned rules are separate from ordinary conditions: Rust hosts can set
`Pokemon.ability_rules`, or setup JSON can provide `ability_rules_lua` (for example,
`"return {{kind=Rule.DamageTaken, factor=0.5}}"`). The same typed rule parser validates
these definitions. `ability_bypassable` overrides the standard bypass metadata
for a host-defined ability. Rules stop when the ability is suppressed; a move
flagged `ignore_ability` bypasses eligible target abilities only for that move.
Lua `who.ability_id` reads the stored, normalized ability even when suppressed;
`who.ability_flags` exposes copy/replacement restrictions. `ctx:copy_ability(to, from)`
transfers the ability together with its host-defined rules and bypass metadata.
`ctx:set_ability(who, id)` replaces it with a named ability and clears old custom rules.
`ctx:swap_abilities(a, b)` exchanges both states before activating either, and rejects
protected abilities without changing either Pokémon.
Temporary changes restore the original state on withdrawal and activate entry effects
without running switch-out recovery.
Mold Breaker, Teravolt and Turboblaze use the same gate. End-of-turn rules become
active again after temporary bypass ends.

Native defensive hooks currently include Multiscale, Shadow Shield, Filter,
Solid Rock, Prism Armor, Fur Coat, Marvel Scale, Ice Scales, Fluffy, Punk Rock's
defense, Battle Armor, Shell Armor, Sturdy, Wonder Guard, Levitate, Soundproof,
Bulletproof, Telepathy, and the type absorption/boost effects of Water Absorb,
Volt Absorb, Dry Skin, Flash Fire, Lightning Rod, Storm Drain, Motor Drive and
Sap Sipper. Huge Power and Pure Power also affect the computed Attack stat.
The ability catalog is extensible and does not yet implement every canonical
ability; move completion means the required mechanics are available, not full
official battle parity. Shadow Shield and Prism Armor are not bypassable.

Intimidate activates for initial active Pokémon and on subsequent entry, targeting
adjacent opponents. Its immunity checks include stat-drop protection, Hyper Cutter,
Inner Focus, Oblivious, Own Tempo and Scrappy; Guard Dog reverses its drop and Rattled
raises Speed. Clear Body, White Smoke and Full Metal Body block external stat drops.
Regenerator and Natural Cure activate on withdrawal (including forced switches),
before suppression resets. Fainted Pokémon do not activate these effects.

Contrary reverses stage changes and Simple doubles them before stage caps and
external stat-drop protection. Suppression and eligible ability bypass apply.
`Target.Ally` offers adjacent teammates. Without one, the move can be selected
and fails after spending PP.

Move metadata `target_user_unless_type = Type.Ghost` makes targeting depend on
the user's current type, both when choices are offered and when the move runs.
The optional `adjacent_target` flag restricts random opponents to adjacent slots.

Lua `ctx.active_per_side` reports the configured battle width, including empty
slots. `ctx:hurry(who)` promotes a pending move; `ctx:delay(who)` moves it after
other pending actions, returning false if that Pokémon has no move left.

Lua `who.types` includes the current added type, while `who.base_types` and
`who.added_type` expose the two parts. `ctx:add_type(who, type)` replaces an existing
added type; `ctx:set_types(who, types)`, form changes and switching clear it.

Lua `who.raw_stats[Stat.Attack]` (and the other combat stats) reads the host-computed
stat before stages and modifiers. `ctx:set_raw_stat(who, stat, value)` changes it
until withdrawal, when the original host data is restored.

Lua `ctx:set_stage(who, stat, stage)` sets an exact stage in -6..6 without these
modifiers; stat theft uses it to clear the donor before boosting the recipient.

Entry hazards run in placement order before entry abilities on every switch path.
`ctx:add_hazard(team, name)` accepts `spikes`, `toxic_spikes`, `stealth_rock`, or
`sticky_web`, and returns false at the layer cap. They are persistent named side
conditions, so Rapid Spin and `end_effect` remove them. Spikes stack to three
layers; Toxic Spikes stack to two and grounded Poison types absorb them. Flying
types and active Levitate avoid grounded hazards unless forcibly grounded.
Stealth Rock uses Rock effectiveness; Sticky Web uses stage abilities and external
stat-drop protection. Hazard damage respects Magic Guard. No new held-item effects
are included. A hazard knockout stops later hazards and the incoming ability.

Magic Bounce reflects flagged status moves, including hazards aimed at the owner's
side. Reflection spends only the original user's PP, cannot bounce recursively,
and respects ability suppression and bypass. Spread status moves still affect
unprotected teammates; reflected Parting Shot offers the reflector a pivot choice.
Scripts should use `try_hit`/`reached` before targeted status effects, as with protection.

Substitute costs one quarter of maximum HP and creates a separate HP pool. It
cannot be created twice or when the user cannot afford the cost; creation releases
binding. Each hit removes substitute HP without spilling into the owner's HP.
Later hits can hit the owner after it breaks. Recoil and drain use the HP actually
removed, while target secondary effects, contact reactions, theft, trapping and
phazing do not pass through that hit. Residual damage still affects the owner.
Sound moves, `bypass_substitute` moves and active Infiltrator bypass the pool.
Switching and fainting clear it; Intimidate respects it.

Dragon Darts uses `smart_target` with `ctx:multi_hit`: Rust checks each recipient
once, then splits the hits between eligible adjacent foes. If one is protected,
immune, hidden or missed, the other receives both hits. Redirection concentrates
the hits on the redirector; these hits do not receive a spread-damage penalty.

Lua exposes `pokemon.substitute_hp` and `ctx:substitute()`. Damage reports include
`substitute`; `hit` still reports a successful attack on the doll, while `reached`
reports whether it reached the owner. The C ABI and Move Lab state include
`substitute_hp`, and `substitute` events carry the target, absorbed amount and
remaining pool HP. Move Lab displays that pool separately from ordinary HP.

`ctx:can_apply_status(who, status, {ignore_terrain=false})` checks current status and immunities without mutation. Delayed moves may check terrain separately from status application.

`Rule.Guard` blocks incoming hits with optional `spread_only`, `priority_only`,
`damaging_only`, or `status_only` filters. `unbypassable` also blocks moves that
normally ignore protection. `contact_stat` and `contact_stages` apply a stage change
to a blocked contact attacker, including moves that decide contact dynamically.
`contact_damage` deals a fraction of the contact attacker's maximum HP.
`contact_status` applies a major status to the contact attacker, checking current
status immunity and terrain. Magic Guard prevents `contact_damage`. Apply the guard to a Pokémon or side for one-turn protection.
`ctx:break_protect(who)` removes the target's protection and its side's guards;
terrain protection remains separate. `ctx:record_protection()` advances the shared
protection counter without rolling a failure chance (Wide Guard and Quick Guard).

`Rule.ExpiresOnAction{except_move=...}` removes its condition when the holder next
attempts an action; an interrupted action also clears it. `Rule.FaintAttacker`
takes down the opposing attacker only when direct move damage knocks out the holder.
`Rule.DepleteAttackerPp` empties the PP of the direct attacker's selected move.
Both faint reactions exclude delayed attacks and residual damage.

`Rule.BlockItems` suppresses existing held-item effects without removing the item.
`Battle::item_active` and Lua `who.item_active` expose activation separately from
physical possession. Suppression covers berries, gems, Power Herb, Shed Shell and
item-dependent move types; forced berry eating still works.

Illusion keeps real battle traits intact and exposes a separate `Pokemon.illusion`
appearance (nickname, species, form and gender). Use `display_name()` for messages
and the appearance for visuals. JSON snapshots expose `display_name` and `illusion`;
`kind: "appearance"` events announce disguise changes and reveals. Snapshots are
full host state, not a filtered opponent view: the host controls information access.
Party identifiers stay stable while an internal party order tracks switches for
Illusion's last-healthy-member selection.

Parental Bond prepares two separate hits for eligible attacks. Lua scripts expose
`ctx.hit_index`, `ctx.hit_count`, and `ctx.last_hit` (updated after damage), plus
`ctx.move_state`, a table shared across those hits. Put one-time preparation data
in that table and use `last_hit` for end-of-move cleanup; ordinary moves have one
hit. Existing multi-hit, charge, future and explicitly excluded moves do not use
this path. Damage-based recoil is accumulated and applied after both hits.
