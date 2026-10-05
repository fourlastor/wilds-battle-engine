//! Browser build of the Wilds battle engine for Move Lab.
//!
//! The page drives battles through the engine's own C ABI (`wbe_*`), the same
//! functions the game host calls. Move files are written to Emscripten's
//! in-memory file system, so the engine loads them exactly as it does on disk.
//! The two `mlab_*` functions below only read state the public API already
//! exposes; they exist so the editor can list moves and draw the battle.

use serde_json::{Value, json};
use std::ffi::{CStr, CString, c_char};
use wilds_battle_engine::{Accuracy, Battle, Effect, MoveCatalog, MoveSpec, Pokemon, Side};

// The engine's C ABI, compiled from its own source file so the page and the
// game run the same code. Depending on the crate instead would also build its
// native `cdylib`, which Emscripten cannot link.
#[path = "../../../crates/wilds-battle-engine-ffi/src/lib.rs"]
mod ffi;

fn main() {}

fn output(value: Value) -> *mut c_char {
    CString::new(value.to_string())
        .unwrap_or_else(|_| CString::new(r#"{"error":"text contains a NUL byte"}"#).unwrap())
        .into_raw()
}

fn accuracy(value: Accuracy) -> Value {
    match value {
        Accuracy::Chance(chance) => json!({"kind": "chance", "value": chance}),
        Accuracy::Always => json!({"kind": "always"}),
        Accuracy::OneHitKnockout => json!({"kind": "ohko"}),
    }
}

fn effect(value: &Effect) -> Value {
    match value {
        Effect::Damage {
            power,
            recoil,
            drain,
            high_crit,
            always_crit,
        } => json!({"kind": "Damage", "power": power, "recoil": recoil, "drain": drain,
            "high_crit": high_crit, "always_crit": always_crit}),
        Effect::MultiHit { power, min, max } => {
            json!({"kind": "MultiHit", "power": power, "min_hits": min, "max_hits": max})
        }
        Effect::FixedDamage(amount) => json!({"kind": "FixedDamage", "amount": amount}),
        Effect::LevelDamage => json!({"kind": "LevelDamage"}),
        Effect::OneHitKnockout => json!({"kind": "Ohko"}),
        Effect::Stats {
            stages,
            chance,
            self_target,
        } => json!({"kind": "Stats", "chance": chance, "self": self_target,
            "stages": stages.iter().map(|(stat, delta)| json!([format!("{stat:?}"), delta])).collect::<Vec<_>>()}),
        Effect::Status {
            status,
            chance,
            replace,
        } => json!({"kind": "Status", "status": format!("{status:?}"), "chance": chance, "replace": replace}),
        Effect::Confuse => json!({"kind": "Confuse"}),
        Effect::Flinch(chance) => json!({"kind": "Flinch", "chance": chance}),
        Effect::Bind => json!({"kind": "Bind"}),
        Effect::Protect => json!({"kind": "Protect"}),
        Effect::Heal {
            fraction,
            hide_message,
        } => json!({"kind": "Heal", "fraction": fraction, "hide_message": hide_message}),
        Effect::Weather { kind, turns } => {
            json!({"kind": "Weather", "weather": format!("{kind:?}"), "turns": turns})
        }
        Effect::TwoTurn {
            power,
            charge_message,
            semi_invulnerable,
            skip_in_sun,
        } => json!({"kind": "TwoTurn", "power": power, "charge_message": charge_message,
            "semi_invulnerable": semi_invulnerable, "skip_in_sun": skip_in_sun}),
        Effect::Consecutive {
            power,
            min,
            max,
            confuse_after,
            double_power,
        } => json!({"kind": "Consecutive", "power": power, "min_turns": min, "max_turns": max,
            "confuse_after": confuse_after, "double_power": double_power}),
        Effect::Splash => json!({"kind": "Splash"}),
    }
}

fn spec(value: &MoveSpec) -> Value {
    json!({
        "id": value.id,
        "name": value.name,
        "type": format!("{:?}", value.move_type),
        "category": format!("{:?}", value.category),
        "pp": value.pp,
        "target": format!("{:?}", value.target),
        "accuracy": accuracy(value.accuracy),
        "priority": value.priority,
        "fail_on_full_hp": value.fail_on_full_hp,
        // A move has either an effects list or a script, never both.
        "scripted": value.effects.is_empty(),
        "effects": value.effects.iter().map(effect).collect::<Vec<_>>(),
    })
}

fn pokemon(value: &Pokemon) -> Value {
    json!({
        "name": value.name,
        "species_id": value.species_id,
        "level": value.level,
        "hp": value.hp,
        "max_hp": value.max_hp,
        "attack": value.attack,
        "defense": value.defense,
        "sp_attack": value.sp_attack,
        "sp_defense": value.sp_defense,
        "speed": value.speed,
        "types": value.types.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        "status": value.status.map(|status| format!("{status:?}")),
        "stages": value.stages.iter().filter(|(_, stage)| **stage != 0)
            .map(|(stat, stage)| json!([format!("{stat:?}"), stage])).collect::<Vec<_>>(),
        "confused": value.confused_turns.is_some(),
        "bound": value.bound.is_some(),
        "protected": value.protected,
        "recharging": value.recharging,
        "hidden": value.semi_invulnerable,
        "locked": value.locked_move.is_some() || value.script_continuation.is_some(),
        "boost": value.next_move_power_boost.map(|boost|
            json!({"type": format!("{:?}", boost.move_type), "multiplier": boost.multiplier})),
        "moves": value.moves,
        "move_pp": value.move_pp,
    })
}

/// Every move in `moves_dir` with its public metadata, or `{"error": …}`.
///
/// # Safety
/// `moves_dir` must be a valid, null-terminated UTF-8 string. Free the result
/// with `wbe_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mlab_catalog(moves_dir: *const c_char) -> *mut c_char {
    if moves_dir.is_null() {
        return output(json!({"error": "null moves directory"}));
    }
    let Ok(directory) = unsafe { CStr::from_ptr(moves_dir) }.to_str() else {
        return output(json!({"error": "moves directory is not UTF-8"}));
    };
    match MoveCatalog::from_directory(directory) {
        Ok(catalog) => {
            let mut moves: Vec<&MoveSpec> =
                catalog.ids().filter_map(|id| catalog.get(id)).collect();
            moves.sort_by(|a, b| a.id.cmp(&b.id));
            output(json!({"moves": moves.into_iter().map(spec).collect::<Vec<_>>()}))
        }
        Err(error) => output(json!({"error": error.to_string()})),
    }
}

/// Weather and the full state of every participant.
///
/// # Safety
/// `battle` must be a live battle returned by `wbe_create`. Free the result
/// with `wbe_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mlab_state(battle: *const Battle) -> *mut c_char {
    let Some(battle) = (unsafe { battle.as_ref() }) else {
        return output(json!({"error": "null battle"}));
    };
    output(json!({
        "turn": battle.turn(),
        "weather": battle.weather().map(|weather|
            json!({"kind": format!("{:?}", weather.kind), "turns_left": weather.turns_left})),
        "allies": battle.participants(Side::Allies).iter().map(pokemon).collect::<Vec<_>>(),
        "foes": battle.participants(Side::Foes).iter().map(pokemon).collect::<Vec<_>>(),
    }))
}
