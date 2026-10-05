use crate::moves::Category;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    Allies,
    Foes,
}
impl Side {
    pub const ALL: [Self; 2] = [Self::Allies, Self::Foes];
    pub const fn index(self) -> usize {
        match self {
            Self::Allies => 0,
            Self::Foes => 1,
        }
    }
    pub const fn opposite(self) -> Self {
        match self {
            Self::Allies => Self::Foes,
            Self::Foes => Self::Allies,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PokemonType {
    Normal,
    Fighting,
    Flying,
    Poison,
    Ground,
    Rock,
    Bug,
    Ghost,
    Steel,
    Fire,
    Water,
    Grass,
    Electric,
    Psychic,
    Ice,
    Dragon,
    Dark,
    Fairy,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stat {
    Attack,
    Defense,
    SpAttack,
    SpDefense,
    Speed,
    Accuracy,
    Evasion,
}
impl Stat {
    pub const fn description(self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Defense => "Defense",
            Self::SpAttack => "Special Attack",
            Self::SpDefense => "Special Defense",
            Self::Speed => "Speed",
            Self::Accuracy => "Accuracy",
            Self::Evasion => "Evasion",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatStat {
    Attack,
    Defense,
    SpAttack,
    SpDefense,
    Speed,
}
impl From<CombatStat> for Stat {
    fn from(value: CombatStat) -> Self {
        match value {
            CombatStat::Attack => Self::Attack,
            CombatStat::Defense => Self::Defense,
            CombatStat::SpAttack => Self::SpAttack,
            CombatStat::SpDefense => Self::SpDefense,
            CombatStat::Speed => Self::Speed,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherKind {
    Sun,
    Sandstorm,
    Rain,
    Hail,
}

impl WeatherKind {
    /// The lines shown when the weather starts, at the end of each turn, and when it stops.
    pub const fn messages(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Sun => (
                "The sunlight turned harsh!",
                "The sunlight is strong.",
                "The harsh sunlight faded.",
            ),
            Self::Sandstorm => (
                "A sandstorm kicked up!",
                "The sandstorm rages.",
                "The sandstorm subsided.",
            ),
            Self::Rain => (
                "It started to rain!",
                "Rain continues to fall.",
                "The rain stopped.",
            ),
            Self::Hail => (
                "It started to hail!",
                "Hail continues to fall.",
                "The hail stopped.",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    #[default]
    Genderless,
}

/// Where a semi-invulnerable Pokémon is. Moves name the places they still reach.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HiddenKind {
    Air,
    Underground,
    Underwater,
    #[default]
    Vanished,
}

/// What a condition is attached to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Pokemon(ParticipantId),
    Side(Side),
    Field,
}

/// One thing the engine enforces for as long as a condition lasts.
/// A rule on a side applies to every Pokémon of that side, one on the field to everyone.
#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    /// Loses this share of max HP at the end of each turn.
    DamageEachTurn {
        fraction: f32,
        except_types: Vec<PokemonType>,
        message: Option<String>,
    },
    /// Regains this share of max HP at the end of each turn.
    HealEachTurn {
        fraction: f32,
        message: Option<String>,
    },
    /// Loses this share of max HP at the end of each turn, and `to` regains what was lost.
    DrainEachTurn {
        fraction: f32,
        to: ParticipantId,
        message: Option<String>,
    },
    /// Damage from moves against the holder is multiplied.
    DamageTaken {
        category: Option<Category>,
        move_type: Option<PokemonType>,
        factor: f32,
        not_on_crit: bool,
    },
    /// Damage from the holder's moves is multiplied.
    DamageDealt {
        category: Option<Category>,
        move_type: Option<PokemonType>,
        factor: f32,
    },
    StatMultiplier {
        stat: CombatStat,
        factor: f32,
    },
    /// The listed statuses (all of them if the list is empty) cannot be inflicted by others.
    BlockStatus {
        statuses: Vec<Status>,
        confusion: bool,
    },
    /// Stats cannot be lowered by others.
    BlockStatDrops,
    /// Chances of the extra effects of the holder's moves are multiplied.
    EffectChance {
        factor: f32,
    },
    /// Moves of one type are used as another.
    MoveType {
        from: PokemonType,
        to: PokemonType,
    },
    /// Loses any immunity to Ground moves.
    Grounded,
    /// Cannot leave the battle. The engine only reports this; it has no switching.
    Trapped,
    /// Survives damage from moves with 1 HP.
    Endure,
    /// The holder's moves cannot miss (only against `against`, if given) and reach hidden targets.
    AlwaysHit {
        against: Option<ParticipantId>,
    },
    /// One of the holder's types is ignored.
    WithoutType(PokemonType),
}

/// A named mark or timed effect on a Pokémon, a side or the field.
/// Moves set these to remember things and to switch engine rules on for a while.
#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub name: String,
    /// A counter that scripts read back; 1 for a plain mark.
    pub value: i32,
    /// Turns left, counting the current one; `None` lasts until removed.
    pub turns_left: Option<u8>,
    pub rules: Vec<Rule>,
    /// Starting a condition ends any other condition of the same group in the same place.
    pub group: Option<String>,
    pub end_message: Option<String>,
}

/// The last hit a Pokémon took from a move this turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitInfo {
    pub attacker: ParticipantId,
    pub move_id: String,
    pub category: Category,
    pub move_type: PokemonType,
    pub damage: u16,
    pub contact: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParticipantId {
    pub side: Side,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Poisoned,
    BadlyPoisoned,
    Burned,
    Paralyzed,
    Asleep,
    Frozen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppliedStatus {
    Poisoned,
    BadlyPoisoned,
    Burned,
    Paralyzed,
    Asleep,
    RestSleep,
    Frozen,
}
impl AppliedStatus {
    pub const fn battle_status(self) -> Status {
        match self {
            Self::Poisoned => Status::Poisoned,
            Self::BadlyPoisoned => Status::BadlyPoisoned,
            Self::Burned => Status::Burned,
            Self::Paralyzed => Status::Paralyzed,
            Self::Asleep | Self::RestSleep => Status::Asleep,
            Self::Frozen => Status::Frozen,
        }
    }
    pub const fn description(self) -> &'static str {
        match self {
            Self::Poisoned => "poisoned",
            Self::BadlyPoisoned => "badly poisoned",
            Self::Burned => "burned",
            Self::Paralyzed => "paralyzed",
            Self::Asleep | Self::RestSleep => "asleep",
            Self::Frozen => "frozen",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bound {
    pub turns_left: u8,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedMove {
    pub move_id: String,
    pub target: ParticipantId,
    pub turns: u8,
    pub max_turns: u8,
    pub charging: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuationTarget {
    SameTarget,
    RandomOpponent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptContinuation {
    pub move_id: String,
    pub target: ParticipantId,
    pub turn: u8,
    pub total_turns: u8,
    pub target_policy: ContinuationTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NextMovePowerBoost {
    pub move_type: PokemonType,
    pub multiplier: f32,
}

#[derive(Clone, Debug)]
pub struct Pokemon {
    pub species_id: u32,
    pub name: String,
    pub level: u8,
    pub hp: u16,
    pub max_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
    pub types: Vec<PokemonType>,
    pub moves: Vec<String>,
    pub move_pp: BTreeMap<String, u8>,
    pub status: Option<Status>,
    pub status_turns: u8,
    pub stages: BTreeMap<Stat, i8>,
    pub confused_turns: Option<u8>,
    pub bound: Option<Bound>,
    pub protected: bool,
    pub flinched: bool,
    pub locked_move: Option<LockedMove>,
    pub script_continuation: Option<ScriptContinuation>,
    pub next_move_power_boost: Option<NextMovePowerBoost>,
    pub last_move: Option<String>,
    pub consecutive_count: u8,
    pub crit_stage: u8,
    pub semi_invulnerable: bool,
    pub recharging: bool,
    /// Free-form species name for scripts that care (Chatter, Relic Song).
    pub species: String,
    pub gender: Gender,
    /// Kilograms; 0 when unknown.
    pub weight: f32,
    /// Held item id. The engine does not know what items do.
    pub item: Option<String>,
    /// Ability id. The engine does not know what abilities do.
    pub ability: Option<String>,
    pub ability_suppressed: bool,
    /// HP, Attack, Defense, Sp. Attack, Sp. Defense, Speed.
    pub ivs: Option<[u8; 6]>,
    pub happiness: Option<u8>,
    /// Where it is while `semi_invulnerable`.
    pub hidden_kind: HiddenKind,
    pub conditions: Vec<Condition>,
    /// The battle turn it entered on (0 at the start).
    pub entered_turn: u64,
    /// Whether it has finished its action this turn.
    pub acted: bool,
    /// HP lost to moves this turn.
    pub damage_taken: u16,
    /// Whether it lost HP for any reason this turn.
    pub hurt_this_turn: bool,
    pub last_hit: Option<HitInfo>,
    pub moves_used: BTreeSet<String>,
    /// The move it last tried to use, whatever came of it.
    pub last_used: Option<String>,
    pub last_move_failed: bool,
    /// How many turns in a row `streak_move` has been used without failing.
    pub streak_move: Option<String>,
    pub streak: u8,
}

impl Pokemon {
    pub fn new(name: impl Into<String>, moves: Vec<String>) -> Self {
        Self {
            species_id: 0,
            name: name.into(),
            level: 100,
            hp: 100,
            max_hp: 100,
            attack: 100,
            defense: 100,
            sp_attack: 100,
            sp_defense: 100,
            speed: 100,
            types: vec![PokemonType::Normal],
            moves,
            move_pp: BTreeMap::new(),
            status: None,
            status_turns: 0,
            stages: BTreeMap::new(),
            confused_turns: None,
            bound: None,
            protected: false,
            flinched: false,
            locked_move: None,
            script_continuation: None,
            next_move_power_boost: None,
            last_move: None,
            consecutive_count: 0,
            crit_stage: 0,
            semi_invulnerable: false,
            recharging: false,
            species: String::new(),
            gender: Gender::Genderless,
            weight: 0.0,
            item: None,
            ability: None,
            ability_suppressed: false,
            ivs: None,
            happiness: None,
            hidden_kind: HiddenKind::Vanished,
            conditions: Vec::new(),
            entered_turn: 0,
            acted: false,
            damage_taken: 0,
            hurt_this_turn: false,
            last_hit: None,
            moves_used: BTreeSet::new(),
            last_used: None,
            last_move_failed: false,
            streak_move: None,
            streak: 0,
        }
    }

    /// Where the Pokémon is hiding, if it is semi-invulnerable.
    pub fn hidden(&self) -> Option<HiddenKind> {
        self.semi_invulnerable.then_some(self.hidden_kind)
    }
    pub fn condition(&self, name: &str) -> Option<&Condition> {
        self.conditions
            .iter()
            .find(|condition| condition.name == name)
    }

    pub fn stage(&self, stat: Stat) -> i8 {
        *self.stages.get(&stat).unwrap_or(&0)
    }
    pub fn change_stage(&mut self, stat: Stat, delta: i8) -> i8 {
        let old = self.stage(stat);
        let new = (old + delta).clamp(-6, 6);
        self.stages.insert(stat, new);
        new - old
    }
    pub fn effective_stat(&self, stat: CombatStat) -> f32 {
        let base = match stat {
            CombatStat::Attack => self.attack,
            CombatStat::Defense => self.defense,
            CombatStat::SpAttack => self.sp_attack,
            CombatStat::SpDefense => self.sp_defense,
            CombatStat::Speed => self.speed,
        } as f32;
        let stage = self.stage(stat.into()) as f32;
        let factor = if stage >= 0.0 {
            (stage + 2.0) / 2.0
        } else {
            2.0 / (2.0 - stage)
        };
        let result = (base * factor).trunc();
        if stat == CombatStat::Speed && self.status == Some(Status::Paralyzed) {
            (result / 2.0).trunc()
        } else {
            result
        }
    }
    pub fn accuracy_factor(&self) -> f32 {
        let stage = self.stage(Stat::Accuracy) as f32;
        if stage >= 0.0 {
            (3.0 + stage) / 3.0
        } else {
            3.0 / (3.0 - stage)
        }
    }
    pub fn evasion_factor(&self) -> f32 {
        let stage = -(self.stage(Stat::Evasion) as f32);
        if stage >= 0.0 {
            (3.0 + stage) / 3.0
        } else {
            3.0 / (3.0 - stage)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weather {
    pub kind: WeatherKind,
    pub turns_left: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BattleEvent {
    Message(String),
    Damage {
        target: ParticipantId,
        amount: u16,
        hp: u16,
    },
    Heal {
        target: ParticipantId,
        amount: u16,
        hp: u16,
    },
    Status {
        target: ParticipantId,
        status: Option<Status>,
    },
    StatChange {
        target: ParticipantId,
        stat: Stat,
        stages: i8,
    },
    Weather(Option<Weather>),
    Fainted(ParticipantId),
    End {
        winner: Option<Side>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    UseMove {
        id: u32,
        move_id: String,
        target: ParticipantId,
    },
}

impl Choice {
    pub const fn id(&self) -> u32 {
        match self {
            Self::UseMove { id, .. } => *id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub id: u64,
    pub actor: ParticipantId,
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvanceStatus {
    Awaiting(Prompt),
    End { winner: Option<Side> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvanceResult {
    pub events: Vec<BattleEvent>,
    pub status: AdvanceStatus,
}

#[derive(Clone, Debug)]
pub struct ActionSelection {
    pub prompt_id: u64,
    pub choice_id: u32,
}

#[derive(Debug)]
pub enum BattleError {
    Script(mlua::Error),
    InvalidSetup(String),
    InvalidResponse(String),
}
impl std::fmt::Display for BattleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Script(e) => write!(f, "Lua: {e}"),
            Self::InvalidSetup(e) | Self::InvalidResponse(e) => f.write_str(e),
        }
    }
}
impl std::error::Error for BattleError {}
impl From<mlua::Error> for BattleError {
    fn from(value: mlua::Error) -> Self {
        Self::Script(value)
    }
}
