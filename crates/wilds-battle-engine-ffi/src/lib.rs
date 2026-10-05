use serde::Deserialize;
use serde_json::{Value, json};
use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, Choice, Condition, Gender, MoveCatalog,
    ParticipantId, Pokemon, PokemonType, Side, Status,
};

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

#[derive(Deserialize)]
struct Setup {
    #[serde(default = "one")]
    seed: u64,
    allies: Vec<SetupPokemon>,
    foes: Vec<SetupPokemon>,
    /// Where the battle takes place ("cave", "grass", ...), for moves that depend on it.
    environment: Option<String>,
}
fn one() -> u64 {
    1
}
fn hundred() -> u16 {
    100
}
fn level() -> u8 {
    100
}

#[derive(Deserialize)]
struct SetupPokemon {
    name: String,
    moves: Vec<String>,
    #[serde(default)]
    species_id: u32,
    #[serde(default = "level")]
    level: u8,
    #[serde(default = "hundred")]
    max_hp: u16,
    hp: Option<u16>,
    #[serde(default = "hundred")]
    attack: u16,
    #[serde(default = "hundred")]
    defense: u16,
    #[serde(default = "hundred")]
    sp_attack: u16,
    #[serde(default = "hundred")]
    sp_defense: u16,
    #[serde(default = "hundred")]
    speed: u16,
    #[serde(default)]
    types: Vec<String>,
    status: Option<String>,
    #[serde(default)]
    status_turns: u8,
    #[serde(default)]
    species: String,
    /// "Male", "Female" or "Genderless".
    gender: Option<String>,
    /// Kilograms.
    #[serde(default)]
    weight: f32,
    item: Option<String>,
    ability: Option<String>,
    /// HP, Attack, Defense, Sp. Attack, Sp. Defense, Speed.
    ivs: Option<[u8; 6]>,
    happiness: Option<u8>,
}

fn pokemon(config: SetupPokemon) -> Result<Pokemon, String> {
    let mut value = Pokemon::new(config.name, config.moves);
    value.species_id = config.species_id;
    value.level = config.level;
    value.max_hp = config.max_hp;
    value.hp = config.hp.unwrap_or(config.max_hp);
    value.attack = config.attack;
    value.defense = config.defense;
    value.sp_attack = config.sp_attack;
    value.sp_defense = config.sp_defense;
    value.speed = config.speed;
    if !config.types.is_empty() {
        value.types = config
            .types
            .iter()
            .map(|name| parse_type(name))
            .collect::<Result<_, _>>()?;
    }
    value.status = config.status.as_deref().map(parse_status).transpose()?;
    value.status_turns = config.status_turns;
    value.species = config.species;
    value.gender = match config.gender.as_deref() {
        None | Some("Genderless") => Gender::Genderless,
        Some("Male") => Gender::Male,
        Some("Female") => Gender::Female,
        Some(other) => return Err(format!("unknown gender {other}")),
    };
    value.weight = config.weight;
    value.item = config.item.filter(|item| !item.is_empty());
    value.ability = config.ability.filter(|ability| !ability.is_empty());
    value.ivs = config.ivs;
    value.happiness = config.happiness;
    Ok(value)
}

fn parse_type(name: &str) -> Result<PokemonType, String> {
    use PokemonType::*;
    Ok(match name {
        "Normal" => Normal,
        "Fighting" => Fighting,
        "Flying" => Flying,
        "Poison" => Poison,
        "Ground" => Ground,
        "Rock" => Rock,
        "Bug" => Bug,
        "Ghost" => Ghost,
        "Steel" => Steel,
        "Fire" => Fire,
        "Water" => Water,
        "Grass" => Grass,
        "Electric" => Electric,
        "Psychic" => Psychic,
        "Ice" => Ice,
        "Dragon" => Dragon,
        "Dark" => Dark,
        "Fairy" => Fairy,
        _ => return Err(format!("unknown Pokémon type {name}")),
    })
}
fn parse_status(name: &str) -> Result<Status, String> {
    Ok(match name {
        "Poisoned" => Status::Poisoned,
        "BadlyPoisoned" => Status::BadlyPoisoned,
        "Burned" => Status::Burned,
        "Paralyzed" => Status::Paralyzed,
        "Asleep" => Status::Asleep,
        "Frozen" => Status::Frozen,
        _ => return Err(format!("unknown status {name}")),
    })
}

fn set_error(error: impl ToString) {
    *LAST_ERROR.lock().unwrap() = error.to_string();
}
unsafe fn input<'a>(ptr: *const c_char, label: &str) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err(format!("null {label}"));
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|error| error.to_string())
}
fn output(value: Value) -> *mut c_char {
    CString::new(value.to_string()).unwrap().into_raw()
}
fn guarded<T: Default>(operation: impl FnOnce() -> Result<T, String>) -> T {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(value)) => {
            set_error("");
            value
        }
        Ok(Err(error)) => {
            set_error(error);
            T::default()
        }
        Err(_) => {
            set_error("battle engine panic");
            T::default()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wbe_last_error() -> *mut c_char {
    CString::new(LAST_ERROR.lock().unwrap().as_str())
        .unwrap()
        .into_raw()
}
#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be a string returned by this library, or null, and freed once.
pub unsafe extern "C" fn wbe_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(unsafe { CString::from_raw(ptr) });
    }
}
#[unsafe(no_mangle)]
/// # Safety
/// Both pointers must refer to valid, null-terminated UTF-8 strings.
pub unsafe extern "C" fn wbe_create(
    setup_json: *const c_char,
    moves_dir: *const c_char,
) -> *mut Battle {
    guarded(|| {
        let setup: Setup = serde_json::from_str(unsafe { input(setup_json, "setup")? })
            .map_err(|error| error.to_string())?;
        let catalog = MoveCatalog::from_directory(unsafe { input(moves_dir, "moves directory")? })
            .map_err(|error| error.to_string())?;
        let allies = setup
            .allies
            .into_iter()
            .map(pokemon)
            .collect::<Result<Vec<_>, _>>()?;
        let foes = setup
            .foes
            .into_iter()
            .map(pokemon)
            .collect::<Result<Vec<_>, _>>()?;
        let mut battle =
            Battle::new(setup.seed, [allies, foes], catalog).map_err(|error| error.to_string())?;
        battle.set_environment(setup.environment.filter(|place| !place.is_empty()));
        Ok(Box::into_raw(Box::new(battle)))
    })
}
#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be a live battle returned by `wbe_create`, or null, and destroyed once.
pub unsafe extern "C" fn wbe_destroy(ptr: *mut Battle) {
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}
#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be a live battle returned by `wbe_create`.
pub unsafe extern "C" fn wbe_advance(ptr: *mut Battle) -> *mut c_char {
    guarded(|| {
        let battle = unsafe { ptr.as_mut() }.ok_or("null battle")?;
        let result = battle.advance().map_err(|error| error.to_string())?;
        let status = match result.status {
            AdvanceStatus::End { winner } => json!({"kind":"end", "winner":winner.map(side)}),
            AdvanceStatus::Awaiting(prompt) => json!({"kind":"awaiting", "prompt_id":prompt.id,
                "actor":participant(prompt.actor),
                "choices":prompt.choices.iter().map(choice).collect::<Vec<_>>() }),
        };
        Ok(output(json!({
            "events":result.events.iter().map(event).collect::<Vec<_>>(),
            "status":status, "turn":battle.turn(),
            "allies":battle.participants(Side::Allies).iter().map(snapshot).collect::<Vec<_>>(),
            "foes":battle.participants(Side::Foes).iter().map(snapshot).collect::<Vec<_>>(),
            "weather":battle.weather().map(|weather| format!("{:?}", weather.kind)),
            "field":conditions(battle.field_conditions()),
            "sides":{"allies":conditions(battle.side_conditions(Side::Allies)),
                "foes":conditions(battle.side_conditions(Side::Foes))},
            "payout":{"allies":battle.payout(Side::Allies), "foes":battle.payout(Side::Foes)},
        })))
    })
}
#[unsafe(no_mangle)]
/// # Safety
/// `ptr` must be a live battle returned by `wbe_create`.
pub unsafe extern "C" fn wbe_respond(ptr: *mut Battle, prompt_id: u64, choice_id: u32) -> bool {
    guarded(|| {
        let battle = unsafe { ptr.as_mut() }.ok_or("null battle")?;
        battle
            .set_response(ActionSelection {
                prompt_id,
                choice_id,
            })
            .map_err(|error| error.to_string())?;
        Ok(true)
    })
}
#[unsafe(no_mangle)]
/// # Safety
/// `moves_dir` must refer to a valid, null-terminated UTF-8 string.
pub unsafe extern "C" fn wbe_catalog(moves_dir: *const c_char) -> *mut c_char {
    guarded(|| {
        let catalog = MoveCatalog::from_directory(unsafe { input(moves_dir, "moves directory")? })
            .map_err(|error| error.to_string())?;
        let mut moves = catalog
            .ids()
            .filter_map(|id| catalog.get(id))
            .map(|spec| {
                json!({
                    "id":spec.id, "name":spec.name, "pp":spec.pp,
                })
            })
            .collect::<Vec<_>>();
        moves.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        Ok(output(json!({"moves":moves})))
    })
}

fn side(value: Side) -> &'static str {
    match value {
        Side::Allies => "allies",
        Side::Foes => "foes",
    }
}
fn participant(id: ParticipantId) -> Value {
    json!({"side":side(id.side), "index":id.index})
}
fn choice(value: &Choice) -> Value {
    match value {
        Choice::UseMove {
            id,
            move_id,
            target,
        } => json!({
            "id":id, "kind":"use_move", "move_id":move_id, "target":participant(*target) }),
    }
}
/// The marks and timed effects in one place: name, value and turns left (null if it does not end).
pub fn conditions(values: &[Condition]) -> Value {
    values
        .iter()
        .map(|condition| {
            json!({"name":condition.name, "value":condition.value, "turns":condition.turns_left})
        })
        .collect()
}
fn snapshot(value: &Pokemon) -> Value {
    json!({"name":value.name, "species_id":value.species_id,
        "hp":value.hp, "max_hp":value.max_hp,
        "status":value.status.map(|status| format!("{status:?}")),
        "moves":value.moves, "move_pp":value.move_pp,
        "types":value.types.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        "item":value.item, "hidden":value.hidden().map(|place| format!("{place:?}")),
        "conditions":conditions(&value.conditions)})
}
fn event(value: &BattleEvent) -> Value {
    match value {
        BattleEvent::Message(text) => json!({"kind":"message", "text":text}),
        BattleEvent::Damage { target, amount, hp } => {
            json!({"kind":"damage", "target":participant(*target), "amount":amount, "hp":hp})
        }
        BattleEvent::Heal { target, amount, hp } => {
            json!({"kind":"heal", "target":participant(*target), "amount":amount, "hp":hp})
        }
        BattleEvent::Status { target, status } => {
            json!({"kind":"status", "target":participant(*target), "status":status.map(|status| format!("{status:?}"))})
        }
        BattleEvent::StatChange {
            target,
            stat,
            stages,
        } => {
            json!({"kind":"stat_change", "target":participant(*target), "stat":format!("{stat:?}"), "stages":stages})
        }
        BattleEvent::Weather(weather) => {
            json!({"kind":"weather", "weather":weather.as_ref().map(|w| format!("{:?}",w.kind))})
        }
        BattleEvent::Fainted(target) => json!({"kind":"fainted", "target":participant(*target)}),
        BattleEvent::End { winner } => json!({"kind":"end", "winner":winner.map(side)}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_api_loads_files_and_runs_a_turn() {
        let setup = CString::new(r#"{"allies":[{"name":"Ally","moves":["facade"]}],"foes":[{"name":"Foe","moves":["splash"]}]}"#).unwrap();
        let directory = CString::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../moves")).unwrap();
        let ptr = unsafe { wbe_create(setup.as_ptr(), directory.as_ptr()) };
        assert!(!ptr.is_null());
        let catalog = unsafe { wbe_catalog(directory.as_ptr()) };
        let catalog_json: Value =
            serde_json::from_str(unsafe { CStr::from_ptr(catalog) }.to_str().unwrap()).unwrap();
        assert!(
            catalog_json["moves"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["id"] == "facade")
        );
        unsafe { wbe_free_string(catalog) };
        for _ in 0..2 {
            let json_ptr = unsafe { wbe_advance(ptr) };
            assert!(!json_ptr.is_null());
            let state: Value =
                serde_json::from_str(unsafe { CStr::from_ptr(json_ptr) }.to_str().unwrap())
                    .unwrap();
            unsafe { wbe_free_string(json_ptr) };
            assert_eq!(state["status"]["kind"], "awaiting");
            let choice = state["status"]["choices"][0]["id"].as_u64().unwrap();
            assert!(unsafe {
                wbe_respond(
                    ptr,
                    state["status"]["prompt_id"].as_u64().unwrap(),
                    choice as u32,
                )
            });
        }
        let json_ptr = unsafe { wbe_advance(ptr) };
        let state: Value =
            serde_json::from_str(unsafe { CStr::from_ptr(json_ptr) }.to_str().unwrap()).unwrap();
        assert!(state["foes"][0]["hp"].as_u64().unwrap() < 100);
        unsafe {
            wbe_free_string(json_ptr);
            wbe_destroy(ptr);
        }
    }
}
