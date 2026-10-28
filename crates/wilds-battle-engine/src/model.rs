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
    HeavyRain,
    HarshSun,
    StrongWinds,
}

impl WeatherKind {
    pub(crate) fn owner_ability(self) -> Option<&'static str> {
        match self {
            Self::HeavyRain => Some("primordialsea"),
            Self::HarshSun => Some("desolateland"),
            Self::StrongWinds => Some("deltastream"),
            _ => None,
        }
    }
    pub(crate) fn battle_kind(self) -> Self {
        match self {
            Self::HeavyRain => Self::Rain,
            Self::HarshSun => Self::Sun,
            other => other,
        }
    }
    /// The lines shown when the weather starts, at the end of each turn, and when it stops.
    pub const fn messages(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::StrongWinds => (
                msg("Mysterious strong winds are protecting Flying-type Pokémon!"),
                msg("The mysterious strong winds blow on."),
                msg("The mysterious strong winds dissipated."),
            ),
            Self::HarshSun => (
                msg("The sunlight turned extremely harsh!"),
                msg("The extremely harsh sunlight continues."),
                msg("The extremely harsh sunlight faded."),
            ),
            Self::HeavyRain => (
                msg("A heavy rain began to fall!"),
                msg("The heavy rain continues."),
                msg("The heavy rain stopped."),
            ),
            Self::Sun => (
                msg("The sunlight turned harsh!"),
                msg("The sunlight is strong."),
                msg("The harsh sunlight faded."),
            ),
            Self::Sandstorm => (
                msg("A sandstorm kicked up!"),
                msg("The sandstorm rages."),
                msg("The sandstorm subsided."),
            ),
            Self::Rain => (
                msg("It started to rain!"),
                msg("Rain continues to fall."),
                msg("The rain stopped."),
            ),
            Self::Hail => (
                msg("It started to hail!"),
                msg("Hail continues to fall."),
                msg("The hail stopped."),
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
    /// Intercepts another user's eligible beneficial move once this turn.
    Snatch {
        moves: Vec<String>,
    },
    /// Restricts selected actions to this learned move while it has PP.
    OnlyMove {
        id: String,
    },
    /// Suppresses ordinary held-item effects without removing the item.
    BlockItems,
    /// Expires when the holder attempts its next action, except an optional move id.
    ExpiresOnAction {
        except_move: Option<String>,
    },
    /// Takes an opposing direct attacker down if its hit knocks the holder out.
    FaintAttacker,
    /// Empties the PP of the direct attacker's selected move after the holder faints.
    DepleteAttackerPp,
    /// Hit protection, removed by moves that break Protect. Filters are combined.
    Guard {
        spread_only: bool,
        priority_only: bool,
        damaging_only: bool,
        status_only: bool,
        unbypassable: bool,
        contact_stat: Option<(Stat, i8)>,
        contact_damage: f32,
        contact_status: Option<AppliedStatus>,
    },
    /// Blocks opposing moves with positive priority, optionally only for grounded holders.
    BlockPriority {
        grounded_only: bool,
    },
    /// Base power of incoming moves, filtered by move id and/or type.
    IncomingBasePower {
        moves: Vec<String>,
        move_type: Option<PokemonType>,
        factor: f32,
        grounded_only: bool,
    },
    /// Status protection for grounded Pokemon, including self-inflicted status.
    BlockGroundedStatus {
        statuses: Vec<Status>,
        confusion: bool,
    },
    /// Ordinary incoming attacks cannot miss; does not affect OHKO checks or hiding.
    IncomingAlwaysHit,
    /// Floats above the ground unless another rule grounds the holder.
    Airborne,
    /// Prevents HP restoration; direct HP assignment and switch-out Regenerator bypass it.
    BlockHealing,
    /// Cancels a move of this type and inflicts residual damage on its user.
    ExplodeOnType {
        move_type: PokemonType,
        fraction: f32,
        message: Option<String>,
    },
    /// Reflects moves explicitly marked reflectable, without recursive rebounds.
    ReflectMoves,
    /// Exchanges raw Defense and Sp. Defense before their own stage modifiers.
    SwapDefenses,
    /// Reverses Speed within each action priority bracket.
    ReverseSpeed,
    /// A countdown which faints the holder on natural expiry, bypassing damage protection.
    FaintOnExpiry,
    /// Inflicts a status when a timed Pokemon condition expires naturally.
    StatusOnExpiry {
        status: AppliedStatus,
        source: ParticipantId,
    },
    /// Changes a stat at turn end while the source stays on the field.
    StatEachTurn {
        stat: Stat,
        stages: i8,
        source: ParticipantId,
    },
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
        grounded_only: bool,
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
        /// Alternate factor for battles with multiple active positions on either side.
        multi_battle_factor: Option<f32>,
        /// Screens are bypassed by Infiltrator and do not combine with the named side effect.
        screen: bool,
        unless_side_effect: Option<String>,
    },
    /// Damage from the holder's moves is multiplied.
    DamageDealt {
        category: Option<Category>,
        move_type: Option<PokemonType>,
        factor: f32,
    },
    /// Ordinary accuracy checks are multiplied without changing stat stages or OHKO checks.
    AccuracyMultiplier {
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
    /// Redirects single-target moves aimed at the holder; higher priority rules win.
    Redirect {
        to: ParticipantId,
        move_type: Option<PokemonType>,
        priority: i8,
    },
    /// Draws opposing single-target moves to a teammate, with optional powder immunity.
    Attention {
        to: ParticipantId,
        priority: i8,
        powder: bool,
        allow_carried: bool,
    },
    /// Adds to the priority of the holder's selected move, before turn actions begin.
    MovePriority {
        amount: i8,
    },
    /// Moves with any listed flag cannot be selected or executed by the holder.
    BlockMoves {
        flags: Vec<String>,
        message: Option<String>,
    },
    BlockCategory {
        category: crate::moves::Category,
        except: Vec<String>,
        message: Option<String>,
    },
    BlockMove {
        id: String,
        message: Option<String>,
    },
    /// Prevent choosing the last used move again; does not interrupt an already queued action.
    NoRepeat,
    /// While the holder is active, foes cannot choose or use its currently learned moves.
    SealMoves,
    /// Modifies base power before the damage formula, optionally for one move type.
    BasePower {
        move_type: Option<PokemonType>,
        factor: f32,
        grounded_only: bool,
    },
    /// Stats cannot be lowered by others.
    BlockStatDrops,
    /// Flinching cannot interrupt the holder.
    BlockFlinch,
    /// Chances of the extra effects of the holder's moves are multiplied.
    EffectChance {
        factor: f32,
    },
    /// Adds to the holder's critical-hit stage while this condition remains active.
    CriticalStage {
        stages: u8,
    },
    /// Incoming attacks cannot score critical hits, including guaranteed criticals.
    BlockCritical,
    /// Moves of one type are used as another.
    MoveType {
        from: PokemonType,
        to: PokemonType,
    },
    /// Loses any immunity to Ground moves.
    Grounded,
    /// Cannot voluntarily switch out.
    Trapped,
    /// Resists forced replacement (for example rooting with Ingrain).
    BlockForcedSwitch,
    /// Cannot voluntarily switch while this source remains alive on the field.
    TrappedBy {
        source: ParticipantId,
    },
    /// Survives damage from moves with 1 HP.
    Endure,
    /// The holder's moves cannot miss (only against `against`, if given) and reach hidden targets.
    AlwaysHit {
        against: Option<ParticipantId>,
    },
    /// One of the holder's types is ignored.
    WithoutType(PokemonType),
    /// Disregard this defending type only for the specified incoming move types.
    NegateTypeImmunity {
        defending: PokemonType,
        incoming: Vec<PokemonType>,
    },
    IgnoreEvasionBoost,
    /// Remove this Pokémon condition when it no longer has the required status.
    RequiresStatus(Status),
    /// Source-linked infatuation; half of attempted moves are prevented.
    InfatuatedBy(ParticipantId),
    /// Changes incoming type effectiveness, including immunity/ability and berry checks.
    TypeEffectiveness {
        move_type: PokemonType,
        factor: f32,
    },
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
    pub by: Option<ParticipantId>,
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

/// One form's host-computed stats. Optional max HP preserves missing HP on a form change.
/// Transform never copies HP; stages, status and PP are preserved by form changes.
#[derive(Clone, Debug, PartialEq)]
pub struct PokemonForm {
    pub max_hp: Option<u16>,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
    pub types: Vec<PokemonType>,
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct AbilityState {
    pub id: Option<String>,
    pub rules: Vec<Rule>,
    pub bypassable: Option<bool>,
}

/// Visual identity used by Illusion; combat always uses the real Pokémon fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PokemonAppearance {
    pub name: String,
    pub species_id: u32,
    pub species: String,
    pub form: Option<String>,
    pub gender: Gender,
}

#[derive(Clone, Debug)]
pub struct Pokemon {
    pub illusion: Option<PokemonAppearance>,
    pub species_id: u32,
    /// Species base Attack of the original party member, for Beat Up; independent of battle stats.
    pub base_attack: Option<u16>,
    pub form: Option<String>,
    pub forms: BTreeMap<String, PokemonForm>,
    /// Saved on the first temporary form change and restored when the Pokemon leaves the field.
    pub original_form: Option<(Option<String>, PokemonForm)>,
    pub(crate) battle_base_form: Option<(Option<String>, PokemonForm)>,
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
    /// Additional type from Trick-or-Treat or Forest's Curse; a new added type replaces it.
    pub added_type: Option<PokemonType>,
    pub moves: Vec<String>,
    pub move_pp: BTreeMap<String, u8>,
    /// Maximum PP per move when it differs from the move's base PP (PP Ups); the host's to set.
    pub pp_max: BTreeMap<String, u8>,
    pub(crate) transformation: Option<crate::transform::OriginalPokemon>,
    pub(crate) temporary_moves: BTreeMap<usize, (String, u8)>,
    pub status: Option<Status>,
    pub status_turns: u8,
    pub stages: BTreeMap<Stat, i8>,
    pub confused_turns: Option<u8>,
    /// Capture attempts fail and the ball is not used up.
    pub uncatchable: bool,
    /// An appearance the host gives, shown until a script reveals it (`ctx:reveal`). Unlike
    /// the Illusion ability's, damage and suppression do not end it.
    pub disguise: Option<PokemonAppearance>,
    /// Species catch rate (3 to 255); a Pokémon without one cannot be caught.
    pub catch_rate: Option<u8>,
    /// Species base Speed, for scripts such as the Fast Ball's.
    pub base_speed: Option<u16>,
    /// Facts the host records about this Pokémon for scripts (`who.flags.name`), such as a
    /// species already registered; scripts can set and clear them too.
    pub flags: BTreeSet<String>,
    pub bound: Option<Bound>,
    pub protected: bool,
    /// Separate HP pool; zero means no substitute. Cleared on withdrawal or fainting.
    pub substitute_hp: u16,
    pub flinched: bool,
    pub locked_move: Option<LockedMove>,
    pub script_continuation: Option<ScriptContinuation>,
    pub next_move_power_boost: Option<NextMovePowerBoost>,
    pub last_move: Option<String>,
    pub consecutive_count: u8,
    /// Successful protection attempts share a streak across move names.
    pub protection_streak: u8,
    pub last_protection_turn: Option<u64>,
    pub crit_stage: u8,
    pub semi_invulnerable: bool,
    pub carrying: Option<ParticipantId>,
    pub carried_by: Option<ParticipantId>,
    pub recharging: bool,
    /// Free-form species name for scripts that care (Chatter, Relic Song).
    pub species: String,
    pub gender: Gender,
    /// Kilograms; 0 when unknown.
    pub weight: f32,
    /// Held item id; supported activation and removal rules run in the engine.
    pub item: Option<String>,
    /// Consumption history persists for the battle; transferring/removing an item does not add to it.
    pub consumed_items: BTreeSet<String>,
    pub ate_berry: bool,
    pub last_consumed_item: Option<String>,
    /// Most recently consumed held item, available once to Recycle. Stolen berries do not count.
    pub recyclable_item: Option<String>,
    pub(crate) item_used_turn: Option<u64>,
    /// Ability id. Supported mechanics query its active/suppressed state.
    pub ability: Option<String>,
    pub(crate) original_ability: Option<AbilityState>,
    pub(crate) ability_move_lock: Option<String>,
    pub ability_suppressed: bool,
    pub(crate) ability_order: u64,
    /// Optional host-defined rules belonging to the ability, gated by suppression and bypass.
    pub ability_rules: Vec<Rule>,
    /// Overrides the standard ability metadata for a host-defined ability.
    pub ability_bypassable: Option<bool>,
    /// HP, Attack, Defense, Sp. Attack, Sp. Defense, Speed.
    pub ivs: Option<[u8; 6]>,
    pub happiness: Option<u8>,
    /// Stat reduced by nature, used for berry flavor preferences. Raw stats remain host-provided.
    pub nature_down: Option<CombatStat>,
    /// Where it is while `semi_invulnerable`.
    pub hidden_kind: HiddenKind,
    pub conditions: Vec<Condition>,
    /// The battle turn it entered on (0 at the start).
    pub entered_turn: u64,
    pub(crate) last_position: Option<usize>,
    /// Whether it has finished its action this turn.
    pub acted: bool,
    /// HP lost to moves this turn.
    pub damage_taken: u16,
    /// Whether it lost HP for any reason this turn.
    pub hurt_this_turn: bool,
    /// Whether an actual stage increase/decrease occurred this turn; resets before turn-start hooks.
    pub stats_raised_this_turn: bool,
    pub stats_lowered_this_turn: bool,
    pub last_hit: Option<HitInfo>,
    pub moves_used: BTreeSet<String>,
    /// The move it last tried to use, whatever came of it.
    pub last_used: Option<String>,
    pub(crate) last_move_target: Option<(Side, usize)>,
    /// The actual type of the last attempted move, including dynamic type changes.
    pub last_move_type: Option<PokemonType>,
    pub last_move_failed: bool,
    /// How many turns in a row `streak_move` has been used without failing.
    pub streak_move: Option<String>,
    pub streak: u8,
}

impl Pokemon {
    pub fn new(name: impl Into<String>, moves: Vec<String>) -> Self {
        Self {
            illusion: None,
            species_id: 0,
            base_attack: None,
            form: None,
            forms: BTreeMap::new(),
            original_form: None,
            battle_base_form: None,
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
            added_type: None,
            moves,
            move_pp: BTreeMap::new(),
            transformation: None,
            temporary_moves: BTreeMap::new(),
            status: None,
            status_turns: 0,
            stages: BTreeMap::new(),
            pp_max: BTreeMap::new(),
            confused_turns: None,
            base_speed: None,
            flags: BTreeSet::new(),
            disguise: None,
            uncatchable: false,
            catch_rate: None,
            bound: None,
            protected: false,
            substitute_hp: 0,
            flinched: false,
            locked_move: None,
            script_continuation: None,
            next_move_power_boost: None,
            last_move: None,
            consecutive_count: 0,
            protection_streak: 0,
            last_protection_turn: None,
            crit_stage: 0,
            semi_invulnerable: false,
            carrying: None,
            carried_by: None,
            recharging: false,
            species: String::new(),
            gender: Gender::Genderless,
            weight: 0.0,
            item: None,
            consumed_items: BTreeSet::new(),
            ate_berry: false,
            last_consumed_item: None,
            recyclable_item: None,
            item_used_turn: None,
            ability: None,
            original_ability: None,
            ability_move_lock: None,
            ability_suppressed: false,
            ability_order: 0,
            ability_rules: Vec::new(),
            ability_bypassable: None,
            ivs: None,
            happiness: None,
            nature_down: None,
            hidden_kind: HiddenKind::Vanished,
            conditions: Vec::new(),
            entered_turn: 0,
            last_position: None,
            acted: false,
            damage_taken: 0,
            hurt_this_turn: false,
            stats_raised_this_turn: false,
            stats_lowered_this_turn: false,
            last_hit: None,
            moves_used: BTreeSet::new(),
            last_used: None,
            last_move_target: None,
            last_move_type: None,
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
    pub fn raw_stat(&self, stat: CombatStat) -> u16 {
        match stat {
            CombatStat::Attack => self.attack,
            CombatStat::Defense => self.defense,
            CombatStat::SpAttack => self.sp_attack,
            CombatStat::SpDefense => self.sp_defense,
            CombatStat::Speed => self.speed,
        }
    }
    pub fn effective_stat(&self, stat: CombatStat) -> f32 {
        let base = f32::from(self.raw_stat(stat));
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
    /// Lasts until replaced, like weather a battle starts with and no turn count.
    pub endless: bool,
}

/// The terrains moves, abilities, setup rules and battle scripts can start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainKind {
    Electric,
    Grassy,
    Misty,
    Psychic,
}

impl TerrainKind {
    /// The field condition that holds the terrain.
    pub const fn condition_name(self) -> &'static str {
        match self {
            Self::Electric => "electric_terrain",
            Self::Grassy => "grassy_terrain",
            Self::Misty => "misty_terrain",
            Self::Psychic => "psychic_terrain",
        }
    }
    /// The line shown when a battle starts on this terrain.
    pub(crate) const fn start_message(self) -> &'static str {
        match self {
            Self::Electric => msg("An electric current is running across the battlefield!"),
            Self::Grassy => msg("Grass is covering the battlefield!"),
            Self::Misty => msg("Mist is swirling about the battlefield!"),
            Self::Psychic => msg("The battlefield seems weird!"),
        }
    }
    pub(crate) fn condition(self, turns: Option<u8>) -> Condition {
        let rules = match self {
            Self::Electric => vec![
                Rule::BasePower {
                    move_type: Some(PokemonType::Electric),
                    factor: 5325.0 / 4096.0,
                    grounded_only: true,
                },
                Rule::BlockGroundedStatus {
                    statuses: vec![Status::Asleep],
                    confusion: false,
                },
            ],
            Self::Psychic => vec![
                Rule::BasePower {
                    move_type: Some(PokemonType::Psychic),
                    factor: 5325.0 / 4096.0,
                    grounded_only: true,
                },
                Rule::BlockPriority {
                    grounded_only: true,
                },
            ],
            Self::Misty => vec![
                Rule::IncomingBasePower {
                    moves: vec![],
                    move_type: Some(PokemonType::Dragon),
                    factor: 0.5,
                    grounded_only: true,
                },
                Rule::BlockGroundedStatus {
                    statuses: vec![],
                    confusion: true,
                },
            ],
            Self::Grassy => vec![
                Rule::BasePower {
                    move_type: Some(PokemonType::Grass),
                    factor: 5325.0 / 4096.0,
                    grounded_only: true,
                },
                Rule::IncomingBasePower {
                    moves: vec!["earthquake".into(), "bulldoze".into(), "magnitude".into()],
                    move_type: None,
                    factor: 0.5,
                    grounded_only: true,
                },
                Rule::HealEachTurn {
                    fraction: 1.0 / 16.0,
                    message: None,
                    grounded_only: true,
                },
            ],
        };
        Condition {
            name: self.condition_name().into(),
            value: 1,
            turns_left: turns,
            rules,
            group: Some("terrain".into()),
            end_message: None,
        }
    }
}

/// A value the host describes the battle's surroundings with (time of day, indoors, ...).
/// Scripts read them from `ctx.context`.
#[derive(Clone, Debug, PartialEq)]
pub enum ContextValue {
    Flag(bool),
    Number(f64),
    Text(String),
}

/// A line of battle text. `key` is an English template whose `{0}`, `{1}`... placeholders
/// stand for `args`, so a host can use it as a translation key (for example a gettext msgid,
/// with `context()` as its msgctxt) and present each argument its own way. Messages compare
/// equal when their English text does.
#[derive(Clone, Debug)]
pub struct Message {
    pub key: std::borrow::Cow<'static, str>,
    pub args: Vec<MessageArg>,
    text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageArg {
    /// A battle participant, with the name and gender it currently shows (an Illusion's).
    Pokemon {
        id: ParticipantId,
        name: String,
        gender: Gender,
    },
    /// A move, with its catalog name.
    Move {
        id: String,
        name: String,
    },
    /// Text that is itself translatable, such as a stat or ability name.
    Text(std::borrow::Cow<'static, str>),
    Number(i64),
}

impl MessageArg {
    fn english(&self) -> String {
        match self {
            Self::Pokemon { name, .. } | Self::Move { name, .. } => name.clone(),
            Self::Text(text) => text.to_string(),
            Self::Number(number) => number.to_string(),
        }
    }
}

impl Message {
    pub fn new(
        key: impl Into<std::borrow::Cow<'static, str>>,
        args: impl IntoIterator<Item = MessageArg>,
    ) -> Self {
        let key = key.into();
        let args: Vec<_> = args.into_iter().collect();
        let mut text = key.to_string();
        for (index, arg) in args.iter().enumerate() {
            text = text.replace(&format!("{{{index}}}"), &arg.english());
        }
        Self { key, args, text }
    }
    /// The English text.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// The English text, like [`Message::text`].
    pub fn as_str(&self) -> &str {
        &self.text
    }
    /// The gender of the first Pokémon argument, for languages whose wording depends on it.
    pub fn context(&self) -> Option<Gender> {
        self.args.iter().find_map(|arg| match arg {
            MessageArg::Pokemon { gender, .. } => Some(*gender),
            _ => None,
        })
    }
}

/// Marks a message key chosen before it is passed on, for scripts/extract_messages.py.
pub(crate) const fn msg(key: &'static str) -> &'static str {
    key
}

/// A message event.
pub(crate) fn say(
    key: impl Into<std::borrow::Cow<'static, str>>,
    args: impl IntoIterator<Item = MessageArg>,
) -> BattleEvent {
    BattleEvent::Message(Message::new(key, args))
}

/// A move as a message argument.
pub(crate) fn move_arg(spec: &crate::moves::MoveSpec) -> MessageArg {
    MessageArg::Move {
        id: spec.id.clone(),
        name: spec.name.clone(),
    }
}

impl From<&'static str> for Message {
    fn from(key: &'static str) -> Self {
        Self::new(key, [])
    }
}

impl std::ops::Deref for Message {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}
impl Eq for Message {}
impl PartialEq<str> for Message {
    fn eq(&self, other: &str) -> bool {
        self.text == other
    }
}
impl PartialEq<&str> for Message {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BattleEvent {
    /// A move is used, before its effects; `targets` are the Pokémon it is aimed at.
    MoveUsed {
        actor: ParticipantId,
        move_id: String,
        targets: Vec<ParticipantId>,
    },
    /// A damaging hit lands, just before the HP (or substitute) change. `hit` counts the hits
    /// on `target` within this use of the move, from 1; `effectiveness_percent` is 100 for
    /// neutral damage, 200 for super effective and so on.
    Hit {
        actor: ParticipantId,
        target: ParticipantId,
        hit: u8,
        effectiveness_percent: u16,
        critical: bool,
    },
    /// An attack missed `target`, or `target` avoided it while hidden.
    Miss {
        actor: ParticipantId,
        target: ParticipantId,
    },
    /// An item from the bag is used on `target`, before its effects. `consumed` is false when
    /// the item was not used up (a ball the target refused), so the host keeps it.
    ItemUsed {
        actor: ParticipantId,
        item: String,
        target: ParticipantId,
        consumed: bool,
    },
    /// A ball thrown at `target`: `shakes` is how many breakout checks it held (up to 4, or 1
    /// for a critical capture), and `caught` whether it held them all.
    Capture {
        actor: ParticipantId,
        target: ParticipantId,
        ball: String,
        shakes: u8,
        critical: bool,
        caught: bool,
    },
    /// `target` cannot be caught; the ball is not used up.
    CaptureRefused {
        actor: ParticipantId,
        target: ParticipantId,
        ball: String,
    },
    /// Something a battle script wants the host to present, such as a cutscene: `key` and
    /// `args` are the script's own.
    Script {
        key: String,
        args: Vec<MessageArg>,
    },
    /// The first turn of a two-turn move.
    Charging {
        actor: ParticipantId,
        move_id: String,
    },
    /// `target` became semi-invulnerable (`place` is where, as Fly or Dig hide) or reappeared.
    Hidden {
        target: ParticipantId,
        place: Option<HiddenKind>,
    },
    PositionsSwapped {
        a: ParticipantId,
        b: ParticipantId,
    },
    /// HP reassignment, without damage or healing mechanics (for example Pain Split).
    HpSet {
        target: ParticipantId,
        hp: u16,
    },
    Substitute {
        target: ParticipantId,
        amount: u16,
        hp: u16,
    },
    Switched {
        from: ParticipantId,
        to: ParticipantId,
        position: usize,
    },
    Message(Message),
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
    AppearanceChanged {
        target: ParticipantId,
        illusion: Option<PokemonAppearance>,
    },
    FormChanged {
        target: ParticipantId,
        form: Option<String>,
    },
    Weather(Option<Weather>),
    Fainted(ParticipantId),
    EscapeAttempt {
        actor: ParticipantId,
        success: bool,
    },
    End {
        winner: Option<Side>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Attempt to leave a wild battle with the entire allied party.
    Escape {
        id: u32,
    },
    /// Resume an otherwise automatic recharge or scripted continuation.
    Continue {
        id: u32,
    },
    Switch {
        id: u32,
        to: ParticipantId,
    },
    UseMove {
        id: u32,
        move_id: String,
        target: ParticipantId,
    },
    /// Use an item from the side's bag on `target`.
    UseItem {
        id: u32,
        item: String,
        target: ParticipantId,
        /// The move it is used on, for items like Ether.
        move_id: Option<String>,
    },
    /// Take back the last choice made for this side this turn, and be asked for it again.
    /// Offered in action prompts once a choice for the same side was answered this turn;
    /// always listed last.
    Undo {
        id: u32,
    },
}

impl Choice {
    pub const fn id(&self) -> u32 {
        match self {
            Self::UseMove { id, .. }
            | Self::Switch { id, .. }
            | Self::Escape { id }
            | Self::Continue { id }
            | Self::UseItem { id, .. }
            | Self::Undo { id } => *id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptReason {
    Action,
    Replacement,
    Pivot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub reason: PromptReason,
    pub id: u64,
    pub actor: ParticipantId,
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvanceStatus {
    Awaiting(Prompt),
    End {
        winner: Option<Side>,
    },
    Escaped {
        side: Side,
    },
    /// A wild Pokémon was caught; the battle is over.
    Captured {
        target: ParticipantId,
    },
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
