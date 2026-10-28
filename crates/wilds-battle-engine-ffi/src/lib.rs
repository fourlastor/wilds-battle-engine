use serde::Deserialize;
use serde_json::{Value, json};
use std::ffi::{CStr, CString, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, Choice, CombatStat, Condition,
    ContextValue, Gender, MessageArg, MoveCatalog, MoveFile, ParticipantId, Pokemon, PokemonForm,
    PokemonType, PromptReason, Side, Status, TerrainKind, WeatherKind,
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
    active: Option<ActiveSetup>,
    #[serde(default)]
    wild_battle: bool,
    #[serde(default)]
    rules: SetupRules,
    /// Items each side may use: {"allies": ["potion"], "foes": []}.
    #[serde(default)]
    bags: SetupBags,
    /// The battle script: Lua hooks and moves, see "Battle scripts" in the README.
    script_lua: Option<String>,
    /// Free-form values about the surroundings (time of day, indoors...), read by scripts.
    #[serde(default)]
    context: std::collections::BTreeMap<String, Value>,
}
#[derive(Deserialize, Default)]
struct SetupBags {
    #[serde(default)]
    allies: Vec<String>,
    #[serde(default)]
    foes: Vec<String>,
}
#[derive(Deserialize, Default)]
struct SetupRules {
    weather: Option<SetupTimed>,
    terrain: Option<SetupTimed>,
    /// Lua list of starting conditions, see `Battle::start_conditions_lua`.
    conditions_lua: Option<String>,
    #[serde(default)]
    no_escape: bool,
}
/// A weather or terrain kind with an optional turn count (none: until replaced).
#[derive(Deserialize)]
struct SetupTimed {
    kind: String,
    turns: Option<u8>,
}
#[derive(Deserialize)]
struct ActiveSetup {
    allies: Vec<usize>,
    foes: Vec<usize>,
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
    base_attack: Option<u16>,
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
    ability_rules_lua: Option<String>,
    ability_bypassable: Option<bool>,
    /// HP, Attack, Defense, Sp. Attack, Sp. Defense, Speed.
    ivs: Option<[u8; 6]>,
    happiness: Option<u8>,
    nature_down: Option<String>,
    form: Option<String>,
    #[serde(default)]
    forms: std::collections::BTreeMap<String, SetupForm>,
    #[serde(default)]
    uncatchable: bool,
    catch_rate: Option<u8>,
    base_speed: Option<u16>,
    /// Facts about this Pokémon for scripts, read as `who.flags.<name>`.
    #[serde(default)]
    flags: std::collections::BTreeSet<String>,
    /// An appearance shown until a script reveals the Pokémon.
    disguise: Option<SetupAppearance>,
    /// Starting stat stages, keyed by "Attack", "Defense", "SpAttack", "SpDefense", "Speed",
    /// "Accuracy" or "Evasion".
    #[serde(default)]
    stages: std::collections::BTreeMap<String, i8>,
    /// Starts confused for this many turns.
    confused_turns: Option<u8>,
    /// PP left per move; missing moves start full.
    #[serde(default)]
    move_pp: std::collections::BTreeMap<String, u8>,
    /// Maximum PP per move when not the move's base PP (PP Ups).
    #[serde(default)]
    pp_max: std::collections::BTreeMap<String, u8>,
}

#[derive(Deserialize)]
struct SetupAppearance {
    name: String,
    #[serde(default)]
    species_id: u32,
    #[serde(default)]
    species: String,
    form: Option<String>,
    gender: Option<String>,
}

#[derive(Deserialize)]
struct SetupForm {
    max_hp: Option<u16>,
    attack: u16,
    defense: u16,
    sp_attack: u16,
    sp_defense: u16,
    speed: u16,
    types: Vec<String>,
    #[serde(default)]
    weight: f32,
}

fn pokemon(config: SetupPokemon) -> Result<Pokemon, String> {
    let mut value = Pokemon::new(config.name, config.moves);
    value.species_id = config.species_id;
    value.base_attack = config.base_attack;
    value.nature_down = config
        .nature_down
        .as_deref()
        .map(|stat| match stat {
            "Attack" => Ok(CombatStat::Attack),
            "Defense" => Ok(CombatStat::Defense),
            "SpAttack" => Ok(CombatStat::SpAttack),
            "SpDefense" => Ok(CombatStat::SpDefense),
            "Speed" => Ok(CombatStat::Speed),
            other => Err(format!("unknown nature_down stat {other}")),
        })
        .transpose()?;
    value.ability_bypassable = config.ability_bypassable;
    if let Some(source) = config.ability_rules_lua {
        value.ability_rules =
            MoveCatalog::rules_from_lua(&source).map_err(|error| error.to_string())?;
    }
    value.form = config.form;
    value.forms = config
        .forms
        .into_iter()
        .map(|(id, form)| {
            Ok((
                id,
                PokemonForm {
                    max_hp: form.max_hp,
                    attack: form.attack,
                    defense: form.defense,
                    sp_attack: form.sp_attack,
                    sp_defense: form.sp_defense,
                    speed: form.speed,
                    weight: form.weight,
                    types: form
                        .types
                        .iter()
                        .map(|kind| parse_type(kind))
                        .collect::<Result<_, _>>()?,
                },
            ))
        })
        .collect::<Result<_, String>>()?;
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
    value.gender = parse_gender(config.gender.as_deref())?;
    value.weight = config.weight;
    value.item = config.item.filter(|item| !item.is_empty());
    value.ability = config.ability.filter(|ability| !ability.is_empty());
    value.ivs = config.ivs;
    value.happiness = config.happiness;
    value.uncatchable = config.uncatchable;
    value.base_speed = config.base_speed;
    value.flags = config.flags;
    value.disguise = config
        .disguise
        .map(|look| {
            Ok::<_, String>(wilds_battle_engine::PokemonAppearance {
                name: look.name,
                species_id: look.species_id,
                species: look.species,
                form: look.form,
                gender: parse_gender(look.gender.as_deref())?,
            })
        })
        .transpose()?;
    value.catch_rate = config.catch_rate;
    for (stat, stages) in config.stages {
        value.stages.insert(parse_stat(&stat)?, stages.clamp(-6, 6));
    }
    value.confused_turns = config.confused_turns.filter(|turns| *turns > 0);
    value.move_pp = config.move_pp;
    value.pp_max = config.pp_max;
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
fn parse_gender(name: Option<&str>) -> Result<Gender, String> {
    Ok(match name {
        None | Some("Genderless") => Gender::Genderless,
        Some("Male") => Gender::Male,
        Some("Female") => Gender::Female,
        Some(other) => return Err(format!("unknown gender {other}")),
    })
}
fn parse_stat(name: &str) -> Result<wilds_battle_engine::Stat, String> {
    use wilds_battle_engine::Stat::*;
    Ok(match name {
        "Attack" => Attack,
        "Defense" => Defense,
        "SpAttack" => SpAttack,
        "SpDefense" => SpDefense,
        "Speed" => Speed,
        "Accuracy" => Accuracy,
        "Evasion" => Evasion,
        _ => return Err(format!("unknown stat {name}")),
    })
}
fn parse_weather(name: &str) -> Result<WeatherKind, String> {
    use WeatherKind::*;
    Ok(match name {
        "Sun" => Sun,
        "Rain" => Rain,
        "Sandstorm" => Sandstorm,
        "Hail" => Hail,
        "HarshSun" => HarshSun,
        "HeavyRain" => HeavyRain,
        "StrongWinds" => StrongWinds,
        _ => return Err(format!("unknown weather {name}")),
    })
}
fn parse_terrain(name: &str) -> Result<TerrainKind, String> {
    use TerrainKind::*;
    Ok(match name {
        "Electric" => Electric,
        "Grassy" => Grassy,
        "Misty" => Misty,
        "Psychic" => Psychic,
        _ => return Err(format!("unknown terrain {name}")),
    })
}
fn context_value(key: &str, value: Value) -> Result<ContextValue, String> {
    Ok(match value {
        Value::Bool(flag) => ContextValue::Flag(flag),
        Value::Number(number) => ContextValue::Number(number.as_f64().unwrap_or_default()),
        Value::String(text) => ContextValue::Text(text),
        _ => return Err(format!("context {key} must be a flag, number or text")),
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
/// Asks the host for the Lua source of a move; see `wbe_move_source` in the C header.
pub type MoveSourceCallback = unsafe extern "C" fn(
    user_data: *mut c_void,
    move_id: *const c_char,
    request: *mut WbeMoveRequest,
) -> bool;

/// Collects the source a host callback supplies with `wbe_move_found`.
pub struct WbeMoveRequest {
    file: Option<MoveFile>,
}

#[unsafe(no_mangle)]
/// # Safety
/// `request` must be the pointer passed to the running callback; `origin` and `text` must be
/// null-terminated UTF-8 strings. Both are copied.
pub unsafe extern "C" fn wbe_move_found(
    request: *mut WbeMoveRequest,
    origin: *const c_char,
    text: *const c_char,
) {
    let (Some(request), Ok(text)) = (unsafe { request.as_mut() }, unsafe { input(text, "text") })
    else {
        return;
    };
    request.file = Some(MoveFile {
        origin: unsafe { input(origin, "origin") }
            .unwrap_or("host move source")
            .to_owned(),
        text: text.to_owned(),
    });
}

/// A host callback as a catalog source.
fn host_source(
    source: Option<MoveSourceCallback>,
    user_data: *mut c_void,
) -> impl Fn(&str) -> Option<MoveFile> + 'static {
    move |id| {
        let source = source?;
        let id = CString::new(id).ok()?;
        let mut request = WbeMoveRequest { file: None };
        // SAFETY: the host keeps the callback and its data valid while the catalog lives.
        let found = unsafe { source(user_data, id.as_ptr(), &mut request) };
        if found { request.file } else { None }
    }
}

/// A catalog over the built-in moves: the host callback first when given, else `moves_dir`.
unsafe fn catalog(
    moves_dir: *const c_char,
    source: Option<MoveSourceCallback>,
    user_data: *mut c_void,
) -> Result<MoveCatalog, String> {
    let catalog = if source.is_some() {
        MoveCatalog::with_source(host_source(source, user_data))
    } else if moves_dir.is_null() {
        MoveCatalog::builtin()
    } else {
        MoveCatalog::from_directory(unsafe { input(moves_dir, "moves directory")? })
    };
    catalog.map_err(|error| error.to_string())
}

unsafe fn create(setup_json: *const c_char, catalog: MoveCatalog) -> Result<*mut Battle, String> {
    let setup: Setup = serde_json::from_str(unsafe { input(setup_json, "setup")? })
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
    battle
        .set_wild_battle(setup.wild_battle)
        .map_err(|error| error.to_string())?;
    if let Some(active) = setup.active {
        battle
            .set_active([active.allies, active.foes])
            .map_err(|error| error.to_string())?;
    }
    battle.set_environment(setup.environment.filter(|place| !place.is_empty()));
    for (key, value) in setup.context {
        let value = context_value(&key, value)?;
        battle.set_context(key, value);
    }
    battle
        .set_bag(Side::Allies, setup.bags.allies)
        .map_err(|error| error.to_string())?;
    battle
        .set_bag(Side::Foes, setup.bags.foes)
        .map_err(|error| error.to_string())?;
    if let Some(source) = setup.script_lua {
        battle
            .set_script(&source)
            .map_err(|error| error.to_string())?;
    }
    let rules = setup.rules;
    battle.set_no_escape(rules.no_escape);
    if let Some(weather) = rules.weather {
        battle
            .set_starting_weather(parse_weather(&weather.kind)?, weather.turns)
            .map_err(|error| error.to_string())?;
    }
    if let Some(terrain) = rules.terrain {
        battle
            .set_starting_terrain(parse_terrain(&terrain.kind)?, terrain.turns)
            .map_err(|error| error.to_string())?;
    }
    if let Some(source) = rules.conditions_lua {
        battle
            .start_conditions_lua(&source)
            .map_err(|error| error.to_string())?;
    }
    Ok(Box::into_raw(Box::new(battle)))
}

#[unsafe(no_mangle)]
/// # Safety
/// `setup_json` must be a valid, null-terminated UTF-8 string; `moves_dir` too, or null.
pub unsafe extern "C" fn wbe_create(
    setup_json: *const c_char,
    moves_dir: *const c_char,
) -> *mut Battle {
    guarded(|| unsafe { create(setup_json, catalog(moves_dir, None, std::ptr::null_mut())?) })
}

#[unsafe(no_mangle)]
/// # Safety
/// `source` and `user_data` must stay valid until the catalog and every battle created from it
/// are destroyed. Catalogs and their battles must be used from one thread.
pub unsafe extern "C" fn wbe_catalog_create(
    source: Option<MoveSourceCallback>,
    user_data: *mut c_void,
) -> *mut MoveCatalog {
    guarded(|| {
        let catalog = unsafe { catalog(std::ptr::null(), source, user_data)? };
        Ok(Box::into_raw(Box::new(catalog)))
    })
}

#[unsafe(no_mangle)]
/// # Safety
/// As `wbe_catalog_create`; `item_source` (which may be null) supplies item scripts the same way
/// `move_source` supplies moves, both over the built-in ones.
pub unsafe extern "C" fn wbe_catalog_create_with_items(
    move_source: Option<MoveSourceCallback>,
    item_source: Option<MoveSourceCallback>,
    user_data: *mut c_void,
) -> *mut MoveCatalog {
    guarded(|| {
        let catalog = MoveCatalog::with_sources(
            host_source(move_source, user_data),
            host_source(item_source, user_data),
        )
        .map_err(|error| error.to_string())?;
        Ok(Box::into_raw(Box::new(catalog)))
    })
}

#[unsafe(no_mangle)]
/// # Safety
/// `catalog` must be a live catalog; `item_id` a valid, null-terminated UTF-8 string.
pub unsafe extern "C" fn wbe_catalog_item(
    catalog: *const MoveCatalog,
    item_id: *const c_char,
) -> *mut c_char {
    guarded(|| {
        let catalog = unsafe { catalog.as_ref() }.ok_or("null catalog")?;
        let item = catalog
            .load_item(unsafe { input(item_id, "item id")? })
            .map_err(|error| error.to_string())?;
        Ok(output(item.map_or(Value::Null, |item| {
            json!({"id":item.id, "name":item.name,
                "targets":match item.targets {
                wilds_battle_engine::ItemTargets::User => "user",
                wilds_battle_engine::ItemTargets::Party => "party",
                wilds_battle_engine::ItemTargets::PartyMove => "party_move",
                wilds_battle_engine::ItemTargets::Fainted => "fainted",
                wilds_battle_engine::ItemTargets::Foe => "foe",
                wilds_battle_engine::ItemTargets::None => "none",
            }})
        })))
    })
}

#[unsafe(no_mangle)]
/// # Safety
/// `battle` must be a live battle; `side` is "allies" or "foes" and `items_json` a JSON array
/// of item IDs, both null-terminated UTF-8 strings.
pub unsafe extern "C" fn wbe_set_bag(
    battle: *mut Battle,
    side: *const c_char,
    items_json: *const c_char,
) -> bool {
    guarded(|| {
        let battle = unsafe { battle.as_mut() }.ok_or("null battle")?;
        let side = match unsafe { input(side, "side")? } {
            "allies" => Side::Allies,
            "foes" => Side::Foes,
            other => return Err(format!("unknown side {other}")),
        };
        let items: Vec<String> = serde_json::from_str(unsafe { input(items_json, "items")? })
            .map_err(|error| error.to_string())?;
        battle
            .set_bag(side, items)
            .map_err(|error| error.to_string())?;
        Ok(true)
    })
}

#[unsafe(no_mangle)]
/// # Safety
/// `catalog` must be a live catalog returned by `wbe_catalog_create`, or null, and destroyed once.
pub unsafe extern "C" fn wbe_catalog_destroy(catalog: *mut MoveCatalog) {
    if !catalog.is_null() {
        drop(unsafe { Box::from_raw(catalog) });
    }
}

#[unsafe(no_mangle)]
/// # Safety
/// `catalog` must be a live catalog; `move_id` a valid, null-terminated UTF-8 string.
pub unsafe extern "C" fn wbe_catalog_move(
    catalog: *const MoveCatalog,
    move_id: *const c_char,
) -> *mut c_char {
    guarded(|| {
        let catalog = unsafe { catalog.as_ref() }.ok_or("null catalog")?;
        let spec = catalog
            .load(unsafe { input(move_id, "move id")? })
            .map_err(|error| error.to_string())?;
        Ok(output(spec.map_or(Value::Null, move_info)))
    })
}

#[unsafe(no_mangle)]
/// # Safety
/// `setup_json` must be a valid, null-terminated UTF-8 string and `catalog` a live catalog. The
/// battle shares the moves the catalog loaded; the catalog can be destroyed before it.
pub unsafe extern "C" fn wbe_create_with_catalog(
    setup_json: *const c_char,
    catalog: *const MoveCatalog,
) -> *mut Battle {
    guarded(|| {
        let catalog = unsafe { catalog.as_ref() }.ok_or("null catalog")?.clone();
        unsafe { create(setup_json, catalog) }
    })
}

fn move_info(spec: &wilds_battle_engine::MoveSpec) -> Value {
    json!({"id":spec.id, "name":spec.name, "pp":spec.pp})
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
            AdvanceStatus::Escaped { side: escaped } => {
                json!({"kind":"escaped", "side":side(escaped)})
            }
            AdvanceStatus::End { winner } => json!({"kind":"end", "winner":winner.map(side)}),
            AdvanceStatus::Captured { target } => {
                json!({"kind":"captured", "target":participant(target)})
            }
            AdvanceStatus::Awaiting(prompt) => json!({"kind":"awaiting", "prompt_id":prompt.id,
                "actor":participant(prompt.actor), "reason": match prompt.reason { PromptReason::Action => "action", PromptReason::Replacement => "replacement", PromptReason::Pivot => "pivot" },
                "choices":prompt.choices.iter().map(choice).collect::<Vec<_>>() }),
        };
        Ok(output(json!({
            "events":result.events.iter().map(event).collect::<Vec<_>>(),
            "status":status, "turn":battle.turn(),
            "active":{"allies":battle.active_indices(Side::Allies),"foes":battle.active_indices(Side::Foes)},
            "allies":snapshots(battle, Side::Allies),
            "foes":snapshots(battle, Side::Foes),
            "weather":battle.weather().map(|weather| format!("{:?}", weather.kind)),
            "no_escape":battle.no_escape(),
            "field":conditions(battle.field_conditions()),
            "sides":{"allies":conditions(battle.side_conditions(Side::Allies)),
                "foes":conditions(battle.side_conditions(Side::Foes))},
            "payout":{"allies":battle.payout(Side::Allies), "foes":battle.payout(Side::Foes)},
            "payout_multiplier":{"allies":battle.payout_multiplier(Side::Allies), "foes":battle.payout_multiplier(Side::Foes)},
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
/// `moves_dir` must refer to a valid, null-terminated UTF-8 string, or be null.
pub unsafe extern "C" fn wbe_catalog(moves_dir: *const c_char) -> *mut c_char {
    guarded(|| {
        let catalog = unsafe { catalog(moves_dir, None, std::ptr::null_mut())? };
        let mut moves = catalog
            .ids()
            .map(|id| catalog.load(id).map(|spec| spec.map(move_info)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
            .into_iter()
            .flatten()
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
pub(crate) fn participant(id: ParticipantId) -> Value {
    json!({"side":side(id.side), "index":id.index})
}
fn choice(value: &Choice) -> Value {
    match value {
        Choice::Escape { id } => json!({"id":id, "kind":"escape"}),
        Choice::Continue { id } => json!({"id":id, "kind":"continue"}),
        Choice::Undo { id } => json!({"id":id, "kind":"undo"}),
        Choice::UseItem {
            id,
            item,
            target,
            move_id,
        } => json!({"id":id, "kind":"use_item", "item":item, "target":participant(*target),
            "move_id":move_id}),
        Choice::Switch { id, to } => json!({"id":id, "kind":"switch", "to":participant(*to)}),
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
fn appearance(value: &wilds_battle_engine::PokemonAppearance) -> Value {
    json!({"name":value.name, "species_id":value.species_id, "species":value.species,
        "form":value.form, "gender":format!("{:?}", value.gender)})
}
/// Each participant's state, plus what needs the battle: maximum PP and opponents faced.
fn snapshots(battle: &Battle, side: Side) -> Vec<Value> {
    battle
        .participants(side)
        .iter()
        .enumerate()
        .map(|(index, pokemon)| {
            let id = ParticipantId { side, index };
            let mut value = snapshot(pokemon);
            value["pp_max"] = pokemon
                .moves
                .iter()
                .filter_map(|move_id| Some((move_id.clone(), json!(battle.max_pp(id, move_id)?))))
                .collect::<serde_json::Map<_, _>>()
                .into();
            value["faced"] = json!(battle.faced_by(id));
            value
        })
        .collect()
}
fn snapshot(value: &Pokemon) -> Value {
    json!({"name":value.name, "display_name":value.display_name(), "illusion":value.illusion.as_ref().map(appearance), "disguise":value.disguise.as_ref().map(appearance), "species_id":value.species_id, "base_attack":value.base_attack, "transformed": value.transformed(), "form":value.form,
        "attack":value.attack, "defense":value.defense, "sp_attack":value.sp_attack,
        "sp_defense":value.sp_defense, "speed":value.speed, "weight":value.weight,
        "hp":value.hp, "max_hp":value.max_hp,
        "substitute_hp":value.substitute_hp,
        "status":value.status.map(|status| format!("{status:?}")),
        "moves":value.moves, "move_pp":value.move_pp,
        "types":value.types.iter().copied().chain(value.added_type).map(|kind| format!("{kind:?}")).collect::<Vec<_>>(),
        "added_type":value.added_type.map(|kind| format!("{kind:?}")),
        "ate_berry":value.ate_berry, "consumed_items":value.consumed_items, "last_consumed_item":value.last_consumed_item, "recyclable_item":value.recyclable_item,
        "nature_down":value.nature_down.map(|stat| format!("{stat:?}")),
        "ability":value.ability, "ability_suppressed":value.ability_suppressed,
        "carrying":value.carrying.map(participant), "carried_by":value.carried_by.map(participant),
        "item":value.item, "hidden":value.hidden().map(|place| format!("{place:?}")),
        "uncatchable":value.uncatchable, "catch_rate":value.catch_rate, "flags":value.flags,
        "stages":value.stages.iter().filter(|(_, stages)| **stages != 0)
            .map(|(stat, stages)| (format!("{stat:?}"), json!(stages))).collect::<serde_json::Map<_, _>>(),
        "confused_turns":value.confused_turns,
        "conditions":conditions(&value.conditions)})
}
/// `{"pokemon":{"side","index"},"name","gender"}`, `{"move":id,"name"}`, `{"text"}` or `{"number"}`.
fn message_arg(arg: &MessageArg) -> Value {
    match arg {
        MessageArg::Pokemon { id, name, gender } => {
            json!({"pokemon":participant(*id), "name":name, "gender":format!("{gender:?}")})
        }
        MessageArg::Move { id, name } => json!({"move":id, "name":name}),
        MessageArg::Text(text) => json!({"text":text}),
        MessageArg::Number(number) => json!({"number":number}),
    }
}
fn event(value: &BattleEvent) -> Value {
    match value {
        BattleEvent::MoveUsed {
            actor,
            move_id,
            targets,
        } => json!({"kind":"move_used", "actor":participant(*actor), "move":move_id,
            "targets":targets.iter().copied().map(participant).collect::<Vec<_>>()}),
        BattleEvent::Hit {
            actor,
            target,
            hit,
            effectiveness_percent,
            critical,
        } => json!({"kind":"hit", "actor":participant(*actor), "target":participant(*target),
            "hit":hit, "effectiveness":f64::from(*effectiveness_percent) / 100.0, "critical":critical}),
        BattleEvent::Miss { actor, target } => {
            json!({"kind":"miss", "actor":participant(*actor), "target":participant(*target)})
        }
        BattleEvent::ItemUsed {
            actor,
            item,
            target,
            consumed,
        } => json!({"kind":"item_used", "actor":participant(*actor), "item":item,
            "target":participant(*target), "consumed":consumed}),
        BattleEvent::Capture {
            actor,
            target,
            ball,
            shakes,
            critical,
            caught,
        } => json!({"kind":"capture", "actor":participant(*actor), "target":participant(*target),
            "ball":ball, "shakes":shakes, "critical":critical, "caught":caught}),
        BattleEvent::CaptureRefused {
            actor,
            target,
            ball,
        } => json!({"kind":"capture_refused", "actor":participant(*actor),
            "target":participant(*target), "ball":ball}),
        BattleEvent::Script { key, args } => json!({"kind":"script", "key":key,
            "args":args.iter().map(message_arg).collect::<Vec<_>>()}),
        BattleEvent::Charging { actor, move_id } => {
            json!({"kind":"charging", "actor":participant(*actor), "move":move_id})
        }
        BattleEvent::Hidden { target, place } => json!({"kind":"hidden",
            "target":participant(*target), "place":place.map(|place| format!("{place:?}"))}),
        BattleEvent::EscapeAttempt { actor, success } => {
            json!({"kind":"escape_attempt", "actor":participant(*actor), "success":success})
        }
        BattleEvent::HpSet { target, hp } => {
            json!({"kind":"hp_set", "target":participant(*target), "hp":hp})
        }
        BattleEvent::PositionsSwapped { a, b } => {
            json!({"kind":"swap_positions", "a":participant(*a), "b":participant(*b)})
        }
        BattleEvent::Switched { from, to, position } => {
            json!({"kind":"switch", "from":participant(*from),"to":participant(*to),"position":position})
        }
        BattleEvent::Message(message) => json!({"kind":"message", "text":message.text(),
            "key":message.key, "args":message.args.iter().map(message_arg).collect::<Vec<_>>(),
            "context":message.context().map(|gender| format!("{gender:?}"))}),
        BattleEvent::Damage { target, amount, hp } => {
            json!({"kind":"damage", "target":participant(*target), "amount":amount, "hp":hp})
        }
        BattleEvent::Substitute { target, amount, hp } => {
            json!({"kind":"substitute", "target":participant(*target), "amount":amount, "hp":hp})
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
        BattleEvent::AppearanceChanged { target, illusion } => {
            json!({"kind":"appearance", "target":participant(*target), "illusion":illusion.as_ref().map(appearance)})
        }
        BattleEvent::FormChanged { target, form } => {
            json!({"kind":"form", "target":participant(*target), "form":form})
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

    unsafe extern "C" fn modded_tackle(
        user_data: *mut c_void,
        move_id: *const c_char,
        request: *mut WbeMoveRequest,
    ) -> bool {
        let asked = unsafe { &mut *(user_data as *mut Vec<String>) };
        let id = unsafe { CStr::from_ptr(move_id) }.to_str().unwrap();
        asked.push(id.to_owned());
        if id != "tackle" {
            return false;
        }
        let origin = CString::new("mods/test/moves/tackle.lua").unwrap();
        let text = CString::new(
            r#"return {id="tackle", name="Modded Tackle", type=Type.Normal,
                category=Category.Physical, pp=7, effects={{kind=Effect.Damage, power=40}}}"#,
        )
        .unwrap();
        unsafe { wbe_move_found(request, origin.as_ptr(), text.as_ptr()) };
        true
    }

    fn take(ptr: *mut c_char) -> Value {
        assert!(!ptr.is_null(), "{}", take_error());
        let value = serde_json::from_str(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap()).unwrap();
        unsafe { wbe_free_string(ptr) };
        value
    }

    fn take_error() -> String {
        let ptr = wbe_last_error();
        let text = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned();
        unsafe { wbe_free_string(ptr) };
        text
    }

    #[test]
    fn c_api_reports_moves_hits_and_participant_details() {
        let setup = CString::new(
            r#"{"allies":[{"name":"Ally","moves":["tackle"],"pp_max":{"tackle":56},"stages":{"Attack":1}}],
                "foes":[{"name":"Foe","moves":["splash"],"confused_turns":2}]}"#,
        )
        .unwrap();
        let battle = unsafe { wbe_create(setup.as_ptr(), std::ptr::null()) };
        assert!(!battle.is_null(), "{}", take_error());
        for _ in 0..2 {
            let state = take(unsafe { wbe_advance(battle) });
            assert!(unsafe {
                wbe_respond(battle, state["status"]["prompt_id"].as_u64().unwrap(), 0)
            });
        }
        let state = take(unsafe { wbe_advance(battle) });
        let events = state["events"].as_array().unwrap();
        let kind = |kind: &str| events.iter().find(|e| e["kind"] == kind).cloned();
        let used: Vec<_> = events.iter().filter(|e| e["kind"] == "move_used").collect();
        assert_eq!(
            *used
                .iter()
                .find(|e| e["move"] == "tackle")
                .copied()
                .unwrap(),
            json!({"kind":"move_used", "actor":{"side":"allies","index":0}, "move":"tackle",
                "targets":[{"side":"foes","index":0}]})
        );
        let hit = kind("hit").unwrap();
        assert_eq!(hit["hit"], 1);
        assert_eq!(hit["effectiveness"], 1.0);
        assert_eq!(state["allies"][0]["pp_max"]["tackle"], 56);
        assert_eq!(state["allies"][0]["stages"]["Attack"], 1);
        assert_eq!(state["allies"][0]["faced"], json!([0]));
        unsafe { wbe_destroy(battle) };
    }

    #[test]
    fn c_api_offers_bag_items_and_reports_their_use() {
        let setup = CString::new(
            r#"{"allies":[{"name":"Ally","moves":["splash"],"max_hp":100,"hp":40}],
                "foes":[{"name":"Foe","moves":["splash"],"disguise":{"name":"Decoy"}}],
                "bags":{"allies":["potion"]}}"#,
        )
        .unwrap();
        let battle = unsafe { wbe_create(setup.as_ptr(), std::ptr::null()) };
        assert!(!battle.is_null(), "{}", take_error());
        let state = take(unsafe { wbe_advance(battle) });
        assert_eq!(state["foes"][0]["display_name"], "Decoy");
        let potion = state["status"]["choices"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == "use_item")
            .cloned()
            .unwrap();
        assert_eq!(potion["item"], "potion");
        assert_eq!(potion["move_id"], Value::Null);
        assert!(unsafe {
            wbe_respond(
                battle,
                state["status"]["prompt_id"].as_u64().unwrap(),
                potion["id"].as_u64().unwrap() as u32,
            )
        });
        let state = take(unsafe { wbe_advance(battle) });
        assert!(unsafe { wbe_respond(battle, state["status"]["prompt_id"].as_u64().unwrap(), 0) });
        let state = take(unsafe { wbe_advance(battle) });
        assert!(
            state["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "item_used"
                    && e["item"] == "potion"
                    && e["consumed"] == true)
        );
        assert_eq!(state["allies"][0]["hp"], 60);
        let side = CString::new("allies").unwrap();
        let items = CString::new(r#"["revive"]"#).unwrap();
        assert!(unsafe { wbe_set_bag(battle, side.as_ptr(), items.as_ptr()) });
        unsafe { wbe_destroy(battle) };
    }

    #[test]
    fn c_api_runs_battle_scripts_and_reports_their_events() {
        let setup = CString::new(
            json!({"allies":[{"name":"Ally","moves":["splash"]}],
                "foes":[{"name":"Foe","moves":["splash"]}],
                "script_lua":"return {on_start = function(ctx) ctx:emit('intro', ctx.user, 3) end}"})
            .to_string(),
        )
        .unwrap();
        let battle = unsafe { wbe_create(setup.as_ptr(), std::ptr::null()) };
        assert!(!battle.is_null(), "{}", take_error());
        let state = take(unsafe { wbe_advance(battle) });
        let script = state["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["kind"] == "script")
            .cloned()
            .unwrap();
        assert_eq!(script["key"], "intro");
        assert_eq!(script["args"][0]["pokemon"]["side"], "foes");
        assert_eq!(script["args"][1]["number"], 3);
        unsafe { wbe_destroy(battle) };
    }

    #[test]
    fn messages_carry_their_key_arguments_and_context() {
        let value = event(&BattleEvent::Message(wilds_battle_engine::Message::new(
            "{0} used {1}!",
            [
                MessageArg::Pokemon {
                    id: ParticipantId {
                        side: Side::Foes,
                        index: 1,
                    },
                    name: "Gengar".into(),
                    gender: Gender::Female,
                },
                MessageArg::Move {
                    id: "shadow-claw".into(),
                    name: "Shadow Claw".into(),
                },
            ],
        )));
        assert_eq!(
            value,
            json!({"kind":"message", "text":"Gengar used Shadow Claw!", "key":"{0} used {1}!",
                "context":"Female", "args":[
                    {"pokemon":{"side":"foes","index":1}, "name":"Gengar", "gender":"Female"},
                    {"move":"shadow-claw", "name":"Shadow Claw"}]})
        );
    }

    #[test]
    fn c_api_catalog_asks_the_host_source_before_builtin_moves() {
        let mut asked: Vec<String> = Vec::new();
        let catalog = unsafe {
            wbe_catalog_create(
                Some(modded_tackle),
                &mut asked as *mut Vec<String> as *mut c_void,
            )
        };
        assert!(!catalog.is_null());
        let lookup = |id: &str| {
            let id = CString::new(id).unwrap();
            take(unsafe { wbe_catalog_move(catalog, id.as_ptr()) })
        };
        assert_eq!(lookup("tackle")["name"], "Modded Tackle");
        assert_eq!(lookup("growl")["name"], "Growl");
        assert_eq!(lookup("not-a-move"), Value::Null);

        let setup = CString::new(
            r#"{"allies":[{"name":"Ally","moves":["tackle"]}],"foes":[{"name":"Foe","moves":["growl"]}]}"#,
        )
        .unwrap();
        let battle = unsafe { wbe_create_with_catalog(setup.as_ptr(), catalog) };
        unsafe { wbe_catalog_destroy(catalog) };
        assert!(!battle.is_null(), "{}", take_error());
        let state = take(unsafe { wbe_advance(battle) });
        assert_eq!(state["allies"][0]["move_pp"]["tackle"], 7);
        unsafe { wbe_destroy(battle) };
        assert_eq!(
            asked,
            ["tackle", "growl", "not-a-move"],
            "each move is asked for once"
        );
    }

    #[test]
    fn c_api_offers_opt_in_escape_and_returns_a_persistent_escape_outcome() {
        for wild in [false, true] {
            let mut setup = json!({
                "allies":[{"name":"Runner","moves":["tackle"],"speed":1,"ability":"run-away"}],
                "foes":[{"name":"Foe","moves":["tackle"],"speed":500}]
            });
            if wild {
                setup["wild_battle"] = json!(true);
            }
            let setup = CString::new(setup.to_string()).unwrap();
            let directory =
                CString::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../moves")).unwrap();
            let battle = unsafe { wbe_create(setup.as_ptr(), directory.as_ptr()) };
            assert!(!battle.is_null());
            let read = || {
                let ptr = unsafe { wbe_advance(battle) };
                assert!(!ptr.is_null());
                let value: Value =
                    serde_json::from_str(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap()).unwrap();
                unsafe {
                    wbe_free_string(ptr);
                }
                value
            };
            let value = read();
            let choices = value["status"]["choices"].as_array().unwrap();
            let escape = choices.iter().find(|c| c["kind"] == "escape");
            assert_eq!(escape.is_some(), wild);
            if let Some(escape) = escape {
                assert!(unsafe {
                    wbe_respond(
                        battle,
                        value["status"]["prompt_id"].as_u64().unwrap(),
                        escape["id"].as_u64().unwrap() as u32,
                    )
                });
                let foe = read();
                assert_eq!(foe["status"]["actor"]["side"], "foes");
                assert!(unsafe {
                    wbe_respond(battle, foe["status"]["prompt_id"].as_u64().unwrap(), 0)
                });
                let result = read();
                assert_eq!(result["status"], json!({"kind":"escaped","side":"allies"}));
                assert_eq!(result["allies"][0]["hp"], 100);
                assert!(
                    result["events"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|e| e["kind"] == "escape_attempt" && e["success"] == true)
                );
                let again = read();
                assert_eq!(again["status"], result["status"]);
                assert_eq!(again["events"], json!([]));
            }
            unsafe {
                wbe_destroy(battle);
            }
        }
        assert_eq!(
            choice(&Choice::Continue { id: 0 }),
            json!({"id":0,"kind":"continue"})
        );
        assert_eq!(
            event(&BattleEvent::EscapeAttempt {
                actor: ParticipantId {
                    side: Side::Allies,
                    index: 0
                },
                success: false
            })["success"],
            false
        );
    }

    #[test]
    fn illusion_snapshot_and_events_keep_real_and_apparent_identity_separate() {
        let mut p = Pokemon::new("Zoroark", vec!["splash".into()]);
        p.species_id = 571;
        let disguise = wilds_battle_engine::PokemonAppearance {
            name: "Pikachu".into(),
            species_id: 25,
            species: "pikachu".into(),
            form: None,
            gender: wilds_battle_engine::Gender::Male,
        };
        p.illusion = Some(disguise.clone());
        let state = snapshot(&p);
        assert_eq!(state["name"], "Zoroark");
        assert_eq!(state["species_id"], 571);
        assert_eq!(state["display_name"], "Pikachu");
        assert_eq!(state["illusion"]["species_id"], 25);
        let value = event(&BattleEvent::AppearanceChanged {
            target: ParticipantId {
                side: Side::Allies,
                index: 0,
            },
            illusion: Some(disguise),
        });
        assert_eq!(value["kind"], "appearance");
        assert_eq!(value["illusion"]["name"], "Pikachu");
    }

    #[test]
    fn setup_keeps_species_base_attack_separate_from_computed_attack() {
        let config = serde_json::from_value(
            json!({"name":"Party member", "moves":["beat-up"], "attack":300, "base_attack":55}),
        )
        .unwrap();
        let value = pokemon(config).unwrap();
        assert_eq!(value.attack, 300);
        assert_eq!(value.base_attack, Some(55));
        assert_eq!(snapshot(&value)["base_attack"], 55);
    }

    #[test]
    fn c_api_exposes_bench_replacement_choices_and_switch_events() {
        let setup = CString::new(r#"{"allies":[{"name":"Fainted","hp":0,"moves":["splash"]},{"name":"Reserve","moves":["splash"]}],"foes":[{"name":"Foe","moves":["splash"]}],"active":{"allies":[0],"foes":[0]}}"#).unwrap();
        let directory = CString::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../moves")).unwrap();
        let battle = unsafe { wbe_create(setup.as_ptr(), directory.as_ptr()) };
        assert!(!battle.is_null());
        let read = || {
            let ptr = unsafe { wbe_advance(battle) };
            let value: Value =
                serde_json::from_str(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap()).unwrap();
            unsafe {
                wbe_free_string(ptr);
            }
            value
        };
        let value = read();
        assert_eq!(value["status"]["reason"], "replacement");
        assert_eq!(value["status"]["choices"][0]["kind"], "switch");
        assert_eq!(value["status"]["choices"][0]["to"]["index"], 1);
        assert!(unsafe { wbe_respond(battle, value["status"]["prompt_id"].as_u64().unwrap(), 0) });
        let value = read();
        assert_eq!(value["active"]["allies"], json!([1]));
        assert_eq!(value["status"]["actor"]["index"], 1);
        assert_eq!(value["status"]["reason"], "action");
        assert!(
            value["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["kind"] == "switch")
        );
        unsafe {
            wbe_destroy(battle);
        }
    }

    #[test]
    fn setup_accepts_typed_ability_rules_and_bypass_metadata() {
        let config = serde_json::from_value(json!({"name":"Target", "moves":["splash"],
            "ability":"host-shield", "ability_bypassable":true,
            "ability_rules_lua":"return {{kind=Rule.DamageTaken, factor=0.5}}"}))
        .unwrap();
        let pokemon = pokemon(config).unwrap();
        assert_eq!(pokemon.ability_bypassable, Some(true));
        assert!(matches!(
            pokemon.ability_rules[0],
            wilds_battle_engine::Rule::DamageTaken { factor: 0.5, .. }
        ));
    }

    #[test]
    fn c_api_accepts_forms_and_reports_transformation() {
        let setup = CString::new(
            json!({
                "allies": [{"name":"Singer", "species_id":648, "moves":["relic-song"],
                    "forms":{"meloetta-pirouette":{"attack":315,"defense":220,
                        "sp_attack":177,"sp_defense":190,"speed":320,
                        "types":["Normal","Fighting"],"weight":6.5}}}],
                "foes": [{"name":"Target", "max_hp":1000, "moves":["splash"]}]
            })
            .to_string(),
        )
        .unwrap();
        let directory = CString::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../moves")).unwrap();
        let ptr = unsafe { wbe_create(setup.as_ptr(), directory.as_ptr()) };
        assert!(!ptr.is_null());
        for _ in 0..2 {
            let output = unsafe { wbe_advance(ptr) };
            let value: Value =
                serde_json::from_str(unsafe { CStr::from_ptr(output) }.to_str().unwrap()).unwrap();
            unsafe {
                wbe_free_string(output);
            }
            assert!(unsafe { wbe_respond(ptr, value["status"]["prompt_id"].as_u64().unwrap(), 0) });
        }
        let output = unsafe { wbe_advance(ptr) };
        let value: Value =
            serde_json::from_str(unsafe { CStr::from_ptr(output) }.to_str().unwrap()).unwrap();
        assert_eq!(value["allies"][0]["form"], "meloetta-pirouette");
        assert_eq!(value["allies"][0]["attack"], 315);
        assert!(
            value["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["kind"] == "form")
        );
        unsafe {
            wbe_free_string(output);
            wbe_destroy(ptr);
        }
    }

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
