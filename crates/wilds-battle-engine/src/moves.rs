use crate::lua_symbols::{self, AccuracyKind, EffectKind, LuaSymbol};
use crate::model::{
    AppliedStatus, BattleError, ContinuationTarget, PokemonType, Stat, WeatherKind,
};
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
    RandomOpponent,
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
    pub(crate) on_interrupt: Option<Function>,
    pub(crate) manual_announce: bool,
    pub(crate) auto_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ScriptOperation {
    Damage {
        power: u16,
        accuracy: Option<f32>,
        drain: Option<f32>,
        min_target_hp: u16,
        typeless: bool,
    },
    FaintUser,
    Recharge,
    Fail,
    ForceMove {
        total_turns: u8,
        target_policy: ContinuationTarget,
    },
    BreakSequence,
    RandomInt {
        min: u8,
        max: u8,
    },
    Message(String),
    Announce,
    ConfuseSelf,
    RecoilMaxHp(f32),
}
impl UserData for ScriptOperation {}

#[derive(Clone, Debug)]
pub(crate) struct ScriptContext {
    pub user_hp: u16,
    pub user_name: String,
    pub target_hp: u16,
    pub target_status: Option<crate::model::Status>,
    pub weather: Option<WeatherKind>,
    pub turn: u8,
    pub total_turns: Option<u8>,
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
        let system: Table = lua.load(include_str!("system_moves.lua")).eval()?;
        let spec = parse_move(&system)?;
        if moves.insert(spec.id.clone(), spec).is_some() {
            return Err(BattleError::InvalidSetup("reserved system move id".into()));
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
        interrupted: bool,
    ) -> Result<(Thread, Table), BattleError> {
        let function = if interrupted {
            spec.on_interrupt.as_ref().expect("interrupt hook")
        } else {
            spec.script.as_ref().expect("scripted move")
        };
        let api = self.lua.create_table()?;
        api.set(
            "damage",
            self.lua
                .create_function(|lua, (power, options): (u16, Option<Table>)| {
                    if let Some(options) = &options {
                        for entry in options.clone().pairs::<String, Value>() {
                            let (key, _) = entry?;
                            if !matches!(
                                key.as_str(),
                                "accuracy" | "drain" | "min_target_hp" | "typeless"
                            ) {
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
                    let typeless = options
                        .as_ref()
                        .map(|table| table.get::<Option<bool>>("typeless"))
                        .transpose()?
                        .flatten()
                        .unwrap_or(false);
                    lua.create_userdata(ScriptOperation::Damage {
                        power,
                        accuracy,
                        drain,
                        min_target_hp,
                        typeless,
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
        api.set(
            "force_move",
            self.lua
                .create_function(|lua, (total_turns, target): (u8, mlua::AnyUserData)| {
                    if !(2..=8).contains(&total_turns) {
                        return Err(mlua::Error::external("forced move duration must be 2..=8"));
                    }
                    let target_policy = match lua_symbols::from_userdata(target, "target policy") {
                        Ok(LuaSymbol::ContinuationTarget(value)) => value,
                        _ => {
                            return Err(mlua::Error::external(
                                "target policy requires TargetPolicy value",
                            ));
                        }
                    };
                    lua.create_userdata(ScriptOperation::ForceMove {
                        total_turns,
                        target_policy,
                    })
                })?,
        )?;
        api.set(
            "break_sequence",
            self.lua
                .create_function(|lua, ()| lua.create_userdata(ScriptOperation::BreakSequence))?,
        )?;
        api.set(
            "random_int",
            self.lua.create_function(|lua, (min, max): (u8, u8)| {
                if min > max {
                    return Err(mlua::Error::external("random_int minimum exceeds maximum"));
                }
                lua.create_userdata(ScriptOperation::RandomInt { min, max })
            })?,
        )?;
        api.set(
            "message",
            self.lua.create_function(|lua, message: String| {
                lua.create_userdata(ScriptOperation::Message(message))
            })?,
        )?;
        api.set(
            "announce",
            self.lua
                .create_function(|lua, ()| lua.create_userdata(ScriptOperation::Announce))?,
        )?;
        api.set(
            "confuse_self",
            self.lua
                .create_function(|lua, ()| lua.create_userdata(ScriptOperation::ConfuseSelf))?,
        )?;
        api.set(
            "recoil_max_hp",
            self.lua.create_function(|lua, fraction: f32| {
                if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                    return Err(mlua::Error::external(
                        "recoil fraction must be between 0 and 1",
                    ));
                }
                lua.create_userdata(ScriptOperation::RecoilMaxHp(fraction))
            })?,
        )?;
        let user = self.lua.create_table()?;
        user.set("hp", context.user_hp)?;
        user.set("name", context.user_name)?;
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
        let target_policy: Table = self.lua.globals().get("TargetPolicy")?;
        let weather = context
            .weather
            .map(|kind| self.lua.create_userdata(LuaSymbol::Weather(kind)))
            .transpose()?;
        let input: Table = self.script_factory.call((
            api,
            user,
            target,
            statuses,
            weather,
            context.turn,
            context.total_turns,
            target_policy,
        ))?;
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
    let on_interrupt: Option<Function> = t.get("on_interrupt")?;
    if on_interrupt.is_some() && script.is_none() {
        return Err(BattleError::InvalidSetup(format!(
            "interrupt hook without script for {id}"
        )));
    }
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
        on_interrupt,
        manual_announce: t.get::<Option<bool>>("manual_announce")?.unwrap_or(false),
        auto_only: t.get::<Option<bool>>("auto_only")?.unwrap_or(false),
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
