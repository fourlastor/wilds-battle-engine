use crate::lua_symbols::{self, AccuracyKind, EffectKind, LuaSymbol};
use crate::model::{AppliedStatus, BattleError, PokemonType, Stat, WeatherKind};
use mlua::{Function, HookTriggers, Lua, Table, Thread, UserData, Value, VmState};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

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
pub(crate) enum ScriptOperation {
    Damage {
        power: u16,
        accuracy: Option<f32>,
        drain: Option<f32>,
        min_target_hp: u16,
    },
    FaintUser,
    Recharge,
    Fail,
}
impl UserData for ScriptOperation {}

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
    script_factory: Function,
}

impl MoveCatalog {
    pub fn builtin() -> Result<Self, BattleError> {
        Self::from_lua(include_str!("builtin_moves.lua"))
    }
    pub fn from_lua(source: &str) -> Result<Self, BattleError> {
        let lua = Lua::new();
        lua_symbols::install(&lua)?;
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
        let script_factory = lua.load(include_str!("script_api.lua")).eval()?;
        Ok(Self {
            moves,
            lua,
            script_factory,
        })
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
    pub(crate) fn start_script(
        &self,
        spec: &MoveSpec,
        context: ScriptContext,
    ) -> Result<(Thread, Table), BattleError> {
        let function = spec.script.as_ref().expect("scripted move");
        let api = self.lua.create_table()?;
        api.set(
            "damage",
            self.lua
                .create_function(|lua, (power, options): (u16, Option<Table>)| {
                    if let Some(options) = &options {
                        for entry in options.clone().pairs::<String, Value>() {
                            let (key, _) = entry?;
                            if !matches!(key.as_str(), "accuracy" | "drain" | "min_target_hp") {
                                return Err(mlua::Error::external(format!(
                                    "unknown damage option {key}"
                                )));
                            }
                        }
                    }
                    let accuracy = options
                        .as_ref()
                        .map(|table| table.get::<Option<f32>>("accuracy"))
                        .transpose()?
                        .flatten();
                    let drain = options
                        .as_ref()
                        .map(|table| table.get::<Option<f32>>("drain"))
                        .transpose()?
                        .flatten();
                    for fraction in [accuracy, drain].into_iter().flatten() {
                        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                            return Err(mlua::Error::external(
                                "damage fractions must be between 0 and 1",
                            ));
                        }
                    }
                    let min_target_hp = options
                        .as_ref()
                        .map(|table| table.get::<Option<u16>>("min_target_hp"))
                        .transpose()?
                        .flatten()
                        .unwrap_or(0);
                    lua.create_userdata(ScriptOperation::Damage {
                        power,
                        accuracy,
                        drain,
                        min_target_hp,
                    })
                })?,
        )?;
        api.set(
            "faint_user",
            self.lua
                .create_function(|lua, ()| lua.create_userdata(ScriptOperation::FaintUser))?,
        )?;
        api.set(
            "recharge",
            self.lua
                .create_function(|lua, ()| lua.create_userdata(ScriptOperation::Recharge))?,
        )?;
        api.set(
            "fail",
            self.lua
                .create_function(|lua, ()| lua.create_userdata(ScriptOperation::Fail))?,
        )?;
        let user = self.lua.create_table()?;
        user.set("hp", context.user_hp)?;
        let target = self.lua.create_table()?;
        target.set("hp", context.target_hp)?;
        if let Some(status) = context.target_status {
            target.set(
                "status",
                self.lua
                    .create_userdata(LuaSymbol::Status(lua_symbols::applied(status)))?,
            )?;
        }
        let statuses: Table = self.lua.globals().get("Status")?;
        let input: Table = self.script_factory.call((api, user, target, statuses))?;
        let thread = self.lua.create_thread(function.clone())?;
        let instructions = Arc::new(AtomicU32::new(0));
        thread.set_hook(
            HookTriggers::new().every_nth_instruction(1000),
            move |_, _| {
                if instructions.fetch_add(1000, Ordering::Relaxed) >= 100_000 {
                    Err(mlua::Error::external(
                        "move script exceeded 100000 instructions",
                    ))
                } else {
                    Ok(VmState::Continue)
                }
            },
        )?;
        Ok((thread, input))
    }
    pub(crate) fn damage_result(&self, hit: bool, damage: u16) -> Result<Table, BattleError> {
        let result = self.lua.create_table()?;
        result.set("hit", hit)?;
        result.set("damage", damage)?;
        Ok(result)
    }
}

fn parse_move(t: &Table) -> Result<MoveSpec, BattleError> {
    let id: String = t.get("id")?;
    let name: String = t.get("name")?;
    let move_type = match lua_symbols::required(t, "type")? {
        LuaSymbol::Type(value) => value,
        _ => return Err(BattleError::InvalidSetup("type requires Type value".into())),
    };
    let category = match lua_symbols::required(t, "category")? {
        LuaSymbol::Category(value) => value,
        _ => {
            return Err(BattleError::InvalidSetup(
                "category requires Category value".into(),
            ));
        }
    };
    let target = match lua_symbols::optional(t, "target")? {
        None => Target::Selected,
        Some(LuaSymbol::Target(value)) => value,
        _ => {
            return Err(BattleError::InvalidSetup(
                "target requires Target value".into(),
            ));
        }
    };
    let accuracy = match t.get::<Value>("accuracy")? {
        Value::Nil => Accuracy::Chance(1.0),
        Value::UserData(value) => match lua_symbols::from_userdata(value, "accuracy")? {
            LuaSymbol::Accuracy(AccuracyKind::Always) => Accuracy::Always,
            LuaSymbol::Accuracy(AccuracyKind::Ohko) => Accuracy::OneHitKnockout,
            _ => {
                return Err(BattleError::InvalidSetup(
                    "accuracy requires Accuracy value".into(),
                ));
            }
        },
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
    let kind = match lua_symbols::required(t, "kind")? {
        LuaSymbol::Effect(value) => value,
        _ => {
            return Err(BattleError::InvalidSetup(
                "kind requires Effect value".into(),
            ));
        }
    };
    let chance =
        || -> Result<f32, mlua::Error> { Ok(t.get::<Option<f32>>("chance")?.unwrap_or(1.0)) };
    let opt_bool =
        |key| -> Result<bool, mlua::Error> { Ok(t.get::<Option<bool>>(key)?.unwrap_or(false)) };
    Ok(match kind {
        EffectKind::Damage => Effect::Damage {
            power: t.get("power")?,
            recoil: t.get("recoil")?,
            drain: t.get("drain")?,
            high_crit: opt_bool("high_crit")?,
            always_crit: opt_bool("always_crit")?,
        },
        EffectKind::MultiHit => Effect::MultiHit {
            power: t.get("power")?,
            min: t.get("min_hits")?,
            max: t.get("max_hits")?,
        },
        EffectKind::FixedDamage => Effect::FixedDamage(t.get("amount")?),
        EffectKind::LevelDamage => Effect::LevelDamage,
        EffectKind::Ohko => Effect::OneHitKnockout,
        EffectKind::Stats => {
            let mut stages = BTreeMap::new();
            for pair in t.get::<Table>("stages")?.pairs::<mlua::AnyUserData, i8>() {
                let (stat, delta) = pair?;
                let stat = match lua_symbols::from_userdata(stat, "stage")? {
                    LuaSymbol::Stat(value) => value,
                    _ => {
                        return Err(BattleError::InvalidSetup(
                            "stage requires Stat value".into(),
                        ));
                    }
                };
                stages.insert(stat, delta);
            }
            Effect::Stats {
                stages,
                chance: chance()?,
                self_target: opt_bool("self")?,
            }
        }
        EffectKind::Status => Effect::Status {
            status: match lua_symbols::required(t, "status")? {
                LuaSymbol::Status(value) => value,
                _ => {
                    return Err(BattleError::InvalidSetup(
                        "status requires Status value".into(),
                    ));
                }
            },
            chance: chance()?,
            replace: opt_bool("replace")?,
        },
        EffectKind::Confuse => Effect::Confuse,
        EffectKind::Flinch => Effect::Flinch(chance()?),
        EffectKind::Bind => Effect::Bind,
        EffectKind::Protect => Effect::Protect,
        EffectKind::Heal => Effect::Heal {
            fraction: t.get("fraction")?,
            hide_message: opt_bool("hide_message")?,
        },
        EffectKind::Weather => Effect::Weather {
            kind: match lua_symbols::required(t, "weather")? {
                LuaSymbol::Weather(value) => value,
                _ => {
                    return Err(BattleError::InvalidSetup(
                        "weather requires Weather value".into(),
                    ));
                }
            },
            turns: t.get("turns")?,
        },
        EffectKind::TwoTurn => Effect::TwoTurn {
            power: t.get("power")?,
            charge_message: t.get("charge_message")?,
            semi_invulnerable: opt_bool("semi_invulnerable")?,
            skip_in_sun: opt_bool("skip_in_sun")?,
        },
        EffectKind::Consecutive => Effect::Consecutive {
            power: t.get("power")?,
            min: t.get("min_turns")?,
            max: t.get("max_turns")?,
            confuse_after: opt_bool("confuse_after")?,
            double_power: opt_bool("double_power")?,
        },
        EffectKind::Splash => Effect::Splash,
    })
}
