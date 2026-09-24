use crate::model::{AppliedStatus, BattleError, PokemonType, Stat, Status, WeatherKind};
use crate::moves::{Category, Target};
use mlua::{AnyUserData, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EffectKind {
    Damage,
    MultiHit,
    FixedDamage,
    LevelDamage,
    Ohko,
    Stats,
    Status,
    Confuse,
    Flinch,
    Bind,
    Protect,
    Heal,
    Weather,
    TwoTurn,
    Consecutive,
    Splash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AccuracyKind {
    Always,
    Ohko,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LuaSymbol {
    Type(PokemonType),
    Stat(Stat),
    Status(AppliedStatus),
    Weather(WeatherKind),
    Category(Category),
    Target(Target),
    Effect(EffectKind),
    Accuracy(AccuracyKind),
}

impl UserData for LuaSymbol {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Eq, |_, this, other: AnyUserData| {
            Ok(other.borrow::<Self>().is_ok_and(|value| *value == *this))
        });
    }
}

fn add(lua: &Lua, table: &Table, name: &str, value: LuaSymbol) -> Result<(), BattleError> {
    table.set(name, lua.create_userdata(value)?)?;
    Ok(())
}

pub(crate) fn install(lua: &Lua) -> Result<(), BattleError> {
    use AppliedStatus as S;
    use EffectKind as E;
    use PokemonType as P;
    use Stat as St;
    let globals = lua.globals();
    let types = lua.create_table()?;
    for (name, value) in [
        ("Normal", P::Normal),
        ("Fighting", P::Fighting),
        ("Flying", P::Flying),
        ("Poison", P::Poison),
        ("Ground", P::Ground),
        ("Rock", P::Rock),
        ("Bug", P::Bug),
        ("Ghost", P::Ghost),
        ("Steel", P::Steel),
        ("Fire", P::Fire),
        ("Water", P::Water),
        ("Grass", P::Grass),
        ("Electric", P::Electric),
        ("Psychic", P::Psychic),
        ("Ice", P::Ice),
        ("Dragon", P::Dragon),
        ("Dark", P::Dark),
        ("Fairy", P::Fairy),
        ("None", P::None),
    ] {
        add(lua, &types, name, LuaSymbol::Type(value))?;
    }
    globals.set("Type", types)?;

    let stats = lua.create_table()?;
    for (name, value) in [
        ("Attack", St::Attack),
        ("Defense", St::Defense),
        ("SpAttack", St::SpAttack),
        ("SpDefense", St::SpDefense),
        ("Speed", St::Speed),
        ("Accuracy", St::Accuracy),
        ("Evasion", St::Evasion),
    ] {
        add(lua, &stats, name, LuaSymbol::Stat(value))?;
    }
    globals.set("Stat", stats)?;

    let statuses = lua.create_table()?;
    for (name, value) in [
        ("Poisoned", S::Poisoned),
        ("BadlyPoisoned", S::BadlyPoisoned),
        ("Burned", S::Burned),
        ("Paralyzed", S::Paralyzed),
        ("Asleep", S::Asleep),
        ("RestSleep", S::RestSleep),
        ("Frozen", S::Frozen),
    ] {
        add(lua, &statuses, name, LuaSymbol::Status(value))?;
    }
    globals.set("Status", statuses)?;

    let weather = lua.create_table()?;
    add(lua, &weather, "Sun", LuaSymbol::Weather(WeatherKind::Sun))?;
    add(
        lua,
        &weather,
        "Sandstorm",
        LuaSymbol::Weather(WeatherKind::Sandstorm),
    )?;
    globals.set("Weather", weather)?;

    let categories = lua.create_table()?;
    for (name, value) in [
        ("Physical", Category::Physical),
        ("Special", Category::Special),
        ("Status", Category::Status),
    ] {
        add(lua, &categories, name, LuaSymbol::Category(value))?;
    }
    globals.set("Category", categories)?;

    let targets = lua.create_table()?;
    for (name, value) in [
        ("Selected", Target::Selected),
        ("User", Target::User),
        ("Field", Target::Field),
        ("AllOthers", Target::AllOthers),
    ] {
        add(lua, &targets, name, LuaSymbol::Target(value))?;
    }
    globals.set("Target", targets)?;

    let effects = lua.create_table()?;
    for (name, value) in [
        ("Damage", E::Damage),
        ("MultiHit", E::MultiHit),
        ("FixedDamage", E::FixedDamage),
        ("LevelDamage", E::LevelDamage),
        ("Ohko", E::Ohko),
        ("Stats", E::Stats),
        ("Status", E::Status),
        ("Confuse", E::Confuse),
        ("Flinch", E::Flinch),
        ("Bind", E::Bind),
        ("Protect", E::Protect),
        ("Heal", E::Heal),
        ("Weather", E::Weather),
        ("TwoTurn", E::TwoTurn),
        ("Consecutive", E::Consecutive),
        ("Splash", E::Splash),
    ] {
        add(lua, &effects, name, LuaSymbol::Effect(value))?;
    }
    globals.set("Effect", effects)?;

    let accuracies = lua.create_table()?;
    add(
        lua,
        &accuracies,
        "Always",
        LuaSymbol::Accuracy(AccuracyKind::Always),
    )?;
    add(
        lua,
        &accuracies,
        "Ohko",
        LuaSymbol::Accuracy(AccuracyKind::Ohko),
    )?;
    globals.set("Accuracy", accuracies)?;
    Ok(())
}

pub(crate) fn from_userdata(value: AnyUserData, field: &str) -> Result<LuaSymbol, BattleError> {
    value
        .borrow::<LuaSymbol>()
        .map(|value| *value)
        .map_err(|_| BattleError::InvalidSetup(format!("{field} requires a battle enum value")))
}

pub(crate) fn required(table: &Table, field: &str) -> Result<LuaSymbol, BattleError> {
    match table.get::<Value>(field)? {
        Value::UserData(value) => from_userdata(value, field),
        _ => Err(BattleError::InvalidSetup(format!(
            "{field} requires a battle enum value"
        ))),
    }
}

pub(crate) fn optional(table: &Table, field: &str) -> Result<Option<LuaSymbol>, BattleError> {
    match table.get::<Value>(field)? {
        Value::Nil => Ok(None),
        Value::UserData(value) => Ok(Some(from_userdata(value, field)?)),
        _ => Err(BattleError::InvalidSetup(format!(
            "{field} requires a battle enum value"
        ))),
    }
}

pub(crate) fn applied(status: Status) -> AppliedStatus {
    match status {
        Status::Poisoned => AppliedStatus::Poisoned,
        Status::BadlyPoisoned => AppliedStatus::BadlyPoisoned,
        Status::Burned => AppliedStatus::Burned,
        Status::Paralyzed => AppliedStatus::Paralyzed,
        Status::Asleep => AppliedStatus::Asleep,
        Status::Frozen => AppliedStatus::Frozen,
    }
}
