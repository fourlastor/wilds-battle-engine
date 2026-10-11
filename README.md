# Wilds battle engine

`wilds-battle-engine` is the Rust battle simulation. Move definitions are loaded
from Lua; `MoveCatalog::builtin()` embeds the 617 files under `moves/`,
and `MoveCatalog::from_directory(path)` reads them from disk at runtime.
Both catalogs also include the system move Struggle, for 618 moves in total.
The [move catalog](moves/README.md) includes 538 entries from the Saved moves sheet,
plus support moves, additional local JSON moves and the original moves outside the sheet. Its manifest
records the 34 excluded Z/Max Moves. Files use canonical slugs and source display
names. [catalog.json](moves/catalog.json) records per-move coverage and the
remaining general engine limitations.
The host supplies choices for both sides.
The Rust API uses enums for sides, Pokémon types, stats, statuses, and weather.
Host-facing prompts contain `Choice::UseMove { id, move_id, target }` or
`Choice::Switch { id, to }`. `PromptReason` distinguishes action selection, faint replacement and move-triggered pivot choices; the host selects an offered choice in either case.
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
use Lua callbacks. The original 41 files retain their runtime IDs and behavior;
their filenames now use hyphens. New moves use the JSON's `Name` as their runtime
ID as well as their filename. Charge's Lua script raises Special Defense and asks Rust to
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

Production definitions live in `moves/`. The older API examples in
`crates/wilds-battle-engine/tests/sheet_moves.lua` exercise the script API, including
deliberately altered move names and prototypes. They are not the production catalog
or an authoritative source of move behavior. `tests/move_catalog.rs` exercises the
shipped files, including identity, spread targeting, critical hits, multi-hit attacks,
switching, source-linked traps and linked carrying.

### Move fields

| Field | Meaning |
|---|---|
| `target` | `Target.Selected` (a chosen foe, the default), `User`, `Ally`, `UserOrAlly`, `AnyOther` (a chosen foe or ally), `RandomOpponent`, `AllOpponents`, `AllAllies`, `UserAndAllies`, `AllOthers`, `All`, `Field`. A plain `effects` list acts on one Pokémon; the kinds that reach several need a script. |
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
| `ctx:hurt(who, amount)`, `ctx:heal(who, amount)` | `amount` is HP or `{fraction = 1/8}` of max HP. Returns the HP that changed. |
| `ctx:apply_status(who, Status.X, {chance, replace, announce_failure})`, `ctx:cure_status(who, Status.X)` | True if it took. |
| `ctx:confuse(who, {chance})`, `ctx:flinch(who, {chance})` | |
| `ctx:change_stat(who, Stat.X, stages, {chance})`, `ctx:reset_stats(who)` | `change_stat` returns how far the stage really moved. |
| `ctx:protect(who)`, `ctx:break_protect(who)`, `ctx:bind(who, {min_turns, max_turns})`, `ctx:free(who)` | A Pokémon that protects itself with the same move turn after turn succeeds a third as often each time, as with `Effect.Protect`. |
| `ctx:hide(Hidden.X)`, `ctx:unhide(who)` | The user hides until it next acts. `unhide` also calls off what the Pokémon was doing. |
| `ctx:set_weather(Weather.X, turns)`, `ctx:clear_weather()` | `Sun`, `Sandstorm`, `Rain`, `Hail`. |
| `ctx:set_types(who, {Type.X})`, `ctx:set_weight(who, kg)`, `ctx:take_item(who)`, `ctx:give_item(who, item)`, `ctx:suppress_ability(who)` | Suppression returns false for absent, already suppressed, fainted, or protected abilities. Active-ability checks immediately respect suppression. |
| `ctx:carry(who)`, `ctx:release()` | Link the user and a foe in the air, then release the pair. Carrying requires a living, active, unhidden foe under 200 kg. Pair state blocks switching and suppresses the carried member's action while it remains held. Interruption, fainting and `unhide` release both and cancel the carrier's continuation. |
| `ctx:switch_out()` | Requests a host-selected bench replacement for the user after this move and its hit reactions. Bypasses ordinary traps. A `Pivot` prompt pauses the turn; responding resumes the remaining actions without rerolling order or repeating moves. |
| `ctx:force_switch(who)` | Requests a random living bench replacement after this move and its hit reactions. Returns false when unavailable or blocked by Suction Cups, Guard Dog or `Rule.BlockForcedSwitch`. Cancels the outgoing member's pending action. |
| `ctx:adjacent(a, b)` | Whether two Pokémon occupy neighboring field slots. `who.position` is its zero-based slot from its trainer's perspective; opposing rows face each other. Fainting does not collapse slots. |
| `ctx:item_removable(who)` | Checks species-bound items independently of Sticky Hold, for damage bonuses such as Knock Off. Removal and atomic theft enforce both restrictions; fainted or suppressed Sticky Hold does not protect an item. |
| `ctx:eat_berry(who, forced)`, `ctx:eat_stolen_berry(from, to)` | Apply berry consumption effects and record history. Forced eating bypasses Unnerve and Klutz; stealing a berry respects removal restrictions and preserves the eater's own item. |
| `ctx:change_form(who, id)`, `ctx:restore_form(who)` | Apply a host-supplied form or restore the original one; return whether it changed. HP, status, stages, and PP are preserved. |
| `ctx:consume_item(who)` | Removes a held item, records it in `who.consumed_items` and `who.last_consumed_item`, and returns its ID (or nil). Taking an item does not record consumption. |
| `ctx:reduce_pp(who, move_id, amount)` | Returns PP removed, clamped to what remains; an absent move loses zero. Refreshed `who.pp` reflects the change. |
| `ctx:chance(probability)`, `ctx:effect_chance(probability)`, `ctx:random_int(min, max)` | `effect_chance` is for a move's extra effects: an `EffectChance` rule can multiply it. |
| `ctx:reached(who)` | Whether this use of the move got through to a Pokémon (the target if left out): the result of the hit already made on it, or else `try_hit` now. Use it to hold an effect back until the move connects. |
| `ctx:steal_boosts(from, to)`, `ctx:steal_item(from, to)`, `ctx:lose_type(who, Type.X)` | Shorthands built from the calls above. |
| `ctx:opposite_genders(a, b)`, `ctx:share_type(a, b)`, `ctx:type_number(0..15)` | `type_number` uses Hidden Power's order. |
| `who:raised_stages()`, `who:used_all_other_moves(id)` | Read-only helpers on a Pokémon. |
| `ctx:add_payout(amount)` | Money for the user's side, read by the host with `Battle::payout`. |
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
| `DamageEachTurn`, `HealEachTurn` | `fraction` of max HP, `message` (`{name}` is the Pokémon); `except_types` for damage |
| `DrainEachTurn` | `fraction`, `to` (who gets the HP), `message` |
| `DamageTaken`, `DamageDealt` | `factor`, and optionally only for a `category` or `type`; `not_on_crit` and optional `multi_battle_factor` for damage taken (alternate factor when either side has multiple active positions) |
| `AccuracyMultiplier` | `factor`; ordinary hit checks only, excludes OHKO and always-hit moves |
| `StatMultiplier` | `stat`, `factor` |
| `Redirect` | `to`, optional `type` and `priority` (-7 to 7). Redirects single-target moves to a living, adjacent teammate of the original target. Higher priority wins; later rules break ties. `ignore_redirection` moves bypass it. Use a shared condition group for mutually exclusive redirection effects. |
| `MovePriority` | `amount` (-7 to 7); adds to priority. Applied after turn-start hooks, retaining existing speed/tie ordering. |
| `BlockMoves` | nonempty `flags` list, optional `message`; prevents selection and execution of matching moves |
| `BlockStatus` | `statuses` (all if left out), `confusion`. Does not stop what a Pokémon does to itself. |
| `BlockStatDrops`, `BlockFlinch`, `Grounded`, `Endure`, `BlockForcedSwitch` | |
| `Trapped` | Optional `by` Pokémon; the trap ends when that source leaves or faints. Without it, a self trap lasts until removed or the holder leaves. |
| `EffectChance` | `factor` for the `chance` of the holder's extra effects |
| `MoveType` | `from`, `to` |
| `AlwaysHit` | `against` one Pokémon, or anyone |
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
Shed Shell can escape. `pokemon.linked_trap` reports source-linked trapping
independently of that escape permission. Binding also ends when its source leaves
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
