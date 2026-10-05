use crate::lua_symbols::{self, AccuracyKind, EffectKind, LuaSymbol, RuleKind};
use crate::model::{
    AppliedStatus, BattleError, CombatStat, Condition, ContinuationTarget, HiddenKind,
    ParticipantId, PokemonType, Rule, Scope, Stat, WeatherKind,
};
use mlua::{Function, HookTriggers, Lua, Table, Thread, UserData, Value, Variadic, VmState};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
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
    AllOpponents,
    Ally,
    UserOrAlly,
    /// One other Pokémon of either side, chosen by the player.
    AnyOther,
    AllAllies,
    UserAndAllies,
    All,
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
        /// Where the user hides while `semi_invulnerable`.
        hidden: HiddenKind,
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
    /// Free-form tags such as "contact" or "sound", for scripts that care.
    pub flags: Vec<String>,
    /// Hidden places this move still reaches.
    pub hits_hidden: Vec<HiddenKind>,
    pub(crate) script: Option<Function>,
    pub(crate) on_interrupt: Option<Function>,
    pub(crate) on_hit: Option<Function>,
    pub(crate) on_turn_start: Option<Function>,
    pub(crate) manual_announce: bool,
    pub(crate) auto_only: bool,
    pub(crate) usable_while_asleep: bool,
    pub(crate) usable_while_frozen: bool,
}
impl MoveSpec {
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|own| own == flag)
    }
    /// Whether a sleeping Pokémon can use this move.
    pub fn usable_while_asleep(&self) -> bool {
        self.usable_while_asleep
    }
    /// Whether a frozen Pokémon can use this move, thawing itself.
    pub fn usable_while_frozen(&self) -> bool {
        self.usable_while_frozen
    }
}

/// How one hit is resolved: what it ignores and what it can still reach.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HitOptions {
    pub accuracy: Option<f32>,
    pub never_miss: bool,
    pub ignore_protect: bool,
    pub ignore_evasion: bool,
    pub hits_hidden: Vec<HiddenKind>,
    pub hits_all_hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct DamageOptions {
    pub hit: HitOptions,
    pub drain: Option<f32>,
    pub recoil: Option<f32>,
    pub min_target_hp: u16,
    pub typeless: bool,
    pub high_crit: bool,
    pub always_crit: bool,
    /// One recipient instead of everyone the move is aimed at.
    pub target: Option<ParticipantId>,
    pub move_type: Option<PokemonType>,
    /// A second type that also counts for type matchups (Flying Press).
    pub also_type: Option<PokemonType>,
    /// Matchups that replace the type chart's against the listed defending types (Freeze-Dry).
    pub effective: Vec<(PokemonType, f32)>,
    pub category: Option<Category>,
    /// Whose attacking stat is used (Foul Play).
    pub attack_from: Option<ParticipantId>,
    pub attack_stat: Option<CombatStat>,
    pub defense_stat: Option<CombatStat>,
    /// The recipient's stat changes do not count.
    pub ignore_stages: bool,
}

/// An amount of HP given either in points or as a share of max HP.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HpAmount {
    Points(u16),
    MaxFraction(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ScriptOperation {
    Damage {
        power: u16,
        options: DamageOptions,
    },
    MultiHit {
        power: u16,
        min: u8,
        max: u8,
        options: DamageOptions,
    },
    DirectDamage {
        who: ParticipantId,
        amount: u16,
        typeless: bool,
        min_target_hp: u16,
    },
    TryHit {
        who: Option<ParticipantId>,
        options: HitOptions,
    },
    Chance(f32),
    /// A chance of an extra effect, which effects on the user's side can multiply.
    EffectChance(f32),
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
    ChangeSelfStat {
        stat: Stat,
        stages: i8,
    },
    BoostNextMove {
        move_type: PokemonType,
        multiplier: f32,
    },
    WatchHitsUntilNextAction,
    HealSelf(f32),
    FlinchTarget(f32),
    Hurt {
        who: ParticipantId,
        amount: HpAmount,
    },
    Heal {
        who: ParticipantId,
        amount: HpAmount,
    },
    ApplyStatus {
        who: ParticipantId,
        status: AppliedStatus,
        chance: f32,
        replace: bool,
        announce_failure: bool,
    },
    CureStatus {
        who: ParticipantId,
        status: Option<AppliedStatus>,
    },
    Confuse {
        who: ParticipantId,
        chance: f32,
    },
    Flinch {
        who: ParticipantId,
        chance: f32,
    },
    ChangeStat {
        who: ParticipantId,
        stat: Stat,
        stages: i8,
        chance: f32,
    },
    ResetStats {
        who: ParticipantId,
    },
    Bind {
        who: ParticipantId,
        min_turns: u8,
        max_turns: u8,
    },
    Free {
        who: ParticipantId,
    },
    Protect {
        who: ParticipantId,
    },
    BreakProtect {
        who: ParticipantId,
    },
    Hide(HiddenKind),
    Unhide {
        who: ParticipantId,
    },
    SetWeather {
        kind: WeatherKind,
        turns: u8,
    },
    ClearWeather,
    Mark {
        scope: Scope,
        name: String,
        value: i32,
        turns: Option<u8>,
    },
    Unmark {
        scope: Scope,
        name: String,
    },
    StartEffect {
        scope: Scope,
        condition: Condition,
        restart: bool,
    },
    EndEffect {
        scope: Scope,
        name: String,
    },
    EndGroup {
        scope: Scope,
        group: String,
    },
    SetTypes {
        who: ParticipantId,
        types: Vec<PokemonType>,
    },
    SetWeight {
        who: ParticipantId,
        weight: f32,
    },
    TakeItem {
        who: ParticipantId,
    },
    GiveItem {
        who: ParticipantId,
        item: String,
    },
    SuppressAbility {
        who: ParticipantId,
    },
    AddPayout(u32),
    Hurry,
}
impl UserData for ScriptOperation {}

/// Marks a Lua table as standing for a Pokémon, a side or the field. Scripts cannot make one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScopeRef(pub Scope);
impl UserData for ScopeRef {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScriptHook {
    Main,
    Interrupt,
    Hit,
    TurnStart,
}

fn bad(message: impl Into<String>) -> mlua::Error {
    mlua::Error::external(message.into())
}

fn scope_of(value: &Value, what: &str) -> mlua::Result<Scope> {
    let Value::Table(table) = value else {
        return Err(bad(format!("{what} needs a Pokémon, a side or the field")));
    };
    match table.raw_get::<Value>("ref")? {
        Value::UserData(handle) => handle
            .borrow::<ScopeRef>()
            .map(|handle| handle.0)
            .map_err(|_| bad(format!("{what} needs a Pokémon, a side or the field"))),
        _ => Err(bad(format!("{what} needs a Pokémon, a side or the field"))),
    }
}

fn pokemon_of(value: &Value, what: &str) -> mlua::Result<ParticipantId> {
    match scope_of(value, what) {
        Ok(Scope::Pokemon(id)) => Ok(id),
        _ => Err(bad(format!("{what} needs a Pokémon"))),
    }
}

fn symbol_of(value: &Value, what: &str) -> mlua::Result<LuaSymbol> {
    match value {
        Value::UserData(data) => data
            .borrow::<LuaSymbol>()
            .map(|symbol| *symbol)
            .map_err(|_| bad(format!("{what} requires a battle enum value"))),
        _ => Err(bad(format!("{what} requires a battle enum value"))),
    }
}

fn type_of(value: &Value, what: &str) -> mlua::Result<PokemonType> {
    match symbol_of(value, what)? {
        LuaSymbol::Type(value) => Ok(value),
        _ => Err(bad(format!("{what} requires a Type value"))),
    }
}

fn status_of(value: &Value, what: &str) -> mlua::Result<AppliedStatus> {
    match symbol_of(value, what)? {
        LuaSymbol::Status(value) => Ok(value),
        _ => Err(bad(format!("{what} requires a Status value"))),
    }
}

fn stat_of(value: &Value, what: &str) -> mlua::Result<Stat> {
    match symbol_of(value, what)? {
        LuaSymbol::Stat(value) => Ok(value),
        _ => Err(bad(format!("{what} requires a Stat value"))),
    }
}

fn combat_stat_of(value: &Value, what: &str) -> mlua::Result<CombatStat> {
    Ok(match stat_of(value, what)? {
        Stat::Attack => CombatStat::Attack,
        Stat::Defense => CombatStat::Defense,
        Stat::SpAttack => CombatStat::SpAttack,
        Stat::SpDefense => CombatStat::SpDefense,
        Stat::Speed => CombatStat::Speed,
        Stat::Accuracy | Stat::Evasion => {
            return Err(bad(format!("{what} cannot be Accuracy or Evasion")));
        }
    })
}

fn hidden_of(value: &Value, what: &str) -> mlua::Result<HiddenKind> {
    match symbol_of(value, what)? {
        LuaSymbol::Hidden(value) => Ok(value),
        _ => Err(bad(format!("{what} requires a Hidden value"))),
    }
}

fn number_of(value: &Value, what: &str) -> mlua::Result<f64> {
    match value {
        Value::Integer(number) => Ok(*number as f64),
        Value::Number(number) if number.is_finite() => Ok(*number),
        _ => Err(bad(format!("{what} requires a number"))),
    }
}

fn whole(value: &Value, what: &str, min: f64, max: f64) -> mlua::Result<f64> {
    let number = number_of(value, what)?.round();
    if number < min || number > max {
        return Err(bad(format!("{what} must be between {min} and {max}")));
    }
    Ok(number)
}

/// An amount of power or HP. A fraction is dropped, as the games do.
fn floored(value: &Value, what: &str) -> mlua::Result<u16> {
    // The small margin keeps a product such as 150 * 2 / 3 from landing just under a whole number.
    let number = (number_of(value, what)? + 1e-6).floor();
    if !(0.0..=65535.0).contains(&number) {
        return Err(bad(format!("{what} must be between 0 and 65535")));
    }
    Ok(number as u16)
}

fn fraction_of(value: &Value, what: &str) -> mlua::Result<f32> {
    let number = number_of(value, what)?;
    if !(0.0..=1.0).contains(&number) {
        return Err(bad(format!("{what} must be between 0 and 1")));
    }
    Ok(number as f32)
}

fn text_of(value: &Value, what: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        _ => Err(bad(format!("{what} requires text"))),
    }
}

fn name_of(value: &Value, what: &str) -> mlua::Result<String> {
    let name = text_of(value, what)?;
    if name.is_empty() || name.len() > 40 {
        return Err(bad(format!("{what} must be 1 to 40 characters long")));
    }
    Ok(name)
}

/// An optional table of named settings; unknown names are an error so that typos are caught.
struct Options {
    table: Option<Table>,
    what: &'static str,
}
impl Options {
    fn new(value: Option<&Value>, what: &'static str, allowed: &[&str]) -> mlua::Result<Self> {
        let table = match value {
            None | Some(Value::Nil) => None,
            Some(Value::Table(table)) => Some(table.clone()),
            Some(_) => return Err(bad(format!("{what} options must be a table"))),
        };
        if let Some(table) = &table {
            for entry in table.clone().pairs::<Value, Value>() {
                let (key, _) = entry?;
                let known = match &key {
                    Value::String(key) => allowed.contains(&&*key.to_str()?),
                    _ => false,
                };
                if !known {
                    let key = match &key {
                        Value::String(key) => key.to_str()?.to_string(),
                        other => other.type_name().to_string(),
                    };
                    return Err(bad(format!("unknown {what} option {key}")));
                }
            }
        }
        Ok(Self { table, what })
    }
    fn get(&self, key: &str) -> mlua::Result<Option<Value>> {
        match &self.table {
            None => Ok(None),
            Some(table) => match table.get::<Value>(key)? {
                Value::Nil => Ok(None),
                value => Ok(Some(value)),
            },
        }
    }
    fn label(&self, key: &str) -> String {
        format!("{} option {key}", self.what)
    }
    fn flag(&self, key: &str) -> mlua::Result<bool> {
        match self.get(key)? {
            None => Ok(false),
            Some(Value::Boolean(value)) => Ok(value),
            Some(_) => Err(bad(format!("{} must be true or false", self.label(key)))),
        }
    }
    fn fraction(&self, key: &str) -> mlua::Result<Option<f32>> {
        self.get(key)?
            .map(|value| fraction_of(&value, &self.label(key)))
            .transpose()
    }
    fn chance(&self) -> mlua::Result<f32> {
        Ok(self.fraction("chance")?.unwrap_or(1.0))
    }
    fn whole(&self, key: &str, min: f64, max: f64) -> mlua::Result<Option<f64>> {
        self.get(key)?
            .map(|value| whole(&value, &self.label(key), min, max))
            .transpose()
    }
    fn text(&self, key: &str) -> mlua::Result<Option<String>> {
        self.get(key)?
            .map(|value| text_of(&value, &self.label(key)))
            .transpose()
    }
    fn pokemon(&self, key: &str) -> mlua::Result<Option<ParticipantId>> {
        self.get(key)?
            .map(|value| pokemon_of(&value, &self.label(key)))
            .transpose()
    }
    fn pokemon_type(&self, key: &str) -> mlua::Result<Option<PokemonType>> {
        self.get(key)?
            .map(|value| type_of(&value, &self.label(key)))
            .transpose()
    }
    fn list<T>(
        &self,
        key: &str,
        read: impl Fn(&Value, &str) -> mlua::Result<T>,
    ) -> mlua::Result<Vec<T>> {
        match self.get(key)? {
            None => Ok(Vec::new()),
            Some(Value::Table(table)) => table
                .sequence_values::<Value>()
                .map(|value| read(&value?, &self.label(key)))
                .collect(),
            Some(_) => Err(bad(format!("{} must be a list", self.label(key)))),
        }
    }
}

const HIT_KEYS: [&str; 5] = [
    "accuracy",
    "never_miss",
    "ignore_protect",
    "ignore_evasion",
    "hits_hidden",
];

fn hit_options(options: &Options) -> mlua::Result<HitOptions> {
    let (hits_hidden, hits_all_hidden) = match options.get("hits_hidden")? {
        Some(Value::Boolean(all)) => (Vec::new(), all),
        _ => (options.list("hits_hidden", hidden_of)?, false),
    };
    Ok(HitOptions {
        accuracy: options.fraction("accuracy")?,
        never_miss: options.flag("never_miss")?,
        ignore_protect: options.flag("ignore_protect")?,
        ignore_evasion: options.flag("ignore_evasion")?,
        hits_hidden,
        hits_all_hidden,
    })
}

fn damage_options(value: Option<&Value>) -> mlua::Result<DamageOptions> {
    let options = Options::new(
        value,
        "damage",
        &[
            "accuracy",
            "never_miss",
            "ignore_protect",
            "ignore_evasion",
            "hits_hidden",
            "drain",
            "recoil",
            "min_target_hp",
            "typeless",
            "high_crit",
            "always_crit",
            "target",
            "type",
            "also_type",
            "effective",
            "category",
            "attack_from",
            "attack_stat",
            "defense_stat",
            "ignore_stages",
        ],
    )?;
    // The two fractions that existed first keep their original error text.
    for key in ["accuracy", "drain"] {
        if let Some(value) = options.get(key)? {
            let number = number_of(&value, &options.label(key))?;
            if !(0.0..=1.0).contains(&number) {
                return Err(bad("damage fractions must be between 0 and 1"));
            }
        }
    }
    let mut effective = Vec::new();
    if let Some(value) = options.get("effective")? {
        let Value::Table(table) = value else {
            return Err(bad(
                "damage option effective must be a table of Type = multiplier",
            ));
        };
        for entry in table.pairs::<Value, Value>() {
            let (defender, factor) = entry?;
            let factor = number_of(&factor, "damage option effective")?;
            if !(0.0..=4.0).contains(&factor) {
                return Err(bad(
                    "damage option effective multipliers must be between 0 and 4",
                ));
            }
            effective.push((
                type_of(&defender, "damage option effective")?,
                factor as f32,
            ));
        }
        effective.sort_by_key(|(defender, _)| *defender);
    }
    let category = match options.get("category")? {
        None => None,
        Some(value) => match symbol_of(&value, "damage option category")? {
            LuaSymbol::Category(Category::Status) => {
                return Err(bad("damage option category must be Physical or Special"));
            }
            LuaSymbol::Category(value) => Some(value),
            _ => return Err(bad("damage option category requires a Category value")),
        },
    };
    Ok(DamageOptions {
        hit: hit_options(&options)?,
        drain: options.fraction("drain")?,
        recoil: options.fraction("recoil")?,
        min_target_hp: options.whole("min_target_hp", 0.0, 65535.0)?.unwrap_or(0.0) as u16,
        typeless: options.flag("typeless")?,
        high_crit: options.flag("high_crit")?,
        always_crit: options.flag("always_crit")?,
        target: options.pokemon("target")?,
        move_type: options.pokemon_type("type")?,
        also_type: options.pokemon_type("also_type")?,
        effective,
        category,
        attack_from: options.pokemon("attack_from")?,
        attack_stat: options
            .get("attack_stat")?
            .map(|value| combat_stat_of(&value, "damage option attack_stat"))
            .transpose()?,
        defense_stat: options
            .get("defense_stat")?
            .map(|value| combat_stat_of(&value, "damage option defense_stat"))
            .transpose()?,
        ignore_stages: options.flag("ignore_stages")?,
    })
}

fn power_of(value: &Value) -> mlua::Result<u16> {
    floored(value, "power")
}

fn hp_amount(value: &Value, what: &'static str) -> mlua::Result<HpAmount> {
    if let Value::Table(_) = value {
        let options = Options::new(Some(value), what, &["fraction"])?;
        return match options.fraction("fraction")? {
            Some(fraction) => Ok(HpAmount::MaxFraction(fraction)),
            None => Err(bad(format!(
                "{what} needs a number of HP or {{fraction = ...}}"
            ))),
        };
    }
    Ok(HpAmount::Points(floored(value, what)?))
}

fn rule_of(value: &Value) -> mlua::Result<Rule> {
    let Value::Table(table) = value else {
        return Err(bad("each rule must be a table with a kind"));
    };
    let kind = match symbol_of(&table.get::<Value>("kind")?, "rule kind")? {
        LuaSymbol::Rule(kind) => kind,
        _ => return Err(bad("rule kind requires a Rule value")),
    };
    let allowed: &[&str] = match kind {
        RuleKind::DamageEachTurn => &["kind", "fraction", "except_types", "message"],
        RuleKind::HealEachTurn => &["kind", "fraction", "message"],
        RuleKind::DrainEachTurn => &["kind", "fraction", "to", "message"],
        RuleKind::DamageTaken => &["kind", "factor", "category", "type", "not_on_crit"],
        RuleKind::DamageDealt => &["kind", "factor", "category", "type"],
        RuleKind::StatMultiplier => &["kind", "stat", "factor"],
        RuleKind::BlockStatus => &["kind", "statuses", "confusion"],
        RuleKind::EffectChance => &["kind", "factor"],
        RuleKind::MoveType => &["kind", "from", "to"],
        RuleKind::AlwaysHit => &["kind", "against"],
        RuleKind::WithoutType => &["kind", "type"],
        RuleKind::BlockStatDrops | RuleKind::Grounded | RuleKind::Trapped | RuleKind::Endure => {
            &["kind"]
        }
    };
    let options = Options::new(Some(value), "rule", allowed)?;
    let required = |key: &str| {
        options
            .get(key)?
            .ok_or_else(|| bad(format!("rule {kind:?} needs {key}")))
    };
    let factor = || -> mlua::Result<f32> {
        let factor = number_of(&required("factor")?, "rule option factor")?;
        if !(0.0..=16.0).contains(&factor) {
            return Err(bad("rule option factor must be between 0 and 16"));
        }
        Ok(factor as f32)
    };
    let category = || -> mlua::Result<Option<Category>> {
        match options.get("category")? {
            None => Ok(None),
            Some(value) => match symbol_of(&value, "rule option category")? {
                LuaSymbol::Category(value) => Ok(Some(value)),
                _ => Err(bad("rule option category requires a Category value")),
            },
        }
    };
    Ok(match kind {
        RuleKind::DamageEachTurn => Rule::DamageEachTurn {
            fraction: fraction_of(&required("fraction")?, "rule option fraction")?,
            except_types: options.list("except_types", type_of)?,
            message: options.text("message")?,
        },
        RuleKind::HealEachTurn => Rule::HealEachTurn {
            fraction: fraction_of(&required("fraction")?, "rule option fraction")?,
            message: options.text("message")?,
        },
        RuleKind::DrainEachTurn => Rule::DrainEachTurn {
            fraction: fraction_of(&required("fraction")?, "rule option fraction")?,
            to: pokemon_of(&required("to")?, "rule option to")?,
            message: options.text("message")?,
        },
        RuleKind::DamageTaken => Rule::DamageTaken {
            category: category()?,
            move_type: options.pokemon_type("type")?,
            factor: factor()?,
            not_on_crit: options.flag("not_on_crit")?,
        },
        RuleKind::DamageDealt => Rule::DamageDealt {
            category: category()?,
            move_type: options.pokemon_type("type")?,
            factor: factor()?,
        },
        RuleKind::StatMultiplier => Rule::StatMultiplier {
            stat: combat_stat_of(&required("stat")?, "rule option stat")?,
            factor: factor()?,
        },
        RuleKind::BlockStatus => Rule::BlockStatus {
            statuses: options
                .list("statuses", status_of)?
                .into_iter()
                .map(AppliedStatus::battle_status)
                .collect(),
            confusion: options.flag("confusion")?,
        },
        RuleKind::BlockStatDrops => Rule::BlockStatDrops,
        RuleKind::EffectChance => Rule::EffectChance { factor: factor()? },
        RuleKind::MoveType => Rule::MoveType {
            from: type_of(&required("from")?, "rule option from")?,
            to: type_of(&required("to")?, "rule option to")?,
        },
        RuleKind::Grounded => Rule::Grounded,
        RuleKind::Trapped => Rule::Trapped,
        RuleKind::Endure => Rule::Endure,
        RuleKind::AlwaysHit => Rule::AlwaysHit {
            against: options.pokemon("against")?,
        },
        RuleKind::WithoutType => {
            Rule::WithoutType(type_of(&required("type")?, "rule option type")?)
        }
    })
}

fn turns_of(value: Option<Value>, what: &str) -> mlua::Result<Option<u8>> {
    value
        .map(|value| whole(&value, what, 1.0, 255.0).map(|turns| turns as u8))
        .transpose()
}

/// Builds one of the operations added after the first script API. `args` are the Lua arguments.
fn parse_op(name: &str, args: &[Value]) -> mlua::Result<ScriptOperation> {
    let arg = |index: usize| args.get(index).unwrap_or(&Value::Nil);
    let given = |index: usize| args.get(index).filter(|value| !value.is_nil());
    let who = |index: usize| pokemon_of(arg(index), name);
    let chance_only = |index: usize| -> mlua::Result<f32> {
        Options::new(given(index), "effect", &["chance"])?.chance()
    };
    Ok(match name {
        "multi_hit" => {
            let min = whole(arg(1), "multi_hit minimum", 1.0, 10.0)? as u8;
            let max = whole(arg(2), "multi_hit maximum", 1.0, 10.0)? as u8;
            if min > max {
                return Err(bad("multi_hit minimum exceeds maximum"));
            }
            ScriptOperation::MultiHit {
                power: power_of(arg(0))?,
                min,
                max,
                options: damage_options(given(3))?,
            }
        }
        "direct_damage" => {
            let options = Options::new(given(2), "direct_damage", &["typeless", "min_target_hp"])?;
            ScriptOperation::DirectDamage {
                who: who(0)?,
                amount: floored(arg(1), "direct_damage amount")?,
                typeless: options.flag("typeless")?,
                min_target_hp: options.whole("min_target_hp", 0.0, 65535.0)?.unwrap_or(0.0) as u16,
            }
        }
        "try_hit" => ScriptOperation::TryHit {
            who: given(0).map(|value| pokemon_of(value, name)).transpose()?,
            options: hit_options(&Options::new(given(1), "try_hit", &HIT_KEYS)?)?,
        },
        "chance" => ScriptOperation::Chance(fraction_of(arg(0), "chance")?),
        "effect_chance" => ScriptOperation::EffectChance(fraction_of(arg(0), "effect_chance")?),
        "hurt" => ScriptOperation::Hurt {
            who: who(0)?,
            amount: hp_amount(arg(1), "hurt")?,
        },
        "heal" => ScriptOperation::Heal {
            who: who(0)?,
            amount: hp_amount(arg(1), "heal")?,
        },
        "apply_status" => {
            let options = Options::new(
                given(2),
                "apply_status",
                &["chance", "replace", "announce_failure"],
            )?;
            ScriptOperation::ApplyStatus {
                who: who(0)?,
                status: status_of(arg(1), "apply_status")?,
                chance: options.chance()?,
                replace: options.flag("replace")?,
                announce_failure: options.flag("announce_failure")?,
            }
        }
        "cure_status" => ScriptOperation::CureStatus {
            who: who(0)?,
            status: given(1)
                .map(|value| status_of(value, "cure_status"))
                .transpose()?,
        },
        "confuse" => ScriptOperation::Confuse {
            who: who(0)?,
            chance: chance_only(1)?,
        },
        "flinch" => ScriptOperation::Flinch {
            who: who(0)?,
            chance: chance_only(1)?,
        },
        "change_stat" => {
            let stages = whole(arg(2), "stat change", -6.0, 6.0)? as i8;
            if stages == 0 {
                return Err(bad("stat change must be nonzero and within -6..=6"));
            }
            ScriptOperation::ChangeStat {
                who: who(0)?,
                stat: stat_of(arg(1), "change_stat")?,
                stages,
                chance: chance_only(3)?,
            }
        }
        "reset_stats" => ScriptOperation::ResetStats { who: who(0)? },
        "bind" => {
            let options = Options::new(given(1), "bind", &["min_turns", "max_turns"])?;
            let min_turns = options.whole("min_turns", 1.0, 15.0)?.unwrap_or(4.0) as u8;
            let max_turns = options.whole("max_turns", 1.0, 15.0)?.unwrap_or(5.0) as u8;
            if min_turns > max_turns {
                return Err(bad("bind minimum exceeds maximum"));
            }
            ScriptOperation::Bind {
                who: who(0)?,
                min_turns,
                max_turns,
            }
        }
        "free" => ScriptOperation::Free { who: who(0)? },
        "protect" => ScriptOperation::Protect { who: who(0)? },
        "break_protect" => ScriptOperation::BreakProtect { who: who(0)? },
        "hide" => ScriptOperation::Hide(hidden_of(arg(0), "hide")?),
        "unhide" => ScriptOperation::Unhide { who: who(0)? },
        "set_weather" => ScriptOperation::SetWeather {
            kind: match symbol_of(arg(0), "set_weather")? {
                LuaSymbol::Weather(kind) => kind,
                _ => return Err(bad("set_weather requires a Weather value")),
            },
            turns: whole(arg(1), "set_weather turns", 1.0, 255.0)? as u8,
        },
        "clear_weather" => ScriptOperation::ClearWeather,
        "mark" => ScriptOperation::Mark {
            scope: scope_of(arg(0), name)?,
            name: name_of(arg(1), "mark name")?,
            value: given(2)
                .map(|value| whole(value, "mark value", -1_000_000.0, 1_000_000.0))
                .transpose()?
                .unwrap_or(1.0) as i32,
            turns: turns_of(given(3).cloned(), "mark turns")?,
        },
        "unmark" => ScriptOperation::Unmark {
            scope: scope_of(arg(0), name)?,
            name: name_of(arg(1), "mark name")?,
        },
        "start_effect" => {
            let options = Options::new(
                given(2),
                "start_effect",
                &["turns", "rules", "group", "end_message", "restart", "value"],
            )?;
            ScriptOperation::StartEffect {
                scope: scope_of(arg(0), name)?,
                condition: Condition {
                    name: name_of(arg(1), "effect name")?,
                    value: options
                        .whole("value", -1_000_000.0, 1_000_000.0)?
                        .unwrap_or(1.0) as i32,
                    turns_left: turns_of(options.get("turns")?, "start_effect option turns")?,
                    rules: options.list("rules", |value, _| rule_of(value))?,
                    group: options.text("group")?,
                    end_message: options.text("end_message")?,
                },
                restart: options.flag("restart")?,
            }
        }
        "end_effect" => ScriptOperation::EndEffect {
            scope: scope_of(arg(0), name)?,
            name: name_of(arg(1), "effect name")?,
        },
        "end_group" => ScriptOperation::EndGroup {
            scope: scope_of(arg(0), name)?,
            group: name_of(arg(1), "effect group")?,
        },
        "set_types" => {
            let Value::Table(list) = arg(1) else {
                return Err(bad("set_types requires a list of Type values"));
            };
            let types = list
                .clone()
                .sequence_values::<Value>()
                .map(|value| type_of(&value?, "set_types"))
                .collect::<mlua::Result<Vec<_>>>()?;
            if types.is_empty() || types.len() > 3 {
                return Err(bad("set_types takes one to three types"));
            }
            ScriptOperation::SetTypes {
                who: who(0)?,
                types,
            }
        }
        "set_weight" => {
            let weight = number_of(arg(1), "set_weight")?;
            if !(0.1..=9999.9).contains(&weight) {
                return Err(bad("set_weight must be between 0.1 and 9999.9"));
            }
            ScriptOperation::SetWeight {
                who: who(0)?,
                weight: weight as f32,
            }
        }
        "take_item" => ScriptOperation::TakeItem { who: who(0)? },
        "give_item" => ScriptOperation::GiveItem {
            who: who(0)?,
            item: name_of(arg(1), "item")?,
        },
        "suppress_ability" => ScriptOperation::SuppressAbility { who: who(0)? },
        "add_payout" => {
            ScriptOperation::AddPayout(whole(arg(0), "add_payout", 0.0, 1_000_000.0)? as u32)
        }
        "hurry" => ScriptOperation::Hurry,
        _ => return Err(bad(format!("unknown battle operation {name}"))),
    })
}

#[derive(Clone, Debug)]
pub struct MoveCatalog {
    moves: HashMap<String, MoveSpec>,
    lua: Lua,
    script_factory: Function,
}

impl MoveCatalog {
    pub fn builtin() -> Result<Self, BattleError> {
        Self::from_lua(include_str!(concat!(env!("OUT_DIR"), "/builtin_moves.lua")))
    }
    pub fn from_directory(path: impl AsRef<Path>) -> Result<Self, BattleError> {
        let path = path.as_ref();
        let mut files = std::fs::read_dir(path)
            .map_err(|error| BattleError::InvalidSetup(format!("{}: {error}", path.display())))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| BattleError::InvalidSetup(error.to_string()))?;
        files.retain(|file| file.extension().is_some_and(|extension| extension == "lua"));
        files.sort();
        if files.is_empty() {
            return Err(BattleError::InvalidSetup(format!(
                "no Lua moves in {}",
                path.display()
            )));
        }
        let lua = Lua::new();
        lua_symbols::install(&lua)?;
        let mut moves = HashMap::new();
        for file in files {
            let move_source = std::fs::read_to_string(&file).map_err(|error| {
                BattleError::InvalidSetup(format!("{}: {error}", file.display()))
            })?;
            let entry: Table = lua
                .load(&move_source)
                .set_name(file.display().to_string())
                .eval()?;
            let spec = parse_move(&entry).map_err(|error| {
                BattleError::InvalidSetup(format!("{}: {error}", file.display()))
            })?;
            if moves.insert(spec.id.clone(), spec).is_some() {
                return Err(BattleError::InvalidSetup(format!(
                    "{}: duplicate move id",
                    file.display()
                )));
            }
        }
        let system: Table = lua.load(include_str!("system_moves.lua")).eval()?;
        let spec = parse_move(&system)?;
        if moves.insert(spec.id.clone(), spec).is_some() {
            return Err(BattleError::InvalidSetup("reserved system move id".into()));
        }
        let script_factory = lua.load(include_str!("script_api.lua")).eval()?;
        Ok(Self {
            moves,
            lua,
            script_factory,
        })
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
    pub(crate) fn lua(&self) -> &Lua {
        &self.lua
    }
    /// Starts one hook of a scripted move. `fields` is what the script sees as `ctx`, before the
    /// battle operations are added to it; `roster` lists the table of every Pokémon.
    pub(crate) fn start_script(
        &self,
        spec: &MoveSpec,
        hook: ScriptHook,
        fields: Table,
        roster: Table,
    ) -> Result<(Thread, Table), BattleError> {
        let function = match hook {
            ScriptHook::Main => spec.script.as_ref().expect("scripted move"),
            ScriptHook::Interrupt => spec.on_interrupt.as_ref().expect("interrupt hook"),
            ScriptHook::Hit => spec.on_hit.as_ref().expect("hit hook"),
            ScriptHook::TurnStart => spec.on_turn_start.as_ref().expect("turn start hook"),
        };
        let api = self.lua.create_table()?;
        api.set(
            "damage",
            self.lua
                .create_function(|lua, (power, options): (Value, Option<Value>)| {
                    lua.create_userdata(ScriptOperation::Damage {
                        power: power_of(&power)?,
                        options: damage_options(options.as_ref())?,
                    })
                })?,
        )?;
        api.set(
            "op",
            self.lua
                .create_function(|lua, (name, args): (String, Variadic<Value>)| {
                    lua.create_userdata(parse_op(&name, &args)?)
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
        api.set(
            "change_self_stat",
            self.lua
                .create_function(|lua, (stat, stages): (mlua::AnyUserData, i8)| {
                    let stat = match lua_symbols::from_userdata(stat, "stat") {
                        Ok(LuaSymbol::Stat(value)) => value,
                        _ => return Err(mlua::Error::external("stat requires Stat value")),
                    };
                    if !(-6..=6).contains(&stages) || stages == 0 {
                        return Err(mlua::Error::external(
                            "stat change must be nonzero and within -6..=6",
                        ));
                    }
                    lua.create_userdata(ScriptOperation::ChangeSelfStat { stat, stages })
                })?,
        )?;
        api.set(
            "boost_next_move",
            self.lua.create_function(
                |lua, (move_type, multiplier): (mlua::AnyUserData, f32)| {
                    let move_type = match lua_symbols::from_userdata(move_type, "type") {
                        Ok(LuaSymbol::Type(value)) => value,
                        _ => return Err(mlua::Error::external("move type requires Type value")),
                    };
                    if !multiplier.is_finite() || multiplier <= 0.0 || multiplier > 8.0 {
                        return Err(mlua::Error::external(
                            "power multiplier must be greater than 0 and at most 8",
                        ));
                    }
                    lua.create_userdata(ScriptOperation::BoostNextMove {
                        move_type,
                        multiplier,
                    })
                },
            )?,
        )?;
        api.set(
            "watch_hits_until_next_action",
            self.lua.create_function(|lua, ()| {
                lua.create_userdata(ScriptOperation::WatchHitsUntilNextAction)
            })?,
        )?;
        api.set(
            "heal_self",
            self.lua.create_function(|lua, fraction: f32| {
                if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                    return Err(mlua::Error::external(
                        "heal fraction must be between 0 and 1",
                    ));
                }
                lua.create_userdata(ScriptOperation::HealSelf(fraction))
            })?,
        )?;
        api.set(
            "flinch_target",
            self.lua.create_function(|lua, chance: f32| {
                if !chance.is_finite() || !(0.0..=1.0).contains(&chance) {
                    return Err(mlua::Error::external(
                        "flinch chance must be between 0 and 1",
                    ));
                }
                lua.create_userdata(ScriptOperation::FlinchTarget(chance))
            })?,
        )?;
        let input: Table = self.script_factory.call((api, fields, roster))?;
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
    let on_hit: Option<Function> = t.get("on_hit")?;
    let on_turn_start: Option<Function> = t.get("on_turn_start")?;
    if on_turn_start.is_some() && script.is_none() {
        return Err(BattleError::InvalidSetup(format!(
            "turn start hook without script for {id}"
        )));
    }
    let flags = match t.get::<Option<Table>>("flags")? {
        Some(list) => list
            .sequence_values::<String>()
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let hits_hidden = match t.get::<Option<Table>>("hits_hidden")? {
        Some(list) => list
            .sequence_values::<Value>()
            .map(|value| hidden_of(&value?, "hits_hidden"))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    if on_interrupt.is_some() && script.is_none() {
        return Err(BattleError::InvalidSetup(format!(
            "interrupt hook without script for {id}"
        )));
    }
    if on_hit.is_some() && script.is_none() {
        return Err(BattleError::InvalidSetup(format!(
            "hit hook without script for {id}"
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
        flags,
        hits_hidden,
        script,
        on_interrupt,
        on_hit,
        on_turn_start,
        manual_announce: t.get::<Option<bool>>("manual_announce")?.unwrap_or(false),
        auto_only: t.get::<Option<bool>>("auto_only")?.unwrap_or(false),
        usable_while_asleep: t
            .get::<Option<bool>>("usable_while_asleep")?
            .unwrap_or(false),
        usable_while_frozen: t
            .get::<Option<bool>>("usable_while_frozen")?
            .unwrap_or(false),
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
            hidden: match lua_symbols::optional(t, "hidden")? {
                None => HiddenKind::Vanished,
                Some(LuaSymbol::Hidden(value)) => value,
                _ => {
                    return Err(BattleError::InvalidSetup(
                        "hidden requires Hidden value".into(),
                    ));
                }
            },
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
