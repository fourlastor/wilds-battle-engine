use crate::model::{
    ActionSelection, AdvanceResult, AdvanceStatus, BattleError, BattleEvent, Bound, Choice,
    CombatStat, Condition, ContinuationTarget, HitInfo, LockedMove, ParticipantId, Pokemon,
    PokemonType, Prompt, PromptReason, Rule, Scope, Side, Stat, Status, Weather, WeatherKind,
};
use crate::moves::{Accuracy, Category, Effect, MoveCatalog, MoveSpec, ScriptHook, Target};
use crate::rng::{BattleRng, SeededRng};
use crate::type_chart;

pub struct Battle {
    pub(crate) catalog: MoveCatalog,
    pub(crate) sides: [Vec<Pokemon>; 2],
    pub(crate) active: [Vec<usize>; 2],
    pending_events: Vec<BattleEvent>,
    pub(crate) switching_target: Option<ParticipantId>,
    pub(crate) pivot: Option<ParticipantId>,
    pub(crate) baton_pass: Option<(ParticipantId, Vec<String>)>,
    pub(crate) forced_switches: Vec<(ParticipantId, ParticipantId)>,
    pub(crate) weather: Option<Weather>,
    pub(crate) rng: Box<dyn BattleRng>,
    pending: Option<Prompt>,
    selections: Vec<(ParticipantId, SelectedAction)>,
    turn_queue: std::collections::VecDeque<QueuedAction>,
    completed_slots: Vec<(Side, usize)>,
    resolving_turn: bool,
    entry_abilities_started: bool,
    next_prompt_id: u64,
    pub(crate) winner: Option<Option<Side>>,
    pub(crate) turn: u64,
    pub(crate) active_power_multiplier: f32,
    pub(crate) active_gem: Option<(ParticipantId, PokemonType)>,
    pub(crate) pending_knockouts: Vec<ParticipantId>,
    pub(crate) pending_faints: Vec<(ParticipantId, crate::model::AbilityState)>,
    pub(crate) item_hits: Vec<(ParticipantId, ParticipantId, HitMeta)>,
    pub(crate) hit_reactions: Vec<HitReaction>,
    pub(crate) reaction_hit: Option<HitInfo>,
    pub(crate) ignored_ability: Option<ParticipantId>,
    pub(crate) ability_bypass: bool,
    pub(crate) reflecting: bool,
    pub(crate) prankster_boosted: bool,
    pub(crate) substitute_hits: std::collections::BTreeSet<ParticipantId>,
    pub(crate) side_conditions: [Vec<Condition>; 2],
    pub(crate) field_conditions: Vec<Condition>,
    pub(crate) delayed_heals: Vec<crate::slots::DelayedHeal>,
    pub(crate) delayed_attacks: Vec<crate::slots::DelayedAttack>,
    pub(crate) replacement_heals: Vec<crate::slots::ReplacementHeal>,
    pub(crate) last_faint_turn: [Option<u64>; 2],
    pub(crate) payout: [u32; 2],
    pub(crate) payout_multiplier: [u8; 2],
    pub(crate) environment: Option<String>,
    /// The move each Pokémon chose this turn.
    pub(crate) turn_choices: Vec<(ParticipantId, String)>,
    /// Move users still waiting after the current action, in queue order.
    pub(crate) pending_actors: Vec<ParticipantId>,
    /// Pending actors promoted by moves or reactions after the current action.
    pub(crate) hurry: Vec<ParticipantId>,
    /// Whether the move being used right now failed, missed or had no effect.
    pub(crate) move_failed: bool,
    call_depth: u8,
    snatching: bool,
    pub(crate) last_battle_move: Option<String>,
    pub(crate) current_move: Option<String>,
}

#[derive(Clone)]
pub(crate) struct HitReaction {
    pub(crate) owner: ParticipantId,
    pub(crate) contact_only: bool,
    pub(crate) move_id: String,
}

/// What a hit was, for the Pokémon it lands on.
#[derive(Clone)]
pub(crate) struct HitMeta {
    move_id: String,
    pub(crate) category: Category,
    pub(crate) move_type: PokemonType,
    contact: bool,
    bypass_substitute: bool,
    pub(crate) defer_items: bool,
    pub(crate) effectiveness: Option<f32>,
    critical: bool,
}
impl HitMeta {
    pub(crate) fn of(spec: &MoveSpec, category: Category, move_type: PokemonType) -> Self {
        Self {
            move_id: spec.id.clone(),
            category,
            move_type,
            contact: spec.has_flag("contact"),
            bypass_substitute: spec.has_flag("sound") || spec.has_flag("bypass_substitute"),
            defer_items: spec.has_flag("item_effect_on_hit"),
            effectiveness: None,
            critical: false,
        }
    }
    pub(crate) fn with_contact(mut self, contact: Option<bool>) -> Self {
        if let Some(contact) = contact {
            self.contact = contact;
        }
        self
    }
    pub(crate) fn with_roll(mut self, roll: &Roll) -> Self {
        self.effectiveness = Some(roll.effectiveness);
        self.critical = roll.critical;
        self
    }
}

/// Everything the damage formula needs to know about one hit.
pub(crate) struct Attack<'a> {
    pub(crate) move_id: &'a str,
    pub(crate) power: u16,
    pub(crate) high_crit: bool,
    pub(crate) always_crit: bool,
    pub(crate) force_stab: bool,
    pub(crate) hits_airborne: bool,
    pub(crate) no_gem: bool,
    pub(crate) sound: bool,
    pub(crate) bypass_substitute: bool,
    pub(crate) bullet: bool,
    pub(crate) contact: bool,
    pub(crate) consecutive_boost: u8,
    pub(crate) typeless: bool,
    pub(crate) move_type: PokemonType,
    pub(crate) also_type: Option<PokemonType>,
    pub(crate) effective: &'a [(PokemonType, f32)],
    pub(crate) category: Category,
    /// Whose attacking stat is used.
    pub(crate) attack_from: ParticipantId,
    pub(crate) attack_stat: Option<CombatStat>,
    pub(crate) defense_stat: Option<CombatStat>,
    pub(crate) ignore_stages: bool,
}

pub(crate) struct Roll {
    pub(crate) amount: u16,
    pub(crate) critical: bool,
    pub(crate) effectiveness: f32,
}

#[derive(Clone)]
enum SelectedAction {
    Move(MoveAction),
    Recharge,
    Switch(ParticipantId),
}

#[derive(Clone)]
struct MoveAction {
    forced: bool,
    move_id: String,
    target: ParticipantId,
    target_position: usize,
}

struct QueuedAction {
    actor: ParticipantId,
    position: usize,
    action: SelectedAction,
    priority: i8,
    fractional_priority: i8,
    speed: i32,
    tie: u64,
    custap: bool,
    quick_draw: bool,
    promoted: bool,
    postponed: bool,
}
impl QueuedAction {
    fn order(a: &Self, b: &Self) -> std::cmp::Ordering {
        b.priority
            .cmp(&a.priority)
            .then(b.fractional_priority.cmp(&a.fractional_priority))
            .then(b.speed.cmp(&a.speed))
            .then(b.tie.cmp(&a.tie))
    }
}

impl Battle {
    pub fn new(
        seed: u64,
        sides: [Vec<Pokemon>; 2],
        catalog: MoveCatalog,
    ) -> Result<Self, BattleError> {
        Self::with_rng(SeededRng::new(seed), sides, catalog)
    }

    pub fn with_rng(
        rng: impl BattleRng + 'static,
        mut sides: [Vec<Pokemon>; 2],
        catalog: MoveCatalog,
    ) -> Result<Self, BattleError> {
        if sides.iter().any(Vec::is_empty) {
            return Err(BattleError::InvalidSetup(
                "each side needs a participant".into(),
            ));
        }
        for pokemon in sides.iter().flatten() {
            if pokemon.hp > pokemon.max_hp || pokemon.max_hp == 0 {
                return Err(BattleError::InvalidSetup("invalid HP".into()));
            }
            for (id, form) in &pokemon.forms {
                if id.is_empty()
                    || id.len() > 40
                    || form.types.is_empty()
                    || form.types.len() > 2
                    || !form.weight.is_finite()
                    || form.weight < 0.0
                {
                    return Err(BattleError::InvalidSetup(format!("invalid form {id}")));
                }
            }
            if pokemon.moves.is_empty() {
                return Err(BattleError::InvalidSetup(format!(
                    "{} has no moves",
                    pokemon.name
                )));
            }
            for id in &pokemon.moves {
                let Some(spec) = catalog.get(id) else {
                    return Err(BattleError::InvalidSetup(format!("unknown move {id}")));
                };
                if spec.auto_only {
                    return Err(BattleError::InvalidSetup(format!("{id} cannot be learned")));
                }
            }
        }
        for pokemon in sides.iter_mut().flatten() {
            pokemon.battle_base_form =
                Some((pokemon.form.clone(), crate::model::PokemonForm::of(pokemon)));
            for id in &pokemon.moves {
                let spec = catalog.get(id).expect("validated catalog");
                pokemon.move_pp.entry(id.clone()).or_insert(spec.pp);
            }
        }
        let active = std::array::from_fn(|side| (0..sides[side].len()).collect());
        Ok(Self {
            catalog,
            active,
            pending_events: Vec::new(),
            forced_switches: Vec::new(),
            pivot: None,
            baton_pass: None,
            switching_target: None,
            sides,
            weather: None,
            rng: Box::new(rng),
            pending: None,
            selections: Vec::new(),
            turn_queue: std::collections::VecDeque::new(),
            completed_slots: Vec::new(),
            resolving_turn: false,
            entry_abilities_started: false,
            next_prompt_id: 1,
            winner: None,
            turn: 0,
            active_power_multiplier: 1.0,
            active_gem: None,
            item_hits: Vec::new(),
            pending_knockouts: Vec::new(),
            pending_faints: Vec::new(),
            hit_reactions: Vec::new(),
            reaction_hit: None,
            ignored_ability: None,
            ability_bypass: false,
            reflecting: false,
            prankster_boosted: false,
            substitute_hits: std::collections::BTreeSet::new(),
            side_conditions: [Vec::new(), Vec::new()],
            field_conditions: Vec::new(),
            delayed_heals: Vec::new(),
            delayed_attacks: Vec::new(),
            replacement_heals: Vec::new(),
            last_faint_turn: [None, None],
            payout: [0, 0],
            payout_multiplier: [1, 1],
            environment: None,
            turn_choices: Vec::new(),
            pending_actors: Vec::new(),
            hurry: Vec::new(),
            move_failed: false,
            call_depth: 0,
            snatching: false,
            last_battle_move: None,
            current_move: None,
        })
    }
    pub(crate) fn has_started(&self) -> bool {
        self.entry_abilities_started || self.next_prompt_id != 1 || self.winner.is_some()
    }
    pub fn participants(&self, side: Side) -> &[Pokemon] {
        &self.sides[side.index()]
    }
    pub fn weather(&self) -> Option<&Weather> {
        self.weather.as_ref()
    }
    pub fn turn(&self) -> u64 {
        self.turn
    }
    pub fn catalog(&self) -> &MoveCatalog {
        &self.catalog
    }
    /// Marks and timed effects on one side, or on the field.
    pub fn side_conditions(&self, side: Side) -> &[Condition] {
        self.conditions(Scope::Side(side))
    }
    pub fn field_conditions(&self) -> &[Condition] {
        self.conditions(Scope::Field)
    }
    /// Money a side's moves have earned so far, including its payout multiplier.
    pub fn payout(&self, side: Side) -> u32 {
        self.payout[side.index()].saturating_mul(u32::from(self.payout_multiplier(side)))
    }
    /// Apply this multiplier to host-owned trainer prize money after the battle.
    pub fn payout_multiplier(&self, side: Side) -> u8 {
        self.payout_multiplier[side.index()]
    }
    /// Where the battle takes place, for moves that depend on it (Secret Power).
    pub fn environment(&self) -> Option<&str> {
        self.environment.as_deref()
    }
    pub fn set_environment(&mut self, environment: Option<String>) {
        self.environment = environment;
    }
    /// Whether an effect keeps this Pokémon from leaving the battle.
    pub fn trapped(&self, id: ParticipantId) -> bool {
        if self.get(id).carrying.is_some() || self.get(id).carried_by.is_some() {
            return true;
        }
        if !self.is_active(id)
            || self.get(id).hp == 0
            || self.types_of(id).contains(&PokemonType::Ghost)
        {
            return false;
        }
        let shell = self
            .get(id)
            .item
            .as_deref()
            .is_some_and(|item| crate::items::item_key(item) == "shedshell")
            && self.item_active(id);
        !shell && (self.is_trapped(id) || self.get(id).bound.is_some() || self.ability_traps(id))
    }
    pub(crate) fn get(&self, id: ParticipantId) -> &Pokemon {
        &self.sides[id.side.index()][id.index]
    }
    /// Field slots are ordered from each trainer's perspective. Opposing rows face each other.
    /// Fainted occupants retain their slot, so neighbors never shift when a Pokémon faints.
    pub fn adjacent(&self, a: ParticipantId, b: ParticipantId) -> bool {
        if a == b {
            return false;
        }
        let (Some(a_pos), Some(b_pos)) = (self.position(a), self.position(b)) else {
            return false;
        };
        let width = self.active.iter().map(Vec::len).max().unwrap_or(1);
        if a.side == b.side {
            a_pos.abs_diff(b_pos) == 1
        } else if width <= 2 {
            true
        } else {
            a_pos.abs_diff(width - 1 - b_pos) <= 1
        }
    }
    pub(crate) fn get_mut(&mut self, id: ParticipantId) -> &mut Pokemon {
        &mut self.sides[id.side.index()][id.index]
    }
    pub(crate) fn actors(&self) -> Vec<ParticipantId> {
        Side::ALL
            .into_iter()
            .flat_map(|side| {
                self.active[side.index()]
                    .iter()
                    .copied()
                    .filter(move |&index| self.sides[side.index()][index].hp > 0)
                    .map(move |index| ParticipantId { side, index })
            })
            .collect()
    }
    pub(crate) fn move_target_kind(&self, actor: ParticipantId, spec: &MoveSpec) -> Target {
        if spec
            .target_user_unless_type
            .is_some_and(|kind| !self.types_of(actor).contains(&kind))
        {
            Target::User
        } else {
            spec.target
        }
    }

    fn legal_choices(&self, actor: ParticipantId) -> Vec<Choice> {
        let pokemon = self.get(actor);
        let move_ids: Vec<&str> = match &pokemon.locked_move {
            Some(lock) => vec![lock.move_id.as_str()],
            None => pokemon.moves.iter().map(String::as_str).collect(),
        };
        let mut result = Vec::new();
        for move_id in move_ids {
            if pokemon.locked_move.is_none() && pokemon.move_pp.get(move_id) == Some(&0) {
                continue;
            }
            let spec = self.catalog.get(move_id).expect("validated catalog");
            if self.move_block(actor, spec).is_some()
                || (pokemon.locked_move.is_none()
                    && move_id != "struggle"
                    && pokemon.last_used.as_deref() == Some(move_id)
                    && self.rules(actor).any(|rule| matches!(rule, Rule::NoRepeat)))
            {
                continue;
            }
            let targets: Vec<_> = if let Some(lock) = &pokemon.locked_move
                && lock.charging
            {
                vec![lock.target]
            } else {
                let allies = |with_user: bool| -> Vec<ParticipantId> {
                    self.active[actor.side.index()]
                        .iter()
                        .copied()
                        .filter(|&index| self.sides[actor.side.index()][index].hp > 0)
                        .filter(|&index| with_user || index != actor.index)
                        .map(|index| ParticipantId {
                            side: actor.side,
                            index,
                        })
                        .collect()
                };
                match self.move_target_kind(actor, spec) {
                    Target::User | Target::AllAllies | Target::UserAndAllies => vec![actor],
                    Target::Field
                    | Target::AllOthers
                    | Target::RandomOpponent
                    | Target::AllOpponents
                    | Target::All => self
                        .actors()
                        .into_iter()
                        .find(|id| id.side != actor.side)
                        .into_iter()
                        .collect(),
                    Target::Selected => self
                        .actors()
                        .into_iter()
                        .filter(|id| id.side != actor.side)
                        .collect(),
                    Target::Ally => {
                        let targets: Vec<_> = allies(false)
                            .into_iter()
                            .filter(|&id| self.adjacent(actor, id))
                            .collect();
                        // An ally move can still be attempted without an ally; it fails at execution.
                        if targets.is_empty() {
                            vec![actor]
                        } else {
                            targets
                        }
                    }
                    Target::UserOrAlly => allies(true),
                    Target::AnyOther => self
                        .actors()
                        .into_iter()
                        .filter(|id| *id != actor)
                        .collect(),
                }
            };
            for target in targets {
                result.push(Choice::UseMove {
                    id: result.len() as u32,
                    move_id: move_id.to_owned(),
                    target,
                });
            }
        }
        if !self.trapped(actor) && pokemon.locked_move.is_none() {
            let reserved: Vec<_> = self
                .selections
                .iter()
                .filter_map(|(_, action)| {
                    if let SelectedAction::Switch(to) = action {
                        Some(*to)
                    } else {
                        None
                    }
                })
                .collect();
            let switches: Vec<_> = self
                .bench(actor.side)
                .into_iter()
                .filter(|to| !reserved.contains(to))
                .collect();
            if result.is_empty()
                && !switches.is_empty()
                && let Some(target) = self.actors().into_iter().find(|id| id.side != actor.side)
            {
                result.push(Choice::UseMove {
                    id: 0,
                    move_id: "struggle".into(),
                    target,
                });
            }
            for to in switches {
                result.push(Choice::Switch {
                    id: result.len() as u32,
                    to,
                });
            }
        }
        result
    }
    pub fn advance(&mut self) -> Result<AdvanceResult, BattleError> {
        if let Some(winner) = self.winner {
            return Ok(AdvanceResult {
                events: Vec::new(),
                status: AdvanceStatus::End { winner },
            });
        }
        if let Some(prompt) = &self.pending {
            return Ok(AdvanceResult {
                events: Vec::new(),
                status: AdvanceStatus::Awaiting(prompt.clone()),
            });
        }
        let mut events = std::mem::take(&mut self.pending_events);
        if !self.entry_abilities_started {
            self.entry_abilities_started = true;
            let mut entrants = self.actors();
            entrants.sort_by_key(|&id| std::cmp::Reverse(self.action_speed(id)));
            for who in entrants {
                self.ability_on_entry(who, &mut events);
            }
        }
        self.resolve_outcome(&mut events);
        if let Some(winner) = self.winner {
            return Ok(AdvanceResult {
                events,
                status: AdvanceStatus::End { winner },
            });
        }
        if self.resolving_turn {
            events.extend(self.resolve_turn()?);
            let next = self.advance()?;
            events.extend(next.events);
            return Ok(AdvanceResult {
                events,
                status: next.status,
            });
        }
        if let Some((actor, choices)) = self.replacement() {
            let prompt = self.make_prompt(actor, choices, PromptReason::Replacement);
            return Ok(AdvanceResult {
                events,
                status: AdvanceStatus::Awaiting(prompt),
            });
        }
        if self.selections.is_empty() {
            self.update_berries(&mut events);
        }
        let actors = self.actors();
        let selected: Vec<_> = self.selections.iter().map(|(actor, _)| *actor).collect();
        for actor in actors {
            if selected.contains(&actor) {
                continue;
            }
            if self.get(actor).recharging {
                self.selections.push((actor, SelectedAction::Recharge));
                continue;
            }
            if let Some(continuation) = self.get(actor).script_continuation.clone() {
                // A RandomOpponent continuation is re-aimed when the move executes.
                self.selections.push((
                    actor,
                    SelectedAction::Move(MoveAction {
                        forced: true,
                        move_id: continuation.move_id,
                        target: continuation.target,
                        target_position: self.target_position(continuation.target),
                    }),
                ));
                continue;
            }
            let choices = self.legal_choices(actor);
            if choices.is_empty() {
                let target = self
                    .actors()
                    .into_iter()
                    .find(|id| id.side != actor.side)
                    .ok_or_else(|| BattleError::InvalidSetup("Struggle has no target".into()))?;
                self.selections.push((
                    actor,
                    SelectedAction::Move(MoveAction {
                        forced: false,
                        move_id: "struggle".into(),
                        target,
                        target_position: self.target_position(target),
                    }),
                ));
                continue;
            }
            if self
                .get(actor)
                .locked_move
                .as_ref()
                .is_some_and(|lock| lock.charging)
            {
                self.selections
                    .push((actor, self.selected_action(actor, &choices[0])));
                continue;
            }
            let prompt = self.make_prompt(actor, choices, PromptReason::Action);
            return Ok(AdvanceResult {
                events,
                status: AdvanceStatus::Awaiting(prompt),
            });
        }
        events.extend(self.resolve_turn()?);
        let status = if let Some(winner) = self.winner {
            AdvanceStatus::End { winner }
        } else {
            // Returning the next prompt together with turn events lets hosts animate first.
            let next = self.advance()?;
            events.extend(next.events);
            self.pending
                .as_ref()
                .map(|p| AdvanceStatus::Awaiting(p.clone()))
                .unwrap_or(next.status)
        };
        Ok(AdvanceResult { events, status })
    }
    pub fn set_response(&mut self, selection: ActionSelection) -> Result<(), BattleError> {
        let prompt = self
            .pending
            .as_ref()
            .ok_or_else(|| BattleError::InvalidResponse("no pending prompt".into()))?;
        if selection.prompt_id != prompt.id {
            return Err(BattleError::InvalidResponse("stale prompt id".into()));
        }
        let choice = prompt
            .choices
            .iter()
            .find(|choice| choice.id() == selection.choice_id)
            .ok_or_else(|| BattleError::InvalidResponse("choice is not legal".into()))?;
        let actor = prompt.actor;
        let reason = prompt.reason;
        let action = self.selected_action(actor, choice);
        self.pending = None;
        if reason == PromptReason::Action {
            self.selections.push((actor, action));
        } else if let SelectedAction::Switch(to) = action {
            let mut events = Vec::new();
            self.switch_member(actor, to, &mut events);
            self.pending_events.extend(events);
        }
        Ok(())
    }
    fn make_prompt(
        &mut self,
        actor: ParticipantId,
        choices: Vec<Choice>,
        reason: PromptReason,
    ) -> Prompt {
        let prompt = Prompt {
            id: self.next_prompt_id,
            actor,
            choices,
            reason,
        };
        self.next_prompt_id += 1;
        self.pending = Some(prompt.clone());
        prompt
    }
    fn target_position(&self, target: ParticipantId) -> usize {
        self.position(target)
            .or(self.get(target).last_position)
            .unwrap_or(0)
    }
    fn selected_action(&self, actor: ParticipantId, choice: &Choice) -> SelectedAction {
        match choice {
            Choice::UseMove {
                move_id, target, ..
            } => SelectedAction::Move(MoveAction {
                forced: self
                    .get(actor)
                    .locked_move
                    .as_ref()
                    .is_some_and(|lock| lock.charging),
                move_id: move_id.clone(),
                target: *target,
                target_position: self.target_position(*target),
            }),
            Choice::Switch { to, .. } => SelectedAction::Switch(*to),
        }
    }
    fn resolve_turn(&mut self) -> Result<Vec<BattleEvent>, BattleError> {
        let mut events = Vec::new();
        if !self.resolving_turn {
            self.begin_turn(&mut events)?;
        }
        self.resume_turn(&mut events)?;
        Ok(events)
    }

    fn begin_turn(&mut self, events: &mut Vec<BattleEvent>) -> Result<(), BattleError> {
        self.turn += 1;
        self.resolving_turn = true;
        self.completed_slots.clear();
        let mut ordered: Vec<_> = std::mem::take(&mut self.selections)
            .into_iter()
            .map(|(actor, choice)| {
                let priority = match &choice {
                    SelectedAction::Move(choice) => self.effective_move_priority(
                        actor,
                        self.catalog.get(&choice.move_id).expect("validated choice"),
                    ),
                    SelectedAction::Recharge => 0,
                    SelectedAction::Switch(_) => 6,
                };
                let custap = matches!(choice, SelectedAction::Move(_))
                    && self.custap_priority(actor, events);
                let quick_draw = matches!(&choice, SelectedAction::Move(choice) if self.catalog.get(&choice.move_id).is_some_and(|spec| spec.category != Category::Status))
                    && self.has_ability(actor, "quickdraw") && self.rng.chance(0.3);
                if quick_draw { events.push(BattleEvent::Message(format!("{}'s Quick Draw activated!", self.get(actor).name))); }
                let speed = self.action_speed(actor);
                let fractional_priority = if custap || quick_draw {
                    1
                } else if !matches!(choice, SelectedAction::Switch(_))
                    && self.has_ability(actor, "stall")
                {
                    -1
                } else {
                    0
                };
                QueuedAction {
                    actor,
                    position: self.position(actor).expect("active choice"),
                    action: choice,
                    priority,
                    fractional_priority,
                    speed,
                    tie: self.rng.next_u64(),
                    custap,
                    quick_draw,
                    promoted: false,
                    postponed: false,
                }
            })
            .collect();
        ordered.sort_by(QueuedAction::order);
        for pokemon in self.sides.iter_mut().flatten() {
            pokemon.acted = false;
            pokemon.damage_taken = 0;
            pokemon.hurt_this_turn = false;
            pokemon.stats_raised_this_turn = false;
            pokemon.stats_lowered_this_turn = false;
            pokemon.last_hit = None;
        }
        self.turn_choices = ordered
            .iter()
            .filter_map(|entry| match &entry.action {
                SelectedAction::Move(choice) => Some((entry.actor, choice.move_id.clone())),
                SelectedAction::Recharge | SelectedAction::Switch(_) => None,
            })
            .collect();
        self.hurry.clear();
        // Moves that get ready before anyone acts (Focus Punch, Beak Blast).
        for entry in &ordered {
            let actor = entry.actor;
            let SelectedAction::Move(choice) = &entry.action else {
                continue;
            };
            let spec = self.catalog.get(&choice.move_id).expect("validated choice");
            if spec.on_turn_start.is_some()
                && self.get(actor).hp > 0
                && self.move_block(actor, spec).is_none()
            {
                let spec = spec.clone();
                self.execute_script(actor, choice.target, &spec, events, ScriptHook::TurnStart)?;
            }
        }
        // Turn-start hooks can establish conditional priority without consuming an action.
        // Preserve the speed and tie rolls; priority changes never reroll ties.
        for entry in &mut ordered {
            if let SelectedAction::Move(choice) = &entry.action {
                entry.priority = self.effective_move_priority(
                    entry.actor,
                    self.catalog.get(&choice.move_id).expect("validated choice"),
                );
            }
        }
        ordered.sort_by(QueuedAction::order);
        self.turn_queue = ordered.into();
        Ok(())
    }

    pub(crate) fn swap_positions(
        &mut self,
        a: ParticipantId,
        b: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if a == b || a.side != b.side || self.get(a).hp == 0 || self.get(b).hp == 0 {
            return false;
        }
        let (Some(pa), Some(pb)) = (self.position(a), self.position(b)) else {
            return false;
        };
        self.active[a.side.index()].swap(pa, pb);
        self.get_mut(a).last_position = Some(pb);
        self.get_mut(b).last_position = Some(pa);
        for entry in &mut self.turn_queue {
            if entry.actor == a {
                entry.position = pb;
            }
            if entry.actor == b {
                entry.position = pa;
            }
        }
        for (side, pos) in &mut self.completed_slots {
            if *side == a.side {
                if *pos == pa {
                    *pos = pb;
                } else if *pos == pb {
                    *pos = pa;
                }
            }
        }
        // Drain recovery follows the vacated slot, as it does on a party switch.
        for condition in self
            .sides
            .iter_mut()
            .flatten()
            .flat_map(|p| &mut p.conditions)
            .chain(self.side_conditions.iter_mut().flatten())
            .chain(&mut self.field_conditions)
        {
            for rule in &mut condition.rules {
                if let Rule::DrainEachTurn { to, .. } = rule {
                    if *to == a {
                        *to = b;
                    } else if *to == b {
                        *to = a;
                    }
                }
            }
        }
        self.heal_replacement(a, events);
        self.heal_replacement(b, events);
        events.push(BattleEvent::PositionsSwapped { a, b });
        events.push(BattleEvent::Message(format!(
            "{} switched places with {}!",
            self.get(a).name,
            self.get(b).name
        )));
        true
    }

    pub(crate) fn delay_action(&mut self, who: ParticipantId) -> bool {
        let Some(entry) = self
            .turn_queue
            .iter_mut()
            .find(|entry| entry.actor == who && matches!(entry.action, SelectedAction::Move(_)))
        else {
            return false;
        };
        entry.promoted = false;
        entry.postponed = true;
        true
    }

    /// Refresh speed after each action (and a paused pivot), without rerolling ties or
    /// undoing moves that explicitly promoted a pending action.
    fn refresh_pending_speed(&mut self) {
        let mut queue = std::mem::take(&mut self.turn_queue);
        for entry in &mut queue {
            if let SelectedAction::Move(choice) = &entry.action {
                entry.priority = self.effective_move_priority(
                    entry.actor,
                    self.catalog.get(&choice.move_id).expect("queued move"),
                );
            }
            entry.fractional_priority = if entry.custap || entry.quick_draw {
                1
            } else if !matches!(entry.action, SelectedAction::Switch(_))
                && self.has_ability(entry.actor, "stall")
            {
                -1
            } else {
                0
            };
            entry.speed = self.action_speed(entry.actor);
        }
        queue.make_contiguous().sort_by(|a, b| {
            b.promoted.cmp(&a.promoted).then_with(|| {
                if a.promoted && b.promoted {
                    std::cmp::Ordering::Equal
                } else {
                    a.postponed
                        .cmp(&b.postponed)
                        .then_with(|| QueuedAction::order(a, b))
                }
            })
        });
        self.turn_queue = queue;
    }

    fn complete_slot(&mut self, side: Side, position: usize) {
        if !self.completed_slots.contains(&(side, position)) {
            self.completed_slots.push((side, position));
        }
    }

    pub(crate) fn instruct(&mut self, actor: ParticipantId) -> bool {
        let Some(position) = self.position(actor) else {
            return false;
        };
        let pokemon = self.get(actor);
        if pokemon.hp == 0 || pokemon.recharging {
            return false;
        }
        let Some(id) = pokemon.last_used.clone() else {
            return false;
        };
        if pokemon.move_pp.get(&id) == Some(&0) {
            return false;
        }
        let Some(spec) = self.catalog.get(&id) else {
            return false;
        };
        let (side, target_position) = pokemon.last_move_target.unwrap_or((actor.side, position));
        let Some(&index) = self.active[side.index()].get(target_position) else {
            return false;
        };
        let entry = QueuedAction {
            actor,
            position,
            action: SelectedAction::Move(MoveAction {
                forced: false,
                move_id: id,
                target: ParticipantId { side, index },
                target_position,
            }),
            priority: spec.priority,
            fractional_priority: 0,
            speed: self.action_speed(actor),
            tie: 0,
            custap: false,
            quick_draw: false,
            promoted: true,
            postponed: false,
        };
        self.turn_queue.push_front(entry);
        true
    }

    fn resume_turn(&mut self, events: &mut Vec<BattleEvent>) -> Result<(), BattleError> {
        self.refresh_pending_speed();
        while let Some(entry) = self.turn_queue.pop_front() {
            if self.winner.is_some() {
                break;
            }
            let actor = entry.actor;
            let choice = entry.action;
            self.pending_actors = self
                .turn_queue
                .iter()
                .filter_map(|entry| {
                    matches!(entry.action, SelectedAction::Move(_)).then_some(entry.actor)
                })
                .collect();
            self.hit_reactions
                .retain(|reaction| reaction.owner != actor);
            if !self.is_active(actor) {
                self.complete_slot(actor.side, entry.position);
                continue;
            }
            if !matches!(choice, SelectedAction::Switch(_)) && self.get(actor).carried_by.is_none()
            {
                self.check_sleep(actor, events);
            }
            match &choice {
                SelectedAction::Switch(to) => {
                    if self.get(actor).hp > 0 && !self.trapped(actor) {
                        self.intercept_switch(actor, events)?;
                        if self.get(actor).hp > 0 {
                            self.switch_member(actor, *to, events);
                        }
                    }
                }
                SelectedAction::Move(choice) => self.execute_move(actor, choice, events)?,
                SelectedAction::Recharge => {
                    self.get_mut(actor).recharging = false;
                    self.end_condition(Scope::Pokemon(actor), "truant");
                    if self.get(actor).hp > 0 {
                        events.push(BattleEvent::Message(format!(
                            "{} must recharge!",
                            self.get(actor).name
                        )));
                    }
                }
            }
            self.update_berries(events);
            self.get_mut(actor).acted = true;
            self.complete_slot(actor.side, self.position(actor).unwrap_or(entry.position));
            self.resolve_outcome(events);
            self.refresh_pending_speed();
            if !self.hurry.is_empty() {
                // Only pending actions move; promoted actors retain their relative order.
                let promoted = std::mem::take(&mut self.hurry);
                let (now, later): (std::collections::VecDeque<_>, std::collections::VecDeque<_>) =
                    std::mem::take(&mut self.turn_queue)
                        .into_iter()
                        .partition(|entry| promoted.contains(&entry.actor));
                self.turn_queue = now;
                for entry in &mut self.turn_queue {
                    entry.promoted = true;
                    entry.postponed = false;
                }
                self.turn_queue.extend(later);
            }
            if let Some(who) = self.pivot.take()
                && self.winner.is_none()
                && self.get(who).hp > 0
                && self.is_active(who)
            {
                // Gen 8 Baton Pass skips Pursuit's BeforeSwitchOut interception.
                if self
                    .baton_pass
                    .as_ref()
                    .is_none_or(|(source, _)| *source != who)
                {
                    self.intercept_switch(who, events)?;
                }
                if self.get(who).hp == 0 || self.winner.is_some() {
                    continue;
                }
                let choices: Vec<_> = self
                    .bench(who.side)
                    .into_iter()
                    .enumerate()
                    .map(|(id, to)| Choice::Switch { id: id as u32, to })
                    .collect();
                if !choices.is_empty() {
                    self.make_prompt(who, choices, PromptReason::Pivot);
                    return Ok(());
                }
            }
        }
        self.baton_pass = None;
        let order: Vec<_> = std::mem::take(&mut self.completed_slots)
            .into_iter()
            .map(|(side, pos)| ParticipantId {
                side,
                index: self.active[side.index()][pos],
            })
            .collect();
        if self.winner.is_none() {
            self.resolve_delayed_attacks(events)?;
        }
        if self.winner.is_none() {
            self.resolve_delayed_heals(events);
            self.residual_status_cures(events);
            for actor in &order {
                self.post_turn_status(*actor, events);
                self.update_berries(events);
                self.resolve_outcome(events);
                if self.winner.is_some() {
                    break;
                }
            }
        }
        if self.winner.is_none() {
            self.post_turn_weather(&order, events);
            self.update_berries(events);
            self.resolve_outcome(events);
        }
        if self.winner.is_none() {
            self.tick_conditions(&order, events);
            self.bad_dreams_residual(events);
            self.ability_residual(events);
            self.residual_berries(events);
            self.update_berries(events);
            self.resolve_outcome(events);
        }
        for pokemon in self.sides.iter_mut().flatten() {
            pokemon.protected = false;
            pokemon.flinched = false;
        }
        self.turn_queue.clear();
        self.resolving_turn = false;
        Ok(())
    }
    fn check_sleep(&mut self, id: ParticipantId, events: &mut Vec<BattleEvent>) {
        if self.get(id).hp == 0 || self.get(id).status != Some(Status::Asleep) {
            return;
        }
        if self.has_ability(id, "earlybird") {
            self.get_mut(id).status_turns = self.get(id).status_turns.saturating_sub(1);
        }
        if self.get(id).status_turns == 0 {
            let name = self.get(id).name.clone();
            self.get_mut(id).status = None;
            self.remove_expired_status_conditions(id);
            events.push(BattleEvent::Status {
                target: id,
                status: None,
            });
            events.push(BattleEvent::Message(format!("{name} woke up!")));
        } else {
            self.get_mut(id).status_turns -= 1;
        }
    }
    pub(crate) fn record_protection(&mut self, actor: ParticipantId) {
        let turn = self.turn;
        let p = self.get_mut(actor);
        p.protection_streak = if p.last_protection_turn.is_some_and(|last| last + 1 >= turn) {
            p.protection_streak.saturating_add(1)
        } else {
            1
        };
        p.last_protection_turn = Some(turn);
    }

    pub(crate) fn protection_check(&mut self, actor: ParticipantId) -> bool {
        let pokemon = self.get(actor);
        let streak = if pokemon
            .last_protection_turn
            .is_some_and(|turn| turn + 1 >= self.turn)
        {
            pokemon.protection_streak
        } else {
            0
        };
        let succeeds = streak == 0 || self.rng.chance((1.0f32 / 3.0).powi(i32::from(streak)));
        let turn = self.turn;
        let pokemon = self.get_mut(actor);
        pokemon.protection_streak = if succeeds {
            streak.saturating_add(1)
        } else {
            0
        };
        pokemon.last_protection_turn = succeeds.then_some(turn);
        succeeds
    }

    fn execute_move(
        &mut self,
        actor: ParticipantId,
        choice: &MoveAction,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        // The queued action retains its original priority. Encore changes its move at execution.
        let mut overridden = choice.clone();
        if !choice.forced
            && choice.move_id != "struggle"
            && let Some(required) = self.only_move(actor)
        {
            overridden.move_id = required.to_owned();
            let spec = self.catalog.get(required).expect("learned move");
            let kind = self.move_target_kind(actor, spec);
            if matches!(
                kind,
                Target::User | Target::UserOrAlly | Target::UserAndAllies
            ) {
                overridden.target = actor;
                overridden.target_position = self.position(actor).expect("active actor");
            } else if overridden.target == actor {
                let foes: Vec<_> = self
                    .actors()
                    .into_iter()
                    .filter(|id| id.side != actor.side)
                    .collect();
                if !foes.is_empty() {
                    overridden.target = foes[self.rng.index(foes.len())];
                    overridden.target_position =
                        self.position(overridden.target).expect("active foe");
                }
            }
        }
        let choice = &overridden;
        self.move_failed = false;
        self.current_move = None;
        self.substitute_hits.clear();
        self.active_gem = None;
        self.expire_action_conditions(actor, Some(&choice.move_id));
        let result = self.perform_move(actor, choice, false, events);
        self.finish_item_hits(events);
        self.finish_ability_knockouts(events);
        self.substitute_hits.clear();
        self.ignored_ability = None;
        self.ability_bypass = false;
        let used = result?;
        if used && let Some(move_id) = self.current_move.take() {
            self.last_battle_move = Some(move_id);
        }
        if !used {
            self.expire_action_conditions(actor, None);
        }
        self.finish_forced_switches(events);
        let failed = self.move_failed;
        let pokemon = self.get_mut(actor);
        if used {
            pokemon.last_move_failed = failed;
        }
        // A streak is the same move used turn after turn without failing.
        if used && !failed {
            pokemon.streak = if pokemon.streak_move.as_deref() == Some(&choice.move_id) {
                pokemon.streak.saturating_add(1)
            } else {
                1
            };
            pokemon.streak_move = Some(choice.move_id.clone());
        } else {
            pokemon.streak = 0;
            pokemon.streak_move = None;
        }
        Ok(())
    }
    fn snatcher(&mut self, actor: ParticipantId, spec: &MoveSpec) -> Option<ParticipantId> {
        if self.snatching {
            return None;
        }
        let mut candidates: Vec<_> = self.actors().into_iter().filter(|who| *who != actor && self.get(*who).carried_by.is_none() && self.rules(*who).any(|rule| matches!(rule,Rule::Snatch{moves} if spec.has_flag("snatch") || moves.contains(&spec.id)))).collect();
        candidates.sort_by(|a, b| {
            self.stat_value(*b, CombatStat::Speed)
                .total_cmp(&self.stat_value(*a, CombatStat::Speed))
        });
        let first = *candidates.first()?;
        let fastest = self.stat_value(first, CombatStat::Speed);
        candidates.retain(|who| self.stat_value(*who, CombatStat::Speed) == fastest);
        Some(candidates[self.rng.index(candidates.len())])
    }

    pub(crate) fn call_move(
        &mut self,
        actor: ParticipantId,
        move_id: &str,
        target: Option<ParticipantId>,
        events: &mut Vec<BattleEvent>,
    ) -> Result<bool, BattleError> {
        if self.call_depth >= 8 || self.get(actor).hp == 0 || !self.is_active(actor) {
            return Ok(false);
        }
        let Some(spec) = self.catalog.get(move_id) else {
            return Ok(false);
        };
        let kind = self.move_target_kind(actor, spec);
        let target = if matches!(
            kind,
            Target::User | Target::Field | Target::UserAndAllies | Target::All
        ) {
            actor
        } else if kind == Target::UserOrAlly {
            target
                .filter(|id| id.side == actor.side && self.is_active(*id) && self.get(*id).hp > 0)
                .unwrap_or(actor)
        } else if let Some(target) = target.filter(|id| {
            self.is_active(*id)
                && self.get(*id).hp > 0
                && *id != actor
                && (kind != Target::Ally || (id.side == actor.side && self.adjacent(actor, *id)))
        }) {
            target
        } else {
            let choices: Vec<_> = self
                .actors()
                .into_iter()
                .filter(|id| {
                    if kind == Target::Ally {
                        id.side == actor.side && *id != actor && self.adjacent(actor, *id)
                    } else {
                        id.side != actor.side
                            && (!spec.has_flag("adjacent_target") || self.adjacent(actor, *id))
                    }
                })
                .collect();
            if choices.is_empty() {
                return Ok(false);
            }
            choices[self.rng.index(choices.len())]
        };
        let choice = MoveAction {
            forced: false,
            move_id: move_id.to_owned(),
            target,
            target_position: self.position(target).expect("active target"),
        };
        let previous_failed = self.move_failed;
        let power = self.active_power_multiplier;
        let gem = self.active_gem.take();
        let bypass = self.ability_bypass;
        let ignored = self.ignored_ability;
        self.move_failed = false;
        self.call_depth += 1;
        let result = self.perform_move(actor, &choice, true, events);
        let failed = self.move_failed;
        self.call_depth -= 1;
        self.move_failed = previous_failed;
        self.active_power_multiplier = power;
        self.active_gem = gem;
        self.ability_bypass = bypass;
        self.ignored_ability = ignored;
        result.map(|used| used && !failed)
    }

    /// Uses a move. Returns false if the Pokémon could not act at all.
    fn perform_move(
        &mut self,
        actor: ParticipantId,
        choice: &MoveAction,
        called: bool,
        events: &mut Vec<BattleEvent>,
    ) -> Result<bool, BattleError> {
        if self.get(actor).hp == 0 || !self.is_active(actor) || self.get(actor).carried_by.is_some()
        {
            return Ok(false);
        }
        if choice.forced
            && self.get(actor).script_continuation.is_none()
            && !self
                .get(actor)
                .locked_move
                .as_ref()
                .is_some_and(|lock| lock.charging)
        {
            return Ok(false);
        }
        let spec = self
            .catalog
            .get(&choice.move_id)
            .expect("validated choice")
            .clone();
        if let Some(message) = if called {
            self.called_move_block(actor, &spec)
        } else {
            self.move_block(actor, &spec)
        } {
            events.push(BattleEvent::Message(crate::conditions::named(
                message,
                &self.get(actor).name,
            )));
            self.clear_consecutive(actor);
            self.interrupt_script(actor, choice.target, &spec, events)?;
            let pokemon = self.get_mut(actor);
            pokemon.locked_move = None;
            pokemon.semi_invulnerable = false;
            return Ok(false);
        }
        // A Pokémon that hid through a script comes back when it next acts.
        if self.get(actor).semi_invulnerable
            && self.get(actor).locked_move.is_none()
            && self.get(actor).carrying.is_none()
        {
            self.get_mut(actor).semi_invulnerable = false;
        }
        if !spec.has_flag("calls_move") && !spec.has_flag("pass_volatiles") {
            self.active_power_multiplier = self
                .get_mut(actor)
                .next_move_power_boost
                .take()
                .filter(|boost| {
                    boost.move_type == spec.move_type && spec.category != Category::Status
                })
                .map_or(1.0, |boost| boost.multiplier);
        }
        let user_name = self.get(actor).name.clone();
        let forced_script = self
            .get(actor)
            .script_continuation
            .as_ref()
            .is_some_and(|lock| lock.move_id == spec.id);
        let self_thaw = spec.usable_while_frozen
            && (!spec.has_flag("thaw_requires_fire")
                || self.types_of(actor).contains(&PokemonType::Fire));
        if !called {
            let status = self.get(actor).status;
            match status {
                Some(Status::Frozen) if self_thaw => {}
                Some(Status::Frozen) if self.rng.chance(0.2) => {
                    self.thaw(actor, events);
                }
                Some(Status::Frozen) => {
                    events.push(BattleEvent::Message(format!(
                        "{user_name} is frozen solid!"
                    )));
                    self.interrupt_script(actor, choice.target, &spec, events)?;
                    return Ok(false);
                }
                Some(Status::Asleep) if !spec.usable_while_asleep => {
                    events.push(BattleEvent::Message(format!("{user_name} is fast asleep.")));
                    self.clear_consecutive(actor);
                    self.interrupt_script(actor, choice.target, &spec, events)?;
                    return Ok(false);
                }
                _ => {}
            }
            if self.has_ability(actor, "truant") {
                if self.end_condition(Scope::Pokemon(actor), "truant") {
                    events.push(BattleEvent::Message(format!(
                        "{user_name} is loafing around!"
                    )));
                    self.clear_consecutive(actor);
                    self.interrupt_script(actor, choice.target, &spec, events)?;
                    return Ok(false);
                }
                self.set_mark(Scope::Pokemon(actor), "truant", 1, None);
            }
            if self.get(actor).flinched {
                events.push(BattleEvent::Message(format!("{user_name} flinched!")));
                if self.has_ability(actor, "steadfast") {
                    self.change_stat(actor, Stat::Speed, 1, events);
                }
                self.get_mut(actor).flinched = false;
                self.clear_consecutive(actor);
                self.interrupt_script(actor, choice.target, &spec, events)?;
                return Ok(false);
            }
            if let Some(turns) = self.get(actor).confused_turns {
                if turns == 0 {
                    self.get_mut(actor).confused_turns = None;
                    events.push(BattleEvent::Message(format!(
                        "{user_name} snapped out of confusion!"
                    )));
                } else {
                    self.get_mut(actor).confused_turns = Some(turns - 1);
                    events.push(BattleEvent::Message(format!("{user_name} is confused!")));
                    if self.rng.chance(0.33) {
                        events.push(BattleEvent::Message(
                            "It hurt itself in its confusion!".into(),
                        ));
                        let amount = self.confusion_damage(actor);
                        self.damage(actor, amount, events);
                        self.clear_consecutive(actor);
                        self.interrupt_script(actor, choice.target, &spec, events)?;
                        return Ok(false);
                    }
                }
            }
            if self.infatuation_prevents_move(actor) {
                events.push(BattleEvent::Message(format!(
                    "{user_name} is immobilized by love!"
                )));
                self.clear_consecutive(actor);
                self.interrupt_script(actor, choice.target, &spec, events)?;
                return Ok(false);
            }
            // Paralysis follows flinch, confusion and attraction in Gen 8.
            if self.get(actor).status == Some(Status::Paralyzed) && self.rng.chance(0.25) {
                events.push(BattleEvent::Message(format!(
                    "{user_name} is paralyzed! It can't move!"
                )));
                self.clear_consecutive(actor);
                self.interrupt_script(actor, choice.target, &spec, events)?;
                return Ok(false);
            }
        }
        if !called
            && self.get(actor).locked_move.is_none()
            && !forced_script
            && self.get(actor).move_pp.get(&spec.id) == Some(&0)
        {
            events.push(BattleEvent::Message(format!(
                "{user_name} has no PP left for {}!",
                spec.name
            )));
            self.move_failed = true;
            self.clear_consecutive(actor);
            return Ok(true);
        }
        if !called && self_thaw {
            self.thaw(actor, events);
        }
        if !called {
            let pokemon = self.get_mut(actor);
            pokemon.last_used = Some(spec.id.clone());
            pokemon.last_move_target = Some((choice.target.side, choice.target_position));
            pokemon.moves_used.insert(spec.id.clone());
        }
        if !called {
            self.prankster_boosted =
                spec.category == Category::Status && self.has_ability(actor, "prankster");
        }
        self.current_move = Some(spec.id.clone());
        self.get_mut(actor).last_move_type = Some(self.converted_type(actor, spec.move_type));
        if spec.effects.iter().any(|e| matches!(e, Effect::Protect))
            && !self.protection_check(actor)
        {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message("But it failed!".into()));
            self.move_failed = true;
            self.clear_consecutive(actor);
            return Ok(true);
        }
        let mut target = self.get(actor).carrying.unwrap_or(ParticipantId {
            side: choice.target.side,
            index: self.active[choice.target.side.index()][choice.target_position],
        });
        let target_kind = self.move_target_kind(actor, &spec);
        if target_kind == Target::User {
            target = actor;
        }
        // Forced turns pick a new opponent too when the script asked for one. This shares the
        // roll below, so a move that is random in both ways still rolls once.
        let random_continuation = forced_script
            && self
                .get(actor)
                .script_continuation
                .as_ref()
                .is_some_and(|lock| lock.target_policy == ContinuationTarget::RandomOpponent);
        if target_kind == Target::RandomOpponent || random_continuation {
            let foes: Vec<_> = self
                .actors()
                .into_iter()
                .filter(|id| {
                    id.side != actor.side
                        && (!spec.has_flag("adjacent_target") || self.adjacent(actor, *id))
                })
                .collect();
            if foes.is_empty() {
                return Ok(false);
            }
            target = foes[self.rng.index(foes.len())];
        }
        if self.get(target).hp == 0
            && matches!(
                target_kind,
                Target::Selected | Target::Ally | Target::AnyOther
            )
        {
            // The chosen Pokémon is gone: another one on its side takes its place, never the user.
            let Some(replacement) = self
                .actors()
                .into_iter()
                .find(|id| id.side == target.side && *id != actor)
            else {
                return Ok(false);
            };
            target = replacement;
        }
        if self.get(target).hp == 0 && target_kind == Target::UserOrAlly {
            target = actor;
        }
        let redirected = if self.get(actor).carrying == Some(target)
            || (self.switching_target == Some(target) && spec.has_flag("intercepts_switch"))
        {
            target
        } else {
            self.redirect_target(actor, target, &spec)
        };
        if redirected != target {
            target = redirected;
            events.push(BattleEvent::Message(format!(
                "{} drew in the move!",
                self.get(target).name
            )));
        }
        self.ability_bypass = spec.has_flag("ignore_ability")
            || self.has_ability(actor, "moldbreaker")
            || self.has_ability(actor, "teravolt")
            || self.has_ability(actor, "turboblaze");
        self.ignored_ability = self.ability_bypass.then_some(target);
        if let Some(snatcher) = self.snatcher(actor, &spec) {
            if !called
                && !spec.auto_only
                && !forced_script
                && self.get(actor).locked_move.is_none()
                && let Some(pp) = self.get_mut(actor).move_pp.get_mut(&spec.id)
            {
                *pp = pp.saturating_sub(1);
            }
            self.get_mut(snatcher).conditions.retain(|condition| {
                !condition
                    .rules
                    .iter()
                    .any(|rule| matches!(rule, Rule::Snatch { .. }))
            });
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message(format!(
                "{} snatched {}!",
                self.get(snatcher).name,
                spec.name
            )));
            self.snatching = true;
            let result = self.call_move(snatcher, &spec.id, None, events);
            self.snatching = false;
            self.move_failed = !result?;
            return Ok(true);
        }
        if spec.fail_on_full_hp && self.get(target).hp == self.get(target).max_hp {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message("But it failed!".into()));
            self.move_failed = true;
            return Ok(true);
        }
        if !called
            && !spec.auto_only
            && !forced_script
            && self.get(actor).locked_move.is_none()
            && let Some(pp) = self.get_mut(actor).move_pp.get_mut(&spec.id)
        {
            *pp = pp.saturating_sub(1);
        }
        if matches!(
            crate::items::item_key(&spec.id).as_str(),
            "explosion" | "selfdestruct" | "mindblown" | "mistyexplosion"
        ) && self.damp_active()
        {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message("Damp prevented the explosion!".into()));
            self.move_failed = true;
            return Ok(true);
        }
        if !spec.auto_only
            && self.explode_on_type(actor, self.converted_type(actor, spec.move_type), events)
        {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            self.move_failed = true;
            self.clear_consecutive(actor);
            self.interrupt_script(actor, target, &spec, events)?;
            return Ok(true);
        }
        // Rest cannot restore HP if sleep itself is prevented (for example by terrain).
        if spec.effects.iter().any(|e| {
            matches!(
                e,
                Effect::Status {
                    status: crate::model::AppliedStatus::RestSleep,
                    ..
                }
            )
        }) && (self.is_asleep(actor)
            || self.status_immune_to(actor, actor, Status::Asleep, spec.move_type, false))
        {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message("But it failed!".into()));
            self.move_failed = true;
            return Ok(true);
        }
        if target_kind == Target::Ally && (target == actor || !self.adjacent(actor, target)) {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message(
                "But there is no adjacent ally!".into(),
            ));
            self.move_failed = true;
            return Ok(true);
        }
        if spec.script.is_some() {
            if !spec.manual_announce {
                events.push(BattleEvent::Message(format!(
                    "{user_name} used {}!",
                    spec.name
                )));
            }
            let mut hit = self.execute_script(actor, target, &spec, events, ScriptHook::Main)?;
            let charging = spec.has_flag("charge")
                && !spec.has_flag("no_power_herb")
                && self
                    .get(actor)
                    .script_continuation
                    .as_ref()
                    .is_some_and(|lock| lock.turn == 1 && lock.total_turns == 2);
            if charging && self.try_skip_charge(actor, events) {
                let pokemon = self.get_mut(actor);
                pokemon
                    .script_continuation
                    .as_mut()
                    .expect("charging continuation")
                    .turn = 2;
                pokemon.semi_invulnerable = false;
                hit = self.execute_script(actor, target, &spec, events, ScriptHook::Main)?;
            }
            if let Some(lock) = self.get_mut(actor).script_continuation.as_mut()
                && lock.move_id == spec.id
            {
                if lock.turn >= lock.total_turns {
                    self.get_mut(actor).script_continuation = None;
                } else {
                    lock.turn += 1;
                }
            }
            if !hit {
                self.clear_consecutive(actor);
                return Ok(true);
            }
            let pokemon = self.get_mut(actor);
            pokemon.consecutive_count = if pokemon.last_move.as_deref() == Some(&spec.id) {
                pokemon.consecutive_count.saturating_add(1)
            } else {
                1
            };
            pokemon.last_move = Some(spec.id);
            return Ok(true);
        }
        let mut second_turn = self
            .get(actor)
            .locked_move
            .as_ref()
            .is_some_and(|lock| lock.charging);
        let mut first_charge = !second_turn && spec.effects.iter().any(|e| matches!(e, Effect::TwoTurn { skip_in_sun, .. } if !skip_in_sun || self.effective_weather() != Some(WeatherKind::Sun)));
        if first_charge && !spec.has_flag("no_power_herb") && self.try_skip_charge(actor, events) {
            first_charge = false;
            second_turn = true;
        }
        if second_turn {
            self.get_mut(actor).locked_move = None;
            self.get_mut(actor).semi_invulnerable = false;
        }
        if !first_charge {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
        }
        if !first_charge && self.prankster_blocked(actor, target, &spec) {
            self.immune_message(target, events);
            return Ok(true);
        }
        if !first_charge && self.priority_blocked(actor, target, &spec) {
            events.push(BattleEvent::Message(format!(
                "{} is protected from priority moves!",
                self.get(target).name
            )));
            self.move_failed = true;
            return Ok(true);
        }
        if !first_charge && self.guard_hit(actor, target, &spec, &Default::default(), events) {
            events.push(BattleEvent::Message(format!(
                "{} was protected!",
                self.get(target).name
            )));
            self.move_failed = true;
            return Ok(true);
        }
        if !first_charge && self.reflect_move(actor, target, &spec, events)? {
            return Ok(true);
        }
        if !(first_charge || self.hit_check(actor, target, &spec, false)) {
            events.push(BattleEvent::Message(format!(
                "{user_name}'s attack missed!"
            )));
            self.move_failed = true;
            self.clear_consecutive(actor);
            return Ok(true);
        }
        if self.get(target).protected
            && actor != target
            && !(spec.has_flag("contact") && self.has_ability(actor, "unseenfist"))
        {
            events.push(BattleEvent::Message(format!(
                "{} protected itself!",
                self.get(target).name
            )));
            self.move_failed = true;
            return Ok(true);
        }
        if !first_charge
            && actor != target
            && let Some(place) = self.get(target).hidden()
            && !(spec.hits_hidden.contains(&place)
                || self.always_hits(actor, target)
                || (spec.has_flag("poison_always_hits")
                    && self.types_of(actor).contains(&PokemonType::Poison)))
        {
            events.push(BattleEvent::Message(format!(
                "{} avoided the attack!",
                self.get(target).name
            )));
            self.move_failed = true;
            return Ok(true);
        }
        if !first_charge && self.ability_blocks_status_move(actor, target, &spec, events) {
            return Ok(true);
        }
        let has_damage = spec.effects.iter().any(|e| {
            matches!(
                e,
                Effect::Damage { .. }
                    | Effect::MultiHit { .. }
                    | Effect::Consecutive { .. }
                    | Effect::TwoTurn { .. }
            )
        });
        let single_effect = spec.effects.len() == 1;
        for effect in &spec.effects {
            if self.get(actor).hp == 0 {
                break;
            }
            if self.get(target).hp == 0
                && !matches!(
                    effect,
                    Effect::Stats {
                        self_target: true,
                        ..
                    } | Effect::Weather { .. }
                )
            {
                continue;
            }
            self.apply_effect(
                actor,
                target,
                &spec,
                effect,
                has_damage,
                single_effect,
                second_turn,
                events,
            )?;
        }
        let pokemon = self.get_mut(actor);
        pokemon.consecutive_count = if pokemon.last_move.as_deref() == Some(&spec.id) {
            pokemon.consecutive_count.saturating_add(1)
        } else {
            1
        };
        pokemon.last_move = Some(spec.id);
        Ok(true)
    }
    pub(crate) fn interrupt_script(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        if self.get(actor).script_continuation.is_some() {
            if spec.on_interrupt.is_some() {
                self.execute_script(actor, target, spec, events, ScriptHook::Interrupt)?;
            }
            self.get_mut(actor).script_continuation = None;
        }
        self.release_carry(actor, events);
        Ok(())
    }
    fn clear_consecutive(&mut self, actor: ParticipantId) {
        let p = self.get_mut(actor);
        p.consecutive_count = 0;
        p.last_move = None;
        if p.locked_move.as_ref().is_some_and(|l| !l.charging) {
            p.locked_move = None;
        }
    }
    /// Changes a stat stage, and returns by how much it really moved.
    pub(crate) fn change_stat(
        &mut self,
        recipient: ParticipantId,
        stat: Stat,
        delta: i8,
        events: &mut Vec<BattleEvent>,
    ) -> i8 {
        let delta = self.ability_stage_delta(recipient, delta);
        self.change_stat_unmodified(recipient, stat, delta, events)
    }

    pub(crate) fn change_stat_from(
        &mut self,
        actor: ParticipantId,
        recipient: ParticipantId,
        stat: Stat,
        delta: i8,
        events: &mut Vec<BattleEvent>,
    ) -> i8 {
        self.change_stat_from_inner(actor, recipient, stat, delta, false, events)
    }

    fn change_stat_from_inner(
        &mut self,
        actor: ParticipantId,
        recipient: ParticipantId,
        stat: Stat,
        delta: i8,
        reflected: bool,
        events: &mut Vec<BattleEvent>,
    ) -> i8 {
        let delta = self.ability_stage_delta(recipient, delta);
        if actor != recipient
            && delta < 0
            && !reflected
            && self.get(recipient).stage(stat) > -6
            && self.has_ability(recipient, "mirrorarmor")
        {
            if self.get(actor).hp > 0 {
                self.change_stat_from_inner(recipient, actor, stat, delta, true, events);
            }
            return 0;
        }
        if actor != recipient
            && delta < 0
            && (self.blocks_stat_drops_from(actor, recipient)
                || (stat == Stat::Attack && self.has_ability(recipient, "hypercutter"))
                || (stat == Stat::Accuracy && self.has_ability(recipient, "keeneye"))
                || (stat == Stat::Defense && self.has_ability(recipient, "bigpecks")))
        {
            events.push(BattleEvent::Message(format!(
                "{}'s stats were not lowered!",
                self.get(recipient).name
            )));
            return 0;
        }
        let changed = self.change_stat_unmodified(recipient, stat, delta, events);
        if changed < 0 && actor.side != recipient.side && self.has_ability(recipient, "defiant") {
            self.change_stat(recipient, Stat::Attack, 2, events);
        }
        if changed < 0 && actor.side != recipient.side && self.has_ability(recipient, "competitive")
        {
            self.change_stat(recipient, Stat::SpAttack, 2, events);
        }
        changed
    }

    pub(crate) fn change_stat_unmodified(
        &mut self,
        recipient: ParticipantId,
        stat: Stat,
        delta: i8,
        events: &mut Vec<BattleEvent>,
    ) -> i8 {
        let pokemon = self.get_mut(recipient);
        let change = pokemon.change_stage(stat, delta);
        pokemon.stats_raised_this_turn |= change > 0;
        pokemon.stats_lowered_this_turn |= change < 0;
        let tail = match change {
            3.. => "rose drastically!",
            2 => "rose sharply!",
            1 => "rose!",
            -1 => "fell!",
            -2 => "harshly fell!",
            ..=-3 => "severely fell!",
            _ if delta > 0 => "won't go any higher!",
            _ => "won't go any lower!",
        };
        events.push(BattleEvent::Message(format!(
            "{}'s {} {tail}",
            self.get(recipient).name,
            stat.description()
        )));
        if change != 0 {
            events.push(BattleEvent::StatChange {
                target: recipient,
                stat,
                stages: change,
            });
        }
        change
    }
    pub(crate) fn hit_check(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        ignore_evasion: bool,
    ) -> bool {
        if spec.has_flag("poison_always_hits")
            && self.types_of(actor).contains(&PokemonType::Poison)
        {
            return true;
        }
        match spec.accuracy {
            Accuracy::Always => {
                self.end_condition(Scope::Pokemon(actor), "micle_berry");
                true
            }
            Accuracy::OneHitKnockout => self.ohko_hit_check(actor, target, 0.3, false),
            Accuracy::Chance(value) => {
                if self.always_hits(actor, target) {
                    self.end_condition(Scope::Pokemon(actor), "micle_berry");
                    true
                } else {
                    self.hit_check_chance(actor, target, value, ignore_evasion, spec.category)
                }
            }
        }
    }
    pub(crate) fn ohko_hit_check(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        base: f32,
        never_miss: bool,
    ) -> bool {
        let user_level = self.get(actor).level;
        let target_level = self.get(target).level;
        user_level >= target_level
            && !self.has_ability(target, "sturdy")
            && (never_miss
                || self.always_hits(actor, target)
                || self
                    .rng
                    .chance(base + 0.01 * f32::from(user_level - target_level)))
    }
    pub(crate) fn hit_check_chance(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        value: f32,
        ignore_evasion: bool,
        category: Category,
    ) -> bool {
        if self
            .rules(target)
            .any(|r| matches!(r, Rule::IncomingAlwaysHit))
        {
            self.end_condition(Scope::Pokemon(actor), "micle_berry");
            return true;
        }
        let evasion = if ignore_evasion
            || self.has_ability(actor, "unaware")
            || (self.get(target).stage(Stat::Evasion) > 0
                && (self.has_ability(actor, "keeneye")
                    || self
                        .rules(target)
                        .any(|rule| matches!(rule, Rule::IgnoreEvasionBoost))))
        {
            1.0
        } else {
            self.get(target).evasion_factor()
        };
        let tangled = if self.get(target).confused_turns.is_some()
            && self.has_ability(target, "tangledfeet")
        {
            0.5
        } else {
            1.0
        };
        let value = if category == Category::Status
            && actor != target
            && self.has_ability(target, "wonderskin")
        {
            value.min(0.5)
        } else {
            value
        };
        let probability = value
            * if self.has_ability(target, "unaware") {
                1.0
            } else {
                self.get(actor).accuracy_factor()
            }
            * evasion
            * self.accuracy_rule_factor(actor)
            * tangled
            * self.ability_accuracy_factor(actor, target, category);
        self.end_condition(Scope::Pokemon(actor), "micle_berry");
        self.rng.chance(probability)
    }
    // These inputs correspond to C# PEffectAttributes plus the event sink.
    #[allow(clippy::too_many_arguments)]
    fn apply_effect(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        effect: &Effect,
        has_damage: bool,
        single_effect: bool,
        second_turn: bool,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        if has_damage
            && actor != target
            && self.has_ability(target, "shielddust")
            && matches!(
                effect,
                Effect::Status { .. }
                    | Effect::Confuse
                    | Effect::Flinch(_)
                    | Effect::Stats {
                        self_target: false,
                        ..
                    }
            )
        {
            return Ok(());
        }
        if self.substitute_blocks_effect(actor, target, spec)
            && matches!(
                effect,
                Effect::Stats {
                    self_target: false,
                    ..
                } | Effect::Status { .. }
                    | Effect::Confuse
                    | Effect::Flinch(_)
                    | Effect::Bind
                    | Effect::Heal { .. }
            )
        {
            if spec.category == Category::Status {
                self.move_failed = true;
                events.push(BattleEvent::Message(format!(
                    "{} is protected by its substitute!",
                    self.get(target).name
                )));
            }
            return Ok(());
        }
        match effect {
            Effect::Damage {
                power,
                recoil,
                drain,
                high_crit,
                always_crit,
            } => {
                let mut details = Vec::new();
                if let Some(roll) = self.calculate_damage(
                    actor,
                    target,
                    spec,
                    *power,
                    *high_crit,
                    *always_crit,
                    0,
                    false,
                    &mut details,
                ) {
                    let ooze = self.has_ability(target, "liquidooze");
                    let amount = self.damage_from_move(
                        actor,
                        target,
                        roll.amount,
                        &HitMeta::of(
                            spec,
                            spec.category,
                            self.converted_type(actor, spec.move_type),
                        )
                        .with_roll(&roll),
                        events,
                    )?;
                    events.extend(details);
                    if let Some(fraction) = recoil
                        && amount > 0
                        && !self.has_ability(actor, "magicguard")
                        && !self.has_ability(actor, "rockhead")
                    {
                        let recoil = ((amount as f32 * fraction).floor() as u16).max(1);
                        self.damage(actor, recoil, events);
                        events.push(BattleEvent::Message(format!(
                            "{} was damaged by the recoil!",
                            self.get(actor).name
                        )));
                    }
                    if let Some(fraction) = drain
                        && amount > 0
                    {
                        self.drain_health(
                            actor,
                            ((amount as f32 * fraction).floor() as u16).max(1),
                            ooze,
                            events,
                        );
                        events.push(BattleEvent::Message(format!(
                            "{} had its energy drained!",
                            self.get(target).name
                        )));
                    }
                }
            }
            Effect::MultiHit { power, min, max } => {
                let hits = self.roll_hit_count(actor, *min, *max);
                let mut count = 0;
                let mut details = Vec::new();
                for _ in 0..hits {
                    if self.get(actor).hp == 0 || self.get(target).hp == 0 {
                        break;
                    }
                    let mut hit_details = Vec::new();
                    if let Some(roll) = self.calculate_damage(
                        actor,
                        target,
                        spec,
                        *power,
                        false,
                        false,
                        0,
                        false,
                        &mut hit_details,
                    ) {
                        self.damage_from_move(
                            actor,
                            target,
                            roll.amount,
                            &HitMeta::of(
                                spec,
                                spec.category,
                                self.converted_type(actor, spec.move_type),
                            )
                            .with_roll(&roll),
                            events,
                        )?;
                        self.update_berries(events);
                        count += 1;
                        for event in hit_details {
                            if !details.contains(&event) {
                                details.push(event);
                            }
                        }
                    } else {
                        events.extend(hit_details);
                        return Ok(());
                    }
                }
                events.extend(details);
                events.push(BattleEvent::Message(format!("Hit {count} times!")));
            }
            Effect::FixedDamage(amount) => {
                self.direct_damage(actor, target, spec, *amount, events)?
            }
            Effect::LevelDamage => {
                self.direct_damage(actor, target, spec, self.get(actor).level as u16, events)?
            }
            Effect::OneHitKnockout => {
                self.direct_damage(actor, target, spec, self.get(target).max_hp, events)?;
                if self.effectiveness(target, spec.move_type) > 0.0 {
                    events.push(BattleEvent::Message("It's a one-hit KO!".into()));
                }
            }
            Effect::Stats {
                stages,
                chance,
                self_target,
            } => {
                let chance = self.effect_chance(actor, *chance);
                if !self.rng.chance(chance)
                    || (has_damage && self.effectiveness(target, spec.move_type) == 0.0)
                {
                    return Ok(());
                }
                let recipient = if *self_target { actor } else { target };
                if self.get(recipient).hp == 0 {
                    return Ok(());
                }
                for (stat, delta) in stages {
                    self.change_stat_from(actor, recipient, *stat, *delta, events);
                }
            }
            Effect::Status {
                status,
                chance,
                replace,
            } => {
                let chance = self.effect_chance(actor, *chance);
                if !self.rng.chance(chance)
                    || (has_damage && self.effectiveness(target, spec.move_type) == 0.0)
                {
                    return Ok(());
                }
                self.inflict_status(
                    actor,
                    target,
                    *status,
                    *replace,
                    spec.move_type,
                    single_effect,
                    events,
                );
            }
            Effect::Confuse => {
                self.rng.chance(1.0); // C# ConfuseEffect checks its default chance.
                if !self.confuse(actor, target, events)
                    && single_effect
                    && self.get(target).confused_turns.is_some()
                {
                    events.push(BattleEvent::Message(format!(
                        "{} was already confused!",
                        self.get(target).name
                    )));
                }
            }
            Effect::Flinch(chance) => {
                let chance = self.effect_chance(actor, *chance);
                if self.rng.chance(chance) && self.effectiveness(target, spec.move_type) != 0.0 {
                    self.flinch(target, spec.category != Category::Status);
                }
            }
            Effect::Bind => {
                if self.get(actor).hp > 0
                    && self.get(target).hp > 0
                    && self.effectiveness(target, spec.move_type) != 0.0
                    && self.get(target).bound.is_none()
                {
                    let turns = self.rng.range(4, 5);
                    self.get_mut(target).bound = Some(Bound {
                        by: Some(actor),
                        turns_left: turns,
                        source: spec.name.clone(),
                    });
                    events.push(BattleEvent::Message(format!(
                        "{} was squeezed by {}!",
                        self.get(target).name,
                        self.get(actor).name
                    )));
                }
            }
            Effect::Protect => {
                if !self.get(target).protected {
                    self.get_mut(target).protected = true;
                    events.push(BattleEvent::Message(format!(
                        "{} protected itself!",
                        self.get(target).name
                    )));
                }
            }
            Effect::Heal {
                fraction,
                hide_message,
            } => {
                let p = self.get(target);
                let amount = (p.max_hp as f32 * fraction).ceil() as u16;
                if p.hp < p.max_hp {
                    self.heal(target, amount, events);
                }
                if !hide_message {
                    events.push(BattleEvent::Message(format!(
                        "{} regained health!",
                        self.get(target).name
                    )));
                }
            }
            Effect::Weather { kind, turns } => {
                if !self.start_weather(*kind, *turns, events) {
                    self.move_failed = true;
                }
            }
            Effect::TwoTurn {
                power,
                charge_message,
                semi_invulnerable,
                skip_in_sun,
                hidden,
            } => {
                if !(second_turn
                    || *skip_in_sun && self.effective_weather() == Some(WeatherKind::Sun))
                {
                    self.get_mut(actor).locked_move = Some(LockedMove {
                        move_id: spec.id.clone(),
                        target,
                        turns: 1,
                        max_turns: 2,
                        charging: true,
                    });
                    self.get_mut(actor).semi_invulnerable = *semi_invulnerable;
                    self.get_mut(actor).hidden_kind = *hidden;
                    events.push(BattleEvent::Message(
                        charge_message.replace("{0}", &self.get(actor).name),
                    ));
                } else {
                    let mut details = Vec::new();
                    if let Some(roll) = self.calculate_damage(
                        actor,
                        target,
                        spec,
                        *power,
                        false,
                        false,
                        0,
                        false,
                        &mut details,
                    ) {
                        self.damage_from_move(
                            actor,
                            target,
                            roll.amount,
                            &HitMeta::of(
                                spec,
                                spec.category,
                                self.converted_type(actor, spec.move_type),
                            )
                            .with_roll(&roll),
                            events,
                        )?;
                    }
                    events.extend(details);
                }
            }
            Effect::Consecutive {
                power,
                min,
                max,
                confuse_after,
                double_power,
            } => {
                let previous = self.get(actor).locked_move.clone();
                let (turns, max_turns) = if let Some(lock) = previous {
                    (lock.turns + 1, lock.max_turns)
                } else {
                    (1, self.rng.range(*min, *max))
                };
                let boost = if *double_power { turns - 1 } else { 0 };
                let mut details = Vec::new();
                if let Some(roll) = self.calculate_damage(
                    actor,
                    target,
                    spec,
                    *power,
                    false,
                    false,
                    boost,
                    false,
                    &mut details,
                ) {
                    self.damage_from_move(
                        actor,
                        target,
                        roll.amount,
                        &HitMeta::of(
                            spec,
                            spec.category,
                            self.converted_type(actor, spec.move_type),
                        )
                        .with_roll(&roll),
                        events,
                    )?;
                }
                events.extend(details);
                if turns < max_turns {
                    self.get_mut(actor).locked_move = Some(LockedMove {
                        move_id: spec.id.clone(),
                        target,
                        turns,
                        max_turns,
                        charging: false,
                    });
                } else {
                    self.get_mut(actor).locked_move = None;
                    if *confuse_after {
                        self.rng.chance(1.0); // ConfuseEffect chance check precedes immunity.
                    }
                    if *confuse_after && self.effectiveness(target, spec.move_type) != 0.0 {
                        self.get_mut(actor).confused_turns = Some(self.rng.range(1, 4));
                        events.push(BattleEvent::Message(format!(
                            "{} became confused!",
                            self.get(actor).name
                        )));
                    }
                }
            }
            Effect::Splash => {
                if self.rng.chance(0.01) {
                    let amount = fractional(self.get(target).max_hp, 16, 1);
                    self.damage_from_move(
                        actor,
                        target,
                        amount,
                        &HitMeta::of(
                            spec,
                            spec.category,
                            self.converted_type(actor, spec.move_type),
                        ),
                        events,
                    )?;
                    events.push(BattleEvent::Message(
                        "Whoa! Its splash hit with force!".into(),
                    ));
                } else {
                    events.push(BattleEvent::Message("But nothing happened!".into()));
                }
            }
        }
        Ok(())
    }
    pub(crate) fn effectiveness(&self, target: ParticipantId, move_type: PokemonType) -> f32 {
        if move_type == PokemonType::Ground && self.floats_by_effect(target) {
            return 0.0;
        }
        type_chart::multiplier(move_type, &self.defending_types(target, move_type))
            * self.effectiveness_factor(target, move_type)
    }
    pub(crate) fn move_defending_types(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        kind: PokemonType,
    ) -> Vec<PokemonType> {
        let mut types = self.defending_types(target, kind);
        if matches!(kind, PokemonType::Normal | PokemonType::Fighting)
            && self.has_ability(actor, "scrappy")
        {
            types.retain(|t| *t != PokemonType::Ghost);
        }
        types
    }
    pub(crate) fn move_effectiveness(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        kind: PokemonType,
    ) -> f32 {
        if kind == PokemonType::Ground && self.floats_by_effect(target) {
            return 0.0;
        }
        type_chart::multiplier(kind, &self.move_defending_types(actor, target, kind))
            * self.effectiveness_factor(target, kind)
    }
    /// How well a hit does against a Pokémon's types, with the hit's own exceptions.
    pub(crate) fn matchup(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        attack: &Attack,
    ) -> f32 {
        let against = |move_type: PokemonType, exceptions: &[(PokemonType, f32)]| -> f32 {
            if exceptions.is_empty() {
                return if attack.hits_airborne {
                    type_chart::multiplier(
                        move_type,
                        &self.move_defending_types(actor, target, move_type),
                    ) * self.effectiveness_factor(target, move_type)
                } else {
                    self.move_effectiveness(actor, target, move_type)
                };
            }
            self.move_defending_types(actor, target, move_type)
                .iter()
                .map(|defender| {
                    exceptions
                        .iter()
                        .find(|(own, _)| own == defender)
                        .map_or_else(
                            || type_chart::multiplier(move_type, &[*defender]),
                            |(_, factor)| *factor,
                        )
                })
                .product::<f32>()
                * self.effectiveness_factor(target, move_type)
        };
        let first = against(attack.move_type, attack.effective);
        match attack.also_type {
            Some(second) => first * against(second, &[]),
            None => first,
        }
    }
    pub(crate) fn immune_message(&mut self, target: ParticipantId, events: &mut Vec<BattleEvent>) {
        self.move_failed = true;
        events.push(BattleEvent::Message(format!(
            "It doesn't affect {}...",
            self.get(target).name
        )));
    }
    fn direct_damage(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        amount: u16,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        if self.ability_bypass {
            self.ignored_ability = Some(target);
        }
        let effectiveness = self.move_effectiveness(actor, target, spec.move_type);
        if self.ability_blocks_damage(
            actor,
            target,
            spec.move_type,
            effectiveness,
            spec.has_flag("sound"),
            spec.has_flag("bullet"),
            spec.has_flag("hits_airborne"),
            events,
        ) {
            return Ok(());
        }
        if effectiveness == 0.0 {
            self.immune_message(target, events);
        } else {
            self.damage_from_move(
                actor,
                target,
                amount,
                &HitMeta::of(
                    spec,
                    spec.category,
                    self.converted_type(actor, spec.move_type),
                ),
                events,
            )?;
        }
        Ok(())
    }
    // Keep each modifier visible at the damage call sites during the parity port.
    #[allow(clippy::too_many_arguments)]
    fn calculate_damage(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        power: u16,
        high_crit: bool,
        always_crit: bool,
        consecutive_boost: u8,
        typeless: bool,
        events: &mut Vec<BattleEvent>,
    ) -> Option<Roll> {
        let attack = Attack {
            move_id: &spec.id,
            hits_airborne: spec.has_flag("hits_airborne"),
            power,
            high_crit,
            always_crit,
            force_stab: false,
            no_gem: spec.has_flag("no_gem"),
            sound: spec.has_flag("sound"),
            bypass_substitute: spec.has_flag("sound") || spec.has_flag("bypass_substitute"),
            bullet: spec.has_flag("bullet"),
            contact: spec.has_flag("contact"),
            consecutive_boost,
            typeless,
            move_type: self.converted_type(actor, spec.move_type),
            also_type: None,
            effective: &[],
            category: spec.category,
            attack_from: actor,
            attack_stat: None,
            defense_stat: None,
            ignore_stages: false,
        };
        self.calculate(actor, target, &attack, events)
    }
    /// The damage formula. Returns nothing if the target is immune.
    pub(crate) fn calculate(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        attack: &Attack,
        events: &mut Vec<BattleEvent>,
    ) -> Option<Roll> {
        if self.ability_bypass {
            self.ignored_ability = Some(target);
        }
        let defender = self.get(target).clone();
        let attacker = self.get(actor).clone();
        let source = self.get(attack.attack_from).clone();
        let move_type = attack.move_type;
        let typeless = attack.typeless;
        let crit_stage = self.rules(actor).fold(
            attacker
                .crit_stage
                .saturating_add(u8::from(attack.high_crit))
                .saturating_add(u8::from(self.has_ability(actor, "superluck"))),
            |stage, rule| match rule {
                Rule::CriticalStage { stages } => stage.saturating_add(*stages),
                _ => stage,
            },
        );
        let chance = match crit_stage {
            0 => 1.0 / 24.0,
            1 => 1.0 / 8.0,
            2 => 0.5,
            _ => 1.0,
        };
        let crit = (attack.always_crit
            || (self.has_ability(actor, "merciless")
                && matches!(
                    self.get(target).status,
                    Some(Status::Poisoned | Status::BadlyPoisoned)
                ))
            || self.rng.chance(chance))
            && !self
                .rules(target)
                .any(|rule| matches!(rule, Rule::BlockCritical))
            && !self.has_ability(target, "battlearmor")
            && !self.has_ability(target, "shellarmor");
        let effectiveness = if typeless {
            1.0
        } else {
            self.matchup(actor, target, attack)
        };
        if !typeless
            && self.ability_blocks_damage(
                actor,
                target,
                move_type,
                effectiveness,
                attack.sound,
                attack.bullet,
                attack.hits_airborne,
                events,
            )
        {
            return None;
        }
        if effectiveness == 0.0 {
            self.immune_message(target, events);
            return None;
        }
        let (atk_name, def_name) = match attack.category {
            Category::Physical => (CombatStat::Attack, CombatStat::Defense),
            _ => (CombatStat::SpAttack, CombatStat::SpDefense),
        };
        let category_attack_stat = atk_name;
        let atk_name = attack
            .attack_stat
            .map(|stat| self.swapped_defensive_stat(stat))
            .unwrap_or(atk_name);
        let def_name = attack.defense_stat.unwrap_or(def_name);
        let ignore_attack_stages =
            self.has_ability(target, "unaware") && attack.attack_from == actor;
        let mut attack_value = if attack.attack_stat.is_some() {
            // An offensive-stat override changes the base stat and stages, not
            // the user's category-based modifiers (Body Press still uses ModifyAtk).
            let value = if ignore_attack_stages || (crit && source.stage(atk_name.into()) < 0) {
                self.battle_raw_stat(attack.attack_from, atk_name)
            } else {
                self.battle_staged_stat(attack.attack_from, atk_name)
            };
            (value * self.stat_factor(actor, category_attack_stat)).trunc()
        } else if ignore_attack_stages || (crit && source.stage(atk_name.into()) < 0) {
            self.battle_raw_stat(attack.attack_from, atk_name)
                * self.stat_factor(attack.attack_from, atk_name)
        } else {
            self.stat_value(attack.attack_from, atk_name)
        };
        if self.has_ability(actor, "stakeout")
            && self.turn > 0
            && defender.entered_turn == self.turn
        {
            attack_value *= 2.0;
        }
        let mut defense = if attack.ignore_stages || self.has_ability(actor, "unaware") {
            (self.battle_raw_stat(target, def_name) * self.stat_factor(target, def_name)).trunc()
        } else if crit && defender.stage(def_name.into()) > 0 {
            self.battle_raw_stat(target, def_name) * self.stat_factor(target, def_name)
        } else {
            self.stat_value(target, def_name)
        };
        let weather = self.effective_weather();
        if weather == Some(WeatherKind::Sandstorm)
            && def_name == CombatStat::SpDefense
            && defender.types.contains(&PokemonType::Rock)
        {
            defense *= 1.5;
        }
        if weather == Some(WeatherKind::Hail)
            && def_name == CombatStat::Defense
            && defender.types.contains(&PokemonType::Ice)
        {
            defense *= 1.5;
        }
        defense = defense.max(1.0);
        let gem = if typeless || attack.no_gem || actor == target {
            1.0
        } else {
            self.gem_factor(actor, move_type, events)
        };
        // Base-power conditions round halves down, before the existing damage modifiers.
        let base_power = (f32::from(attack.power)
            * self.base_power_factor(actor, move_type)
            * self.ability_base_power_factor(actor, target, attack)
            * self.team_ability_power_factor(actor, attack)
            * self.incoming_base_power_factor(target, attack.move_id, move_type)
            - 0.5)
            .ceil()
            .max(1.0);
        let adjusted_power = gem
            * base_power
            * self.status_power_factor(actor, attack.category)
            * 2f32.powi(attack.consecutive_boost as i32)
            * self.active_power_multiplier;
        let mut damage =
            (2.0 * f32::from(attacker.level) * 0.2 + 2.0) * adjusted_power * attack_value
                / defense
                / 50.0
                + 2.0;
        if attacker.status == Some(Status::Burned)
            && attack.category == Category::Physical
            && !self.has_ability(actor, "guts")
            && !self
                .catalog
                .get(attack.move_id)
                .is_some_and(|m| m.has_flag("ignore_burn"))
        {
            damage /= 2.0;
        }
        if !typeless {
            let (boosted, weakened) = match weather {
                Some(WeatherKind::Sun) => (Some(PokemonType::Fire), Some(PokemonType::Water)),
                Some(WeatherKind::Rain) => (Some(PokemonType::Water), Some(PokemonType::Fire)),
                _ => (None, None),
            };
            if boosted == Some(move_type) {
                damage *= 1.5;
            }
            if weakened == Some(move_type) {
                damage *= 0.5;
            }
        }
        damage *= self.ability_damage_factor(actor, target, attack, effectiveness);
        damage *= effectiveness;
        if !typeless && (attack.force_stab || self.types_of(actor).contains(&move_type)) {
            damage *= if self.has_ability(actor, "adaptability") {
                2.0
            } else {
                1.5
            };
        }
        if crit {
            damage *= if self.has_ability(actor, "sniper") {
                2.25
            } else {
                1.5
            };
        }
        let dealt = self.damage_dealt_factor(actor, attack.category, move_type);
        let taken = self.damage_taken_factor(actor, target, attack.category, move_type, crit);
        if dealt != 1.0 || taken != 1.0 {
            damage *= dealt * taken;
        }
        if !typeless && !self.substitute_intercepts(actor, target, attack.bypass_substitute) {
            damage *= self.resistance_berry_factor(target, move_type, effectiveness, events);
        }
        damage *= self.rng.damage_roll();
        let amount = (damage.round() as u16).max(1);
        if effectiveness < 1.0 {
            events.push(BattleEvent::Message("It's not very effective...".into()));
        } else if effectiveness > 1.0 {
            events.push(BattleEvent::Message("It's super effective!".into()));
        }
        if crit {
            events.push(BattleEvent::Message("A critical hit!".into()));
        }
        Some(Roll {
            amount,
            critical: crit,
            effectiveness,
        })
    }
    /// Takes HP away as the damage of a move: it can be endured and it sets off hit reactions.
    pub(crate) fn damage_from_move(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        amount: u16,
        meta: &HitMeta,
        events: &mut Vec<BattleEvent>,
    ) -> Result<u16, BattleError> {
        let mut resolved_meta = meta.clone();
        resolved_meta.contact &= !self.has_ability(actor, "longreach");
        let meta = &resolved_meta;
        let before = self.get(target).hp;
        if before == 0 {
            return Ok(0);
        }
        self.substitute_hits.remove(&target);
        if self.substitute_intercepts(actor, target, meta.bypass_substitute) {
            let lost = amount.min(self.get(target).substitute_hp);
            self.get_mut(target).substitute_hp -= lost;
            self.substitute_hits.insert(target);
            events.push(BattleEvent::Substitute {
                target,
                amount: lost,
                hp: self.get(target).substitute_hp,
            });
            events.push(BattleEvent::Message(format!(
                "{}'s substitute {}!",
                self.get(target).name,
                if self.get(target).substitute_hp == 0 {
                    "broke"
                } else {
                    "took the hit"
                }
            )));
            return Ok(lost);
        }
        let deplete_pp = self
            .rules(target)
            .any(|rule| matches!(rule, Rule::DepleteAttackerPp))
            && !self
                .catalog
                .get(&meta.move_id)
                .is_some_and(|spec| spec.has_flag("future_move"));
        let faint_attacker = actor.side != target.side
            && self
                .rules(target)
                .any(|rule| matches!(rule, Rule::FaintAttacker))
            && !self
                .catalog
                .get(&meta.move_id)
                .is_some_and(|spec| spec.has_flag("future_move"));
        // Capture contact listeners before damage clears a fainted Pokémon's effects.
        let reactions: Vec<_> = self
            .hit_reactions
            .iter()
            .filter(|r| r.owner == target)
            .cloned()
            .collect();
        let mut amount = amount;
        let hit_ability = if self.ability_active(target) {
            crate::items::item_key(self.get(target).ability.as_deref().unwrap_or_default())
        } else {
            String::new()
        };
        if amount >= before
            && (self.endures(target)
                || (before == self.get(target).max_hp && self.has_ability(target, "sturdy")))
        {
            amount = before - 1;
            events.push(BattleEvent::Message(format!(
                "{} endured the hit!",
                self.get(target).name
            )));
        }
        if amount > 0 {
            self.damage(target, amount, events);
        }
        let lost = before - self.get(target).hp;
        if lost > 0 && meta.move_type == PokemonType::Fire && meta.category != Category::Status {
            self.thaw(target, events);
        }
        if lost > 0
            && self.has_ability(target, "rattled")
            && matches!(
                meta.move_type,
                PokemonType::Bug | PokemonType::Dark | PokemonType::Ghost
            )
        {
            self.change_stat(target, Stat::Speed, 1, events);
        }
        self.item_hits.push((actor, target, meta.clone()));
        if !meta.defer_items {
            self.berry_hit(actor, target, meta, events);
        }
        let hit = HitInfo {
            attacker: actor,
            move_id: meta.move_id.clone(),
            category: meta.category,
            move_type: meta.move_type,
            damage: lost,
            contact: meta.contact,
        };
        if lost > 0 {
            let pokemon = self.get_mut(target);
            pokemon.damage_taken = pokemon.damage_taken.saturating_add(lost);
            pokemon.last_hit = Some(hit.clone());
        }
        if lost > 0 && meta.critical && self.has_ability(target, "angerpoint") {
            self.change_stat(target, Stat::Attack, 12, events);
        }
        self.ability_damaging_hit(target, &hit_ability, &hit, events);
        self.ability_attacking_hit(target, &hit, events);
        for reaction in reactions {
            let triggers = if reaction.contact_only {
                meta.contact
            } else {
                lost > 0 && self.get(target).hp > 0
            };
            if !triggers {
                continue;
            }
            let spec = self
                .catalog
                .get(&reaction.move_id)
                .expect("registered reaction has a move")
                .clone();
            let previous = self.reaction_hit.replace(hit.clone());
            let result = self.execute_script(target, actor, &spec, events, ScriptHook::Hit);
            self.reaction_hit = previous;
            result?;
        }
        if deplete_pp
            && self.get(target).hp == 0
            && self.get(actor).hp > 0
            && let Some(move_id) = self.get(actor).last_used.clone()
            && let Some(pp) = self.get_mut(actor).move_pp.get_mut(&move_id)
        {
            *pp = 0;
            events.push(BattleEvent::Message(format!(
                "{} lost all PP for {} due to Grudge!",
                self.get(actor).name,
                move_id
            )));
        }
        if faint_attacker && self.get(target).hp == 0 && self.get(actor).hp > 0 {
            events.push(BattleEvent::Message(format!(
                "{} took its attacker down with it!",
                self.get(target).name
            )));
            self.damage(actor, self.get(actor).hp, events);
        }
        if lost > 0 && self.get(target).hp == 0 && actor != target {
            self.pending_knockouts.push(actor);
        }
        Ok(lost)
    }

    /// Residual/recoil damage is distinct from direct hits and deliberate HP costs.
    pub(crate) fn indirect_damage(
        &mut self,
        target: ParticipantId,
        amount: u16,
        events: &mut Vec<BattleEvent>,
    ) -> u16 {
        if self.has_ability(target, "magicguard") {
            return 0;
        }
        let before = self.get(target).hp;
        self.damage(target, amount, events);
        before - self.get(target).hp
    }

    pub(crate) fn damage(
        &mut self,
        target: ParticipantId,
        amount: u16,
        events: &mut Vec<BattleEvent>,
    ) {
        let turn = self.turn;
        let p = self.get_mut(target);
        let was_alive = p.hp > 0;
        p.hp = p.hp.saturating_sub(amount);
        if amount > 0 {
            p.hurt_this_turn = true;
        }
        let hp = p.hp;
        let name = p.name.clone();
        events.push(BattleEvent::Damage { target, amount, hp });
        if was_alive && hp == 0 {
            self.pending_faints
                .push((target, self.ability_state(target)));
            if self
                .baton_pass
                .as_ref()
                .is_some_and(|(source, _)| *source == target)
            {
                self.baton_pass = None;
            }
            self.restore_transform(target, events);
            self.get_mut(target).restore_moves();
            self.get_mut(target).substitute_hp = 0;
            self.release_carry(target, events);
            self.release_links(target);
            self.restore_form(target, events);
            self.get_mut(target).conditions.clear();
            self.get_mut(target).stages.clear();
            self.last_faint_turn[target.side.index()] = Some(turn);
            self.hit_reactions
                .retain(|reaction| reaction.owner != target);
            events.push(BattleEvent::Message(format!("{name} fainted!")));
            events.push(BattleEvent::Fainted(target));
        }
    }
    pub(crate) fn heal(
        &mut self,
        target: ParticipantId,
        amount: u16,
        events: &mut Vec<BattleEvent>,
    ) {
        if !self.healing_blocked(target) {
            self.heal_unchecked(target, amount, events);
        }
    }

    pub(crate) fn heal_unchecked(
        &mut self,
        target: ParticipantId,
        amount: u16,
        events: &mut Vec<BattleEvent>,
    ) {
        let p = self.get_mut(target);
        p.hp = p.hp.saturating_add(amount).min(p.max_hp);
        events.push(BattleEvent::Heal {
            target,
            amount,
            hp: p.hp,
        });
    }
    fn post_turn_status(&mut self, id: ParticipantId, events: &mut Vec<BattleEvent>) {
        if self.get(id).hp == 0 {
            return;
        }
        let p = self.get(id).clone();
        let dot = match p.status {
            Some(Status::Poisoned | Status::BadlyPoisoned) => {
                self.poison_residual(id, events);
                None
            }
            Some(Status::Burned) if !self.has_ability(id, "magicguard") => {
                let divisor = if self.has_ability(id, "heatproof") {
                    32
                } else {
                    16
                };
                Some(((p.max_hp / divisor).max(1), Status::Burned))
            }
            _ => None,
        };
        if let Some((amount, cause)) = dot {
            events.push(BattleEvent::Message(format!(
                "{} is hurt by {}!",
                p.name,
                if cause == Status::Burned {
                    "its burn"
                } else {
                    "poison"
                }
            )));
            self.damage(id, amount, events);
        }
        if self.get(id).hp == 0 {
            return;
        }
        if let Some(bound) = self.get(id).bound.clone() {
            if bound.turns_left == 0 {
                self.get_mut(id).bound = None;
                events.push(BattleEvent::Message(format!(
                    "{} was freed from {}!",
                    p.name, bound.source
                )));
            } else {
                self.get_mut(id).bound.as_mut().unwrap().turns_left -= 1;
                events.push(BattleEvent::Message(format!(
                    "{} is hurt by {}!",
                    p.name, bound.source
                )));
                self.indirect_damage(id, fractional(p.max_hp, 8, 1), events);
            }
        }
    }
    fn post_turn_weather(&mut self, order: &[ParticipantId], events: &mut Vec<BattleEvent>) {
        let Some(weather) = self.weather.clone() else {
            return;
        };
        let (_, continues, stops) = weather.kind.messages();
        if weather.turns_left == 0 {
            events.push(BattleEvent::Message(stops.into()));
            self.weather = None;
            events.push(BattleEvent::Weather(None));
            return;
        }
        self.weather.as_mut().unwrap().turns_left -= 1;
        events.push(BattleEvent::Message(continues.into()));
        if self.effective_weather().is_none() {
            return;
        }
        for &id in order {
            self.ability_weather_residual(id, weather.kind, events);
        }
        let (spared, name): (&[PokemonType], &str) = match weather.kind {
            WeatherKind::Sandstorm => (
                &[PokemonType::Rock, PokemonType::Ground, PokemonType::Steel],
                "sandstorm",
            ),
            WeatherKind::Hail => (&[PokemonType::Ice], "hail"),
            WeatherKind::Sun | WeatherKind::Rain => return,
        };
        for id in order {
            let p = self.get(*id).clone();
            if p.hp == 0
                || p.types.iter().any(|kind| spared.contains(kind))
                || self.ability_weather_immunity(*id, weather.kind)
            {
                continue;
            }
            events.push(BattleEvent::Message(format!(
                "{} is buffeted by the {name}!",
                p.name
            )));
            self.indirect_damage(*id, fractional(p.max_hp, 16, 1), events);
        }
    }
    pub(crate) fn resolve_outcome(&mut self, events: &mut Vec<BattleEvent>) {
        self.finish_ability_faints(events);
        let alive: Vec<_> = self
            .sides
            .iter()
            .map(|side| side.iter().any(|p| p.hp > 0))
            .collect();
        let winner = match (alive[0], alive[1]) {
            (true, true) => return,
            (true, false) => Some(Side::Allies),
            (false, true) => Some(Side::Foes),
            (false, false) => Some(Side::Allies),
        };
        self.winner = Some(winner);
        events.push(BattleEvent::End { winner });
    }
}

fn fractional(max_hp: u16, denominator: u16, numerator: u16) -> u16 {
    ((u32::from(max_hp) * u32::from(numerator) / u32::from(denominator)) as u16).max(1)
}

impl Battle {
    /// Pursuit consumes the waiting action once and executes it before withdrawal.
    fn intercept_switch(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        while self.get(who).hp > 0 && self.is_active(who) && self.winner.is_none() {
            let Some(index) = self.turn_queue.iter().position(|entry| {
                self.get(entry.actor).hp > 0 && self.is_active(entry.actor)
                    && self.get(entry.actor).carried_by.is_none()
                    && entry.actor.side != who.side && self.adjacent(entry.actor,who)
                    && matches!(&entry.action, SelectedAction::Move(choice)
                        if self.catalog.get(&choice.move_id).is_some_and(|spec| spec.has_flag("intercepts_switch")))
            }) else { break; };
            let entry = self.turn_queue.remove(index).expect("pending interceptor");
            let SelectedAction::Move(mut choice) = entry.action else {
                unreachable!()
            };
            choice.target = who;
            choice.target_position = self.position(who).expect("active switcher");
            self.pending_actors = self
                .turn_queue
                .iter()
                .filter_map(|entry| {
                    matches!(entry.action, SelectedAction::Move(_)).then_some(entry.actor)
                })
                .collect();
            self.hit_reactions
                .retain(|reaction| reaction.owner != entry.actor);
            self.check_sleep(entry.actor, events);
            self.switching_target = Some(who);
            let result = self.execute_move(entry.actor, &choice, events);
            self.switching_target = None;
            result?;
            self.update_berries(events);
            self.get_mut(entry.actor).acted = true;
            self.complete_slot(entry.actor.side, entry.position);
            self.resolve_outcome(events);
        }
        Ok(())
    }
}

impl Battle {
    /// A reflected move is an immediate effect execution, not another selected action:
    /// no additional PP, status-to-act checks, or recursive reflection.
    pub(crate) fn reflect_move(
        &mut self,
        actor: ParticipantId,
        reflector: ParticipantId,
        spec: &MoveSpec,
        events: &mut Vec<BattleEvent>,
    ) -> Result<bool, BattleError> {
        if self.reflecting
            || self.prankster_blocked(actor, reflector, spec)
            || self.priority_blocked(actor, reflector, spec)
            || self.guarded(actor, reflector, spec, false)
            || actor == reflector
            || !spec.has_flag("reflectable")
            || !(self.has_ability(reflector, "magicbounce")
                || self
                    .rules(reflector)
                    .any(|r| matches!(r, Rule::ReflectMoves)))
            || self.get(reflector).hidden().is_some()
            || (self.get(reflector).protected && !spec.has_flag("hazard"))
        {
            return Ok(false);
        }
        events.push(BattleEvent::Message(format!(
            "{} bounced back {}!",
            self.get(reflector).name,
            spec.name
        )));
        let ignored = self.ignored_ability.take();
        let bypass = std::mem::replace(&mut self.ability_bypass, false);
        self.reflecting = true;
        let result = if spec.script.is_some() {
            self.execute_script(reflector, actor, spec, events, ScriptHook::Main)
                .map(|_| ())
        } else {
            (|| {
                if !self.hit_check(reflector, actor, spec, false) {
                    events.push(BattleEvent::Message(format!(
                        "{}'s attack missed!",
                        self.get(reflector).name
                    )));
                } else if !self.get(actor).protected && self.get(actor).hidden().is_none() {
                    for effect in &spec.effects {
                        self.apply_effect(
                            reflector,
                            actor,
                            spec,
                            effect,
                            false,
                            spec.effects.len() == 1,
                            false,
                            events,
                        )?;
                    }
                }
                Ok(())
            })()
        };
        self.reflecting = false;
        self.ignored_ability = ignored;
        self.ability_bypass = bypass;
        result?;
        Ok(true)
    }
}
