use std::collections::BTreeMap;

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
    pub status: Option<Status>,
    pub status_turns: u8,
    pub stages: BTreeMap<Stat, i8>,
    pub confused_turns: Option<u8>,
    pub bound: Option<Bound>,
    pub protected: bool,
    pub flinched: bool,
    pub locked_move: Option<LockedMove>,
    pub last_move: Option<String>,
    pub consecutive_count: u8,
    pub crit_stage: u8,
    pub semi_invulnerable: bool,
    pub recharging: bool,
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
            status: None,
            status_turns: 0,
            stages: BTreeMap::new(),
            confused_turns: None,
            bound: None,
            protected: false,
            flinched: false,
            locked_move: None,
            last_move: None,
            consecutive_count: 0,
            crit_stage: 0,
            semi_invulnerable: false,
            recharging: false,
        }
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
pub struct Choice {
    pub id: u32,
    pub move_id: String,
    pub target: ParticipantId,
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
