use crate::model::{AppliedStatus, BattleError, PokemonType, Stat, WeatherKind};
use mlua::{Function, Lua, Table, Value};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Physical,
    Special,
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Selected,
    User,
    Field,
    AllOthers,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Accuracy {
    Chance(f32),
    Always,
    OneHitKnockout,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Damage {
        power: u16,
        recoil: Option<f32>,
        drain: Option<f32>,
        high_crit: bool,
        always_crit: bool,
    },
    MultiHit {
        power: u16,
        min: u8,
        max: u8,
    },
    FixedDamage(u16),
    LevelDamage,
    OneHitKnockout,
    Stats {
        stages: BTreeMap<Stat, i8>,
        chance: f32,
        self_target: bool,
    },
    Status {
        status: AppliedStatus,
        chance: f32,
        replace: bool,
    },
    Confuse,
    Flinch(f32),
    Bind,
    Protect,
    Heal {
        fraction: f32,
        hide_message: bool,
    },
    Weather {
        kind: WeatherKind,
        turns: u8,
    },
    TwoTurn {
        power: u16,
        charge_message: String,
        semi_invulnerable: bool,
        skip_in_sun: bool,
    },
    Consecutive {
        power: u16,
        min: u8,
        max: u8,
        confuse_after: bool,
        double_power: bool,
    },
    Splash,
}

#[derive(Clone, Debug)]
pub struct MoveSpec {
    pub id: String,
    pub name: String,
    pub move_type: PokemonType,
    pub category: Category,
    pub pp: u8,
    pub target: Target,
    pub accuracy: Accuracy,
    pub priority: i8,
    pub fail_on_full_hp: bool,
    pub effects: Vec<Effect>,
    pub(crate) script: Option<Function>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ScriptAction {
    Damage {
        power: u16,
        accuracy: Option<f32>,
        drain: Option<f32>,
        min_target_hp: u16,
        stop_on_miss: bool,
    },
    FaintUser,
    RechargeIfHit,
    Fail,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScriptContext {
    pub user_hp: u16,
    pub target_hp: u16,
    pub target_status: Option<crate::model::Status>,
}

#[derive(Clone, Debug)]
pub struct MoveCatalog {
    moves: HashMap<String, MoveSpec>,
    lua: Lua,
}

impl MoveCatalog {
    pub fn builtin() -> Result<Self, BattleError> {
        Self::from_lua(include_str!("builtin_moves.lua"))
    }
    pub fn from_lua(source: &str) -> Result<Self, BattleError> {
        let lua = Lua::new();
        let table: Table = lua.load(source).eval()?;
        let mut moves = HashMap::new();
        for value in table.sequence_values::<Table>() {
            let entry = value?;
            let spec = parse_move(&entry)?;
            if moves.insert(spec.id.clone(), spec).is_some() {
                return Err(BattleError::InvalidSetup("duplicate move id".into()));
            }
        }
        if moves.is_empty() {
            return Err(BattleError::InvalidSetup("empty move catalog".into()));
        }
        Ok(Self { moves, lua })
    }
    pub fn get(&self, id: &str) -> Option<&MoveSpec> {
        self.moves.get(id)
    }
    pub fn len(&self) -> usize {
        self.moves.len()
    }
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.moves.keys().map(String::as_str)
    }
    pub(crate) fn script_actions(
        &self,
        spec: &MoveSpec,
        context: ScriptContext,
    ) -> Result<Vec<ScriptAction>, BattleError> {
        let function = spec.script.as_ref().expect("scripted move");
        let input = self.lua.create_table()?;
        let user = self.lua.create_table()?;
        user.set("hp", context.user_hp)?;
        input.set("user", user)?;
        let target = self.lua.create_table()?;
        target.set("hp", context.target_hp)?;
        target.set(
            "status",
            context.target_status.map(|status| status.lua_name()),
        )?;
        input.set("target", target)?;
        let output: Table = function.call(input)?;
        output
            .sequence_values::<Table>()
            .map(|value| parse_script_action(&value?))
            .collect()
    }
}

fn parse_script_action(t: &Table) -> Result<ScriptAction, BattleError> {
    let kind: String = t.get("kind")?;
    Ok(match kind.as_str() {
        "damage" => ScriptAction::Damage {
            power: t.get("power")?,
            accuracy: t.get("accuracy")?,
            drain: t.get("drain")?,
            min_target_hp: t.get::<Option<u16>>("min_target_hp")?.unwrap_or(0),
            stop_on_miss: t.get::<Option<bool>>("stop_on_miss")?.unwrap_or(false),
        },
        "faint_user" => ScriptAction::FaintUser,
        "recharge_if_hit" => ScriptAction::RechargeIfHit,
        "fail" => ScriptAction::Fail,
        other => {
            return Err(BattleError::InvalidSetup(format!(
                "unknown script action {other}"
            )));
        }
    })
}

fn parse_move(t: &Table) -> Result<MoveSpec, BattleError> {
    let id: String = t.get("id")?;
    let name: String = t.get("name")?;
    let move_type = parse_name::<PokemonType>(t.get("type")?)?;
    let category = match t.get::<String>("category")?.as_str() {
        "physical" => Category::Physical,
        "special" => Category::Special,
        "status" => Category::Status,
        other => {
            return Err(BattleError::InvalidSetup(format!(
                "unknown category {other}"
            )));
        }
    };
    let target = match t
        .get::<Option<String>>("target")?
        .as_deref()
        .unwrap_or("selected")
    {
        "selected" => Target::Selected,
        "user" => Target::User,
        "field" => Target::Field,
        "all_others" => Target::AllOthers,
        other => return Err(BattleError::InvalidSetup(format!("unknown target {other}"))),
    };
    let accuracy = match t.get::<Value>("accuracy")? {
        Value::Nil => Accuracy::Chance(1.0),
        Value::String(s) if s.to_str()? == "always" => Accuracy::Always,
        Value::String(s) if s.to_str()? == "ohko" => Accuracy::OneHitKnockout,
        Value::Number(n) => Accuracy::Chance(n as f32),
        Value::Integer(n) => Accuracy::Chance(n as f32),
        _ => {
            return Err(BattleError::InvalidSetup(format!(
                "invalid accuracy for {id}"
            )));
        }
    };
    let script: Option<Function> = t.get("script")?;
    let effects = if let Some(table) = t.get::<Option<Table>>("effects")? {
        table
            .sequence_values::<Table>()
            .map(|e| parse_effect(&e?))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };
    if effects.is_empty() && script.is_none() {
        return Err(BattleError::InvalidSetup(format!("no behavior for {id}")));
    }
    if !effects.is_empty() && script.is_some() {
        return Err(BattleError::InvalidSetup(format!(
            "both effects and script for {id}"
        )));
    }
    Ok(MoveSpec {
        id,
        name,
        move_type,
        category,
        pp: t.get("pp")?,
        target,
        accuracy,
        priority: t.get::<Option<i8>>("priority")?.unwrap_or(0),
        fail_on_full_hp: t.get::<Option<bool>>("fail_on_full_hp")?.unwrap_or(false),
        effects,
        script,
    })
}

fn parse_effect(t: &Table) -> Result<Effect, BattleError> {
    let kind: String = t.get("kind")?;
    let chance =
        || -> Result<f32, mlua::Error> { Ok(t.get::<Option<f32>>("chance")?.unwrap_or(1.0)) };
    let opt_bool =
        |key| -> Result<bool, mlua::Error> { Ok(t.get::<Option<bool>>(key)?.unwrap_or(false)) };
    Ok(match kind.as_str() {
        "damage" => Effect::Damage {
            power: t.get("power")?,
            recoil: t.get("recoil")?,
            drain: t.get("drain")?,
            high_crit: opt_bool("high_crit")?,
            always_crit: opt_bool("always_crit")?,
        },
        "multi_hit" => Effect::MultiHit {
            power: t.get("power")?,
            min: t.get("min_hits")?,
            max: t.get("max_hits")?,
        },
        "fixed_damage" => Effect::FixedDamage(t.get("amount")?),
        "level_damage" => Effect::LevelDamage,
        "ohko" => Effect::OneHitKnockout,
        "stats" => {
            let mut stages = BTreeMap::new();
            for pair in t.get::<Table>("stages")?.pairs::<String, i8>() {
                let (stat, delta) = pair?;
                stages.insert(parse_name::<Stat>(stat)?, delta);
            }
            Effect::Stats {
                stages,
                chance: chance()?,
                self_target: opt_bool("self")?,
            }
        }
        "status" => Effect::Status {
            status: parse_name::<AppliedStatus>(t.get("status")?)?,
            chance: chance()?,
            replace: opt_bool("replace")?,
        },
        "confuse" => Effect::Confuse,
        "flinch" => Effect::Flinch(chance()?),
        "bind" => Effect::Bind,
        "protect" => Effect::Protect,
        "heal" => Effect::Heal {
            fraction: t.get("fraction")?,
            hide_message: opt_bool("hide_message")?,
        },
        "weather" => Effect::Weather {
            kind: parse_name::<WeatherKind>(t.get("weather")?)?,
            turns: t.get("turns")?,
        },
        "two_turn" => Effect::TwoTurn {
            power: t.get("power")?,
            charge_message: t.get("charge_message")?,
            semi_invulnerable: opt_bool("semi_invulnerable")?,
            skip_in_sun: opt_bool("skip_in_sun")?,
        },
        "consecutive" => Effect::Consecutive {
            power: t.get("power")?,
            min: t.get("min_turns")?,
            max: t.get("max_turns")?,
            confuse_after: opt_bool("confuse_after")?,
            double_power: opt_bool("double_power")?,
        },
        "splash" => Effect::Splash,
        other => {
            return Err(BattleError::InvalidSetup(format!(
                "unknown effect kind {other}"
            )));
        }
    })
}

fn parse_name<T: FromStr<Err = String>>(name: String) -> Result<T, BattleError> {
    name.parse().map_err(BattleError::InvalidSetup)
}
