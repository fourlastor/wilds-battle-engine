use crate::model::{
    ActionSelection, AdvanceResult, AdvanceStatus, BattleError, BattleEvent, Bound, Choice,
    CombatStat, Condition, ContinuationTarget, HitInfo, LockedMove, ParticipantId, Pokemon,
    PokemonType, Prompt, Scope, Side, Stat, Status, Weather, WeatherKind,
};
use crate::moves::{Accuracy, Category, Effect, MoveCatalog, MoveSpec, ScriptHook, Target};
use crate::rng::{BattleRng, SeededRng};
use crate::type_chart;

pub struct Battle {
    pub(crate) catalog: MoveCatalog,
    pub(crate) sides: [Vec<Pokemon>; 2],
    pub(crate) weather: Option<Weather>,
    pub(crate) rng: Box<dyn BattleRng>,
    pending: Option<Prompt>,
    selections: Vec<(ParticipantId, SelectedAction)>,
    next_prompt_id: u64,
    pub(crate) winner: Option<Option<Side>>,
    pub(crate) turn: u64,
    active_power_multiplier: f32,
    pub(crate) hit_reactions: Vec<HitReaction>,
    pub(crate) side_conditions: [Vec<Condition>; 2],
    pub(crate) field_conditions: Vec<Condition>,
    pub(crate) last_faint_turn: [Option<u64>; 2],
    pub(crate) payout: [u32; 2],
    pub(crate) environment: Option<String>,
    /// The move each Pokémon chose this turn.
    pub(crate) turn_choices: Vec<(ParticipantId, String)>,
    /// A move whose other users should act right away (Round).
    pub(crate) hurry: Option<String>,
    /// Whether the move being used right now failed, missed or had no effect.
    pub(crate) move_failed: bool,
}

#[derive(Clone)]
pub(crate) struct HitReaction {
    pub(crate) owner: ParticipantId,
    pub(crate) move_id: String,
}

/// What a hit was, for the Pokémon it lands on.
pub(crate) struct HitMeta {
    move_id: String,
    category: Category,
    move_type: PokemonType,
    contact: bool,
}
impl HitMeta {
    pub(crate) fn of(spec: &MoveSpec, category: Category, move_type: PokemonType) -> Self {
        Self {
            move_id: spec.id.clone(),
            category,
            move_type,
            contact: spec.has_flag("contact"),
        }
    }
    fn plain(spec: &MoveSpec) -> Self {
        Self::of(spec, spec.category, spec.move_type)
    }
}

/// Everything the damage formula needs to know about one hit.
pub(crate) struct Attack<'a> {
    pub(crate) power: u16,
    pub(crate) high_crit: bool,
    pub(crate) always_crit: bool,
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

fn raw_stat(pokemon: &Pokemon, stat: CombatStat) -> f32 {
    f32::from(match stat {
        CombatStat::Attack => pokemon.attack,
        CombatStat::Defense => pokemon.defense,
        CombatStat::SpAttack => pokemon.sp_attack,
        CombatStat::SpDefense => pokemon.sp_defense,
        CombatStat::Speed => pokemon.speed,
    })
}

#[derive(Clone)]
enum SelectedAction {
    Move(MoveAction),
    Recharge,
}

#[derive(Clone)]
struct MoveAction {
    move_id: String,
    target: ParticipantId,
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
            for id in &pokemon.moves {
                let spec = catalog.get(id).expect("validated catalog");
                pokemon.move_pp.entry(id.clone()).or_insert(spec.pp);
            }
        }
        Ok(Self {
            catalog,
            sides,
            weather: None,
            rng: Box::new(rng),
            pending: None,
            selections: Vec::new(),
            next_prompt_id: 1,
            winner: None,
            turn: 0,
            active_power_multiplier: 1.0,
            hit_reactions: Vec::new(),
            side_conditions: [Vec::new(), Vec::new()],
            field_conditions: Vec::new(),
            last_faint_turn: [None, None],
            payout: [0, 0],
            environment: None,
            turn_choices: Vec::new(),
            hurry: None,
            move_failed: false,
        })
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
    /// Money a side's moves have earned so far (Pay Day).
    pub fn payout(&self, side: Side) -> u32 {
        self.payout[side.index()]
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
        self.is_trapped(id) || self.get(id).bound.is_some()
    }
    pub(crate) fn get(&self, id: ParticipantId) -> &Pokemon {
        &self.sides[id.side.index()][id.index]
    }
    pub(crate) fn get_mut(&mut self, id: ParticipantId) -> &mut Pokemon {
        &mut self.sides[id.side.index()][id.index]
    }
    pub(crate) fn actors(&self) -> Vec<ParticipantId> {
        Side::ALL
            .into_iter()
            .flat_map(|side| {
                (0..self.sides[side.index()].len())
                    .filter(move |&index| self.sides[side.index()][index].hp > 0)
                    .map(move |index| ParticipantId { side, index })
            })
            .collect()
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
            let targets: Vec<_> = if let Some(lock) = &pokemon.locked_move
                && lock.charging
            {
                vec![lock.target]
            } else {
                let allies = |with_user: bool| -> Vec<ParticipantId> {
                    (0..self.sides[actor.side.index()].len())
                        .filter(|&index| self.sides[actor.side.index()][index].hp > 0)
                        .filter(|&index| with_user || index != actor.index)
                        .map(|index| ParticipantId {
                            side: actor.side,
                            index,
                        })
                        .collect()
                };
                match spec.target {
                    Target::User | Target::AllAllies | Target::UserAndAllies => vec![actor],
                    Target::Field
                    | Target::AllOthers
                    | Target::RandomOpponent
                    | Target::AllOpponents
                    | Target::All => self.sides[actor.side.opposite().index()]
                        .iter()
                        .position(|p| p.hp > 0)
                        .map(|index| ParticipantId {
                            side: actor.side.opposite(),
                            index,
                        })
                        .into_iter()
                        .collect(),
                    Target::Selected => (0..self.sides[actor.side.opposite().index()].len())
                        .filter(|&index| self.sides[actor.side.opposite().index()][index].hp > 0)
                        .map(|index| ParticipantId {
                            side: actor.side.opposite(),
                            index,
                        })
                        .collect(),
                    Target::Ally => allies(false),
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
                        move_id: continuation.move_id,
                        target: continuation.target,
                    }),
                ));
                continue;
            }
            let choices = self.legal_choices(actor);
            if choices.is_empty() {
                let target = self.sides[actor.side.opposite().index()]
                    .iter()
                    .position(|pokemon| pokemon.hp > 0)
                    .ok_or_else(|| BattleError::InvalidSetup("Struggle has no target".into()))?;
                self.selections.push((
                    actor,
                    SelectedAction::Move(MoveAction {
                        move_id: "struggle".into(),
                        target: ParticipantId {
                            side: actor.side.opposite(),
                            index: target,
                        },
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
                    .push((actor, SelectedAction::Move(Self::move_action(&choices[0]))));
                continue;
            }
            let prompt = Prompt {
                id: self.next_prompt_id,
                actor,
                choices,
            };
            self.next_prompt_id += 1;
            self.pending = Some(prompt.clone());
            return Ok(AdvanceResult {
                events: Vec::new(),
                status: AdvanceStatus::Awaiting(prompt),
            });
        }
        let mut events = self.resolve_turn()?;
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
        self.selections.push((
            prompt.actor,
            SelectedAction::Move(Self::move_action(choice)),
        ));
        self.pending = None;
        Ok(())
    }
    fn move_action(choice: &Choice) -> MoveAction {
        match choice {
            Choice::UseMove {
                move_id, target, ..
            } => MoveAction {
                move_id: move_id.clone(),
                target: *target,
            },
        }
    }
    fn resolve_turn(&mut self) -> Result<Vec<BattleEvent>, BattleError> {
        self.turn += 1;
        let mut ordered: Vec<_> = std::mem::take(&mut self.selections)
            .into_iter()
            .map(|(actor, choice)| {
                let priority = match &choice {
                    SelectedAction::Move(choice) => {
                        self.catalog
                            .get(&choice.move_id)
                            .expect("validated choice")
                            .priority
                    }
                    SelectedAction::Recharge => 0,
                };
                let speed = self.stat_value(actor, CombatStat::Speed) as i32;
                (actor, choice, priority, speed, self.rng.next_u64())
            })
            .collect();
        ordered.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)).then(b.4.cmp(&a.4)));
        for pokemon in self.sides.iter_mut().flatten() {
            pokemon.acted = false;
            pokemon.damage_taken = 0;
            pokemon.hurt_this_turn = false;
            pokemon.last_hit = None;
        }
        self.turn_choices = ordered
            .iter()
            .filter_map(|(actor, choice, ..)| match choice {
                SelectedAction::Move(choice) => Some((*actor, choice.move_id.clone())),
                SelectedAction::Recharge => None,
            })
            .collect();
        self.hurry = None;
        let mut events = Vec::new();
        // Moves that get ready before anyone acts (Focus Punch, Beak Blast).
        for (actor, choice, ..) in &ordered {
            let SelectedAction::Move(choice) = choice else {
                continue;
            };
            let spec = self.catalog.get(&choice.move_id).expect("validated choice");
            if spec.on_turn_start.is_some() && self.get(*actor).hp > 0 {
                let spec = spec.clone();
                self.execute_script(
                    *actor,
                    choice.target,
                    &spec,
                    &mut events,
                    ScriptHook::TurnStart,
                )?;
            }
        }
        let mut next = 0;
        while next < ordered.len() {
            if self.winner.is_some() {
                break;
            }
            let actor = ordered[next].0;
            let choice = ordered[next].1.clone();
            next += 1;
            self.hit_reactions
                .retain(|reaction| reaction.owner != actor);
            self.check_sleep(actor, &mut events);
            match &choice {
                SelectedAction::Move(choice) => self.execute_move(actor, choice, &mut events)?,
                SelectedAction::Recharge => {
                    self.get_mut(actor).recharging = false;
                    if self.get(actor).hp > 0 {
                        events.push(BattleEvent::Message(format!(
                            "{} must recharge!",
                            self.get(actor).name
                        )));
                    }
                }
            }
            self.get_mut(actor).acted = true;
            self.resolve_outcome(&mut events);
            if let Some(move_id) = self.hurry.take() {
                // Everyone else who chose that move goes next, in their usual order.
                let (now, later): (Vec<_>, Vec<_>) = ordered.drain(next..).partition(|entry| {
                    matches!(&entry.1, SelectedAction::Move(action) if action.move_id == move_id)
                });
                ordered.extend(now);
                ordered.extend(later);
            }
        }
        if self.winner.is_none() {
            for (actor, _, _, _, _) in &ordered {
                self.post_turn_status(*actor, &mut events);
                self.resolve_outcome(&mut events);
                if self.winner.is_some() {
                    break;
                }
            }
        }
        if self.winner.is_none() {
            self.post_turn_weather(&ordered, &mut events);
            self.resolve_outcome(&mut events);
        }
        if self.winner.is_none() {
            let order: Vec<ParticipantId> = ordered.iter().map(|entry| entry.0).collect();
            self.tick_conditions(&order, &mut events);
            self.resolve_outcome(&mut events);
        }
        for pokemon in self.sides.iter_mut().flatten() {
            pokemon.protected = false;
            pokemon.flinched = false;
        }
        Ok(events)
    }
    fn check_sleep(&mut self, id: ParticipantId, events: &mut Vec<BattleEvent>) {
        if self.get(id).hp == 0 || self.get(id).status != Some(Status::Asleep) {
            return;
        }
        if self.get(id).status_turns == 0 {
            let name = self.get(id).name.clone();
            self.get_mut(id).status = None;
            events.push(BattleEvent::Status {
                target: id,
                status: None,
            });
            events.push(BattleEvent::Message(format!("{name} woke up!")));
        } else {
            self.get_mut(id).status_turns -= 1;
        }
    }
    fn execute_move(
        &mut self,
        actor: ParticipantId,
        choice: &MoveAction,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        self.move_failed = false;
        let used = self.perform_move(actor, choice, events)?;
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
    /// Uses a move. Returns false if the Pokémon could not act at all.
    fn perform_move(
        &mut self,
        actor: ParticipantId,
        choice: &MoveAction,
        events: &mut Vec<BattleEvent>,
    ) -> Result<bool, BattleError> {
        if self.get(actor).hp == 0 {
            return Ok(false);
        }
        let spec = self
            .catalog
            .get(&choice.move_id)
            .expect("validated choice")
            .clone();
        // A Pokémon that hid through a script comes back when it next acts.
        if self.get(actor).semi_invulnerable && self.get(actor).locked_move.is_none() {
            self.get_mut(actor).semi_invulnerable = false;
        }
        self.active_power_multiplier = self
            .get_mut(actor)
            .next_move_power_boost
            .take()
            .filter(|boost| boost.move_type == spec.move_type && spec.category != Category::Status)
            .map_or(1.0, |boost| boost.multiplier);
        let user_name = self.get(actor).name.clone();
        let forced_script = self
            .get(actor)
            .script_continuation
            .as_ref()
            .is_some_and(|lock| lock.move_id == spec.id);
        if self.get(actor).status == Some(Status::Frozen) && spec.usable_while_frozen {
            self.get_mut(actor).status = None;
            events.push(BattleEvent::Message(format!("{user_name} thawed out!")));
            events.push(BattleEvent::Status {
                target: actor,
                status: None,
            });
        }
        let status = self.get(actor).status;
        match status {
            Some(Status::Frozen) if self.rng.chance(0.2) => {
                self.get_mut(actor).status = None;
                events.push(BattleEvent::Message(format!("{user_name} thawed out!")));
                events.push(BattleEvent::Status {
                    target: actor,
                    status: None,
                });
            }
            Some(Status::Frozen) => {
                events.push(BattleEvent::Message(format!(
                    "{user_name} is frozen solid!"
                )));
                self.interrupt_script(actor, choice.target, &spec, events)?;
                return Ok(false);
            }
            Some(Status::Paralyzed) if self.rng.chance(0.25) => {
                events.push(BattleEvent::Message(format!(
                    "{user_name} is paralyzed! It can't move!"
                )));
                self.clear_consecutive(actor);
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
        if self.get(actor).flinched {
            events.push(BattleEvent::Message(format!("{user_name} flinched!")));
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
                    let p = self.get(actor).clone();
                    let attack = p.effective_stat(CombatStat::Attack);
                    let defense = p.effective_stat(CombatStat::Defense).max(1.0);
                    let amount = (((2.0 * p.level as f32 * 0.2 + 2.0) * 40.0 * attack
                        / defense
                        / 50.0
                        + 2.0)
                        .round() as u16)
                        .max(1);
                    self.damage(actor, amount, events);
                    self.clear_consecutive(actor);
                    self.interrupt_script(actor, choice.target, &spec, events)?;
                    return Ok(false);
                }
            }
        }
        {
            let pokemon = self.get_mut(actor);
            pokemon.last_used = Some(spec.id.clone());
            pokemon.moves_used.insert(spec.id.clone());
        }
        let streak = if self.get(actor).last_move.as_deref() == Some(&spec.id) {
            self.get(actor).consecutive_count
        } else {
            0
        };
        if spec.effects.iter().any(|e| matches!(e, Effect::Protect))
            && !self.rng.chance((1.0f32 / 3.0).powi(streak as i32))
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
        let mut target = choice.target;
        // Forced turns pick a new opponent too when the script asked for one. This shares the
        // roll below, so a move that is random in both ways still rolls once.
        let random_continuation = forced_script
            && self
                .get(actor)
                .script_continuation
                .as_ref()
                .is_some_and(|lock| lock.target_policy == ContinuationTarget::RandomOpponent);
        if spec.target == Target::RandomOpponent || random_continuation {
            let foes: Vec<_> = self.sides[actor.side.opposite().index()]
                .iter()
                .enumerate()
                .filter(|(_, pokemon)| pokemon.hp > 0)
                .map(|(index, _)| ParticipantId {
                    side: actor.side.opposite(),
                    index,
                })
                .collect();
            if foes.is_empty() {
                return Ok(false);
            }
            target = foes[self.rng.index(foes.len())];
        }
        if self.get(target).hp == 0
            && matches!(
                spec.target,
                Target::Selected | Target::Ally | Target::AnyOther
            )
        {
            // The chosen Pokémon is gone: another one on its side takes its place, never the user.
            let Some(index) =
                self.sides[target.side.index()]
                    .iter()
                    .enumerate()
                    .position(|(index, p)| {
                        p.hp > 0 && (target.side != actor.side || index != actor.index)
                    })
            else {
                return Ok(false);
            };
            target.index = index;
        }
        if self.get(target).hp == 0 && spec.target == Target::UserOrAlly {
            target = actor;
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
        if !spec.auto_only
            && !forced_script
            && self.get(actor).locked_move.is_none()
            && let Some(pp) = self.get_mut(actor).move_pp.get_mut(&spec.id)
        {
            *pp = pp.saturating_sub(1);
        }
        if spec.script.is_some() {
            if !spec.manual_announce {
                events.push(BattleEvent::Message(format!(
                    "{user_name} used {}!",
                    spec.name
                )));
            }
            let hit = self.execute_script(actor, target, &spec, events, ScriptHook::Main)?;
            if let Some(lock) = self.get_mut(actor).script_continuation.as_mut() {
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
        let second_turn = self
            .get(actor)
            .locked_move
            .as_ref()
            .is_some_and(|lock| lock.charging);
        let first_charge = !second_turn && spec.effects.iter().any(|e| matches!(e, Effect::TwoTurn { skip_in_sun, .. } if !skip_in_sun || self.weather.as_ref().is_none_or(|w| w.kind != WeatherKind::Sun)));
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
        if !(first_charge
            || self.always_hits(actor, target)
            || self.hit_check(actor, target, &spec, false))
        {
            events.push(BattleEvent::Message(format!(
                "{user_name}'s attack missed!"
            )));
            self.move_failed = true;
            self.clear_consecutive(actor);
            return Ok(true);
        }
        if self.get(target).protected && actor != target {
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
            && !(spec.hits_hidden.contains(&place) || self.always_hits(actor, target))
        {
            events.push(BattleEvent::Message(format!(
                "{} avoided the attack!",
                self.get(target).name
            )));
            self.move_failed = true;
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
        let change = self.get_mut(recipient).change_stage(stat, delta);
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
        match spec.accuracy {
            Accuracy::Always => true,
            Accuracy::OneHitKnockout => {
                let user = self.get(actor);
                let defender = self.get(target);
                if user.level < defender.level {
                    false
                } else {
                    self.rng
                        .chance(0.3 + 0.01 * f32::from(user.level - defender.level))
                }
            }
            Accuracy::Chance(value) => self.hit_check_chance(actor, target, value, ignore_evasion),
        }
    }
    pub(crate) fn hit_check_chance(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        value: f32,
        ignore_evasion: bool,
    ) -> bool {
        let evasion = if ignore_evasion {
            1.0
        } else {
            self.get(target).evasion_factor()
        };
        let probability = value * self.get(actor).accuracy_factor() * evasion;
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
        match effect {
            Effect::Damage {
                power,
                recoil,
                drain,
                high_crit,
                always_crit,
            } => {
                let mut details = Vec::new();
                if let Some(amount) = self.calculate_damage(
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
                    self.damage_from_move(actor, target, amount, &HitMeta::plain(spec), events)?;
                    events.extend(details);
                    if let Some(fraction) = recoil {
                        let recoil = ((amount as f32 * fraction).floor() as u16).max(1);
                        self.damage(actor, recoil, events);
                        events.push(BattleEvent::Message(format!(
                            "{} was damaged by the recoil!",
                            self.get(actor).name
                        )));
                    }
                    if let Some(fraction) = drain {
                        if self.get(actor).hp < self.get(actor).max_hp {
                            self.heal(
                                actor,
                                ((amount as f32 * fraction).floor() as u16).max(1),
                                events,
                            );
                        }
                        events.push(BattleEvent::Message(format!(
                            "{} had its energy drained!",
                            self.get(target).name
                        )));
                    }
                }
            }
            Effect::MultiHit { power, min, max } => {
                if self.effectiveness(target, spec.move_type) == 0.0 {
                    self.immune_message(target, events);
                    return Ok(());
                }
                let hits = self.rng.range(*min, *max);
                // C# calculates all hits from the same pre-hit target snapshot.
                let mut amounts = Vec::new();
                let mut details = Vec::new();
                for _ in 0..hits {
                    let mut hit_details = Vec::new();
                    if let Some(amount) = self.calculate_damage(
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
                        amounts.push(amount);
                        if details.is_empty() {
                            details.extend(hit_details.iter().filter(|event| !matches!(event, BattleEvent::Message(message) if message == "A critical hit!")).cloned());
                        }
                        if hit_details.iter().any(|event| matches!(event, BattleEvent::Message(message) if message == "A critical hit!"))
                            && !details.iter().any(|event| matches!(event, BattleEvent::Message(message) if message == "A critical hit!")) {
                            details.push(BattleEvent::Message("A critical hit!".into()));
                        }
                    }
                }
                for amount in amounts {
                    self.damage_from_move(actor, target, amount, &HitMeta::plain(spec), events)?;
                }
                events.extend(details);
                events.push(BattleEvent::Message(format!("Hit {hits} times!")));
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
                    if *delta < 0 && recipient != actor && self.blocks_stat_drops(recipient) {
                        events.push(BattleEvent::Message(format!(
                            "{}'s stats were not lowered!",
                            self.get(recipient).name
                        )));
                        break;
                    }
                    self.change_stat(recipient, *stat, *delta, events);
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
                if target != actor && self.blocks_confusion(target) {
                    return Ok(());
                }
                if self.get(target).confused_turns.is_none() {
                    let turns = self.rng.range(1, 4);
                    self.get_mut(target).confused_turns = Some(turns);
                    events.push(BattleEvent::Message(format!(
                        "{} became confused!",
                        self.get(target).name
                    )));
                } else if single_effect {
                    events.push(BattleEvent::Message(format!(
                        "{} was already confused!",
                        self.get(target).name
                    )));
                }
            }
            Effect::Flinch(chance) => {
                let chance = self.effect_chance(actor, *chance);
                if self.rng.chance(chance) && self.effectiveness(target, spec.move_type) != 0.0 {
                    self.get_mut(target).flinched = true;
                }
            }
            Effect::Bind => {
                if self.effectiveness(target, spec.move_type) != 0.0
                    && self.get(target).bound.is_none()
                {
                    let turns = self.rng.range(4, 5);
                    self.get_mut(target).bound = Some(Bound {
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
                    || *skip_in_sun
                        && self
                            .weather
                            .as_ref()
                            .is_some_and(|w| w.kind == WeatherKind::Sun))
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
                    if let Some(amount) = self.calculate_damage(
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
                            amount,
                            &HitMeta::plain(spec),
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
                if let Some(amount) = self.calculate_damage(
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
                    self.damage_from_move(actor, target, amount, &HitMeta::plain(spec), events)?;
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
                    self.damage_from_move(actor, target, amount, &HitMeta::plain(spec), events)?;
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
        type_chart::multiplier(move_type, &self.defending_types(target, move_type))
    }
    /// How well a hit does against a Pokémon's types, with the hit's own exceptions.
    pub(crate) fn matchup(&self, target: ParticipantId, attack: &Attack) -> f32 {
        let against = |move_type: PokemonType, exceptions: &[(PokemonType, f32)]| -> f32 {
            if exceptions.is_empty() {
                return self.effectiveness(target, move_type);
            }
            self.defending_types(target, move_type)
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
                .product()
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
        if self.effectiveness(target, spec.move_type) == 0.0 {
            self.immune_message(target, events);
        } else {
            self.damage_from_move(actor, target, amount, &HitMeta::plain(spec), events)?;
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
    ) -> Option<u16> {
        let attack = Attack {
            power,
            high_crit,
            always_crit,
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
            .map(|roll| roll.amount)
    }
    /// The damage formula. Returns nothing if the target is immune.
    pub(crate) fn calculate(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        attack: &Attack,
        events: &mut Vec<BattleEvent>,
    ) -> Option<Roll> {
        let defender = self.get(target).clone();
        let attacker = self.get(actor).clone();
        let source = self.get(attack.attack_from).clone();
        let move_type = attack.move_type;
        let typeless = attack.typeless;
        let crit_stage = attacker
            .crit_stage
            .saturating_add(u8::from(attack.high_crit));
        let chance = match crit_stage {
            0 => 1.0 / 24.0,
            1 => 1.0 / 8.0,
            2 => 0.5,
            _ => 1.0,
        };
        let crit = attack.always_crit || self.rng.chance(chance);
        let effectiveness = if typeless {
            1.0
        } else {
            self.matchup(target, attack)
        };
        if effectiveness == 0.0 {
            self.immune_message(target, events);
            return None;
        }
        let (atk_name, def_name) = match attack.category {
            Category::Physical => (CombatStat::Attack, CombatStat::Defense),
            _ => (CombatStat::SpAttack, CombatStat::SpDefense),
        };
        let atk_name = attack.attack_stat.unwrap_or(atk_name);
        let def_name = attack.defense_stat.unwrap_or(def_name);
        let attack_value = if crit && source.stage(atk_name.into()) < 0 {
            raw_stat(&source, atk_name)
        } else {
            self.stat_value(attack.attack_from, atk_name)
        };
        let mut defense = if attack.ignore_stages {
            (raw_stat(&defender, def_name) * self.stat_factor(target, def_name)).trunc()
        } else if crit && defender.stage(def_name.into()) > 0 {
            raw_stat(&defender, def_name)
        } else {
            self.stat_value(target, def_name)
        };
        let weather = self.weather.as_ref().map(|weather| weather.kind);
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
        let adjusted_power = f32::from(attack.power)
            * 2f32.powi(attack.consecutive_boost as i32)
            * self.active_power_multiplier;
        let mut damage =
            (2.0 * f32::from(attacker.level) * 0.2 + 2.0) * adjusted_power * attack_value
                / defense
                / 50.0
                + 2.0;
        if attacker.status == Some(Status::Burned) && attack.category == Category::Physical {
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
        damage *= effectiveness;
        if !typeless && self.types_of(actor).contains(&move_type) {
            damage *= 1.5;
        }
        if crit {
            damage *= 1.5;
        }
        let dealt = self.damage_dealt_factor(actor, attack.category, move_type);
        let taken = self.damage_taken_factor(target, attack.category, move_type, crit);
        if dealt != 1.0 || taken != 1.0 {
            damage *= dealt * taken;
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
    ) -> Result<(), BattleError> {
        let before = self.get(target).hp;
        let mut amount = amount;
        if amount >= before && before > 0 && self.endures(target) {
            amount = before - 1;
            events.push(BattleEvent::Message(format!(
                "{} endured the hit!",
                self.get(target).name
            )));
            if amount == 0 {
                return Ok(());
            }
        }
        self.damage(target, amount, events);
        let lost = before - self.get(target).hp;
        if lost > 0 {
            let pokemon = self.get_mut(target);
            pokemon.damage_taken = pokemon.damage_taken.saturating_add(lost);
            pokemon.last_hit = Some(HitInfo {
                attacker: actor,
                move_id: meta.move_id.clone(),
                category: meta.category,
                move_type: meta.move_type,
                damage: lost,
                contact: meta.contact,
            });
        }
        if self.get(target).hp == 0 || self.get(target).hp == before {
            return Ok(());
        }
        let reactions: Vec<_> = self
            .hit_reactions
            .iter()
            .filter(|reaction| reaction.owner == target)
            .cloned()
            .collect();
        for reaction in reactions {
            let spec = self
                .catalog
                .get(&reaction.move_id)
                .expect("registered reaction has a move")
                .clone();
            self.execute_script(target, actor, &spec, events, ScriptHook::Hit)?;
        }
        Ok(())
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
            self.get_mut(target).conditions.clear();
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
            Some(Status::Poisoned) => Some((fractional(p.max_hp, 8, 1), Status::Poisoned)),
            Some(Status::BadlyPoisoned) => Some((
                fractional(p.max_hp, 16, p.status_turns as u16),
                Status::BadlyPoisoned,
            )),
            Some(Status::Burned) => Some((fractional(p.max_hp, 8, 1), Status::Burned)),
            _ => None,
        };
        if let Some((amount, cause)) = dot {
            // C# increments StackedStatusTurns for ordinary poison and burn too.
            self.get_mut(id).status_turns = p.status_turns.saturating_add(1);
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
                self.damage(id, fractional(p.max_hp, 8, 1), events);
            }
        }
    }
    fn post_turn_weather(
        &mut self,
        order: &[(ParticipantId, SelectedAction, i8, i32, u64)],
        events: &mut Vec<BattleEvent>,
    ) {
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
        let (spared, name): (&[PokemonType], &str) = match weather.kind {
            WeatherKind::Sandstorm => (
                &[PokemonType::Rock, PokemonType::Ground, PokemonType::Steel],
                "sandstorm",
            ),
            WeatherKind::Hail => (&[PokemonType::Ice], "hail"),
            WeatherKind::Sun | WeatherKind::Rain => return,
        };
        for (id, _, _, _, _) in order {
            let p = self.get(*id).clone();
            if p.hp == 0 || p.types.iter().any(|kind| spared.contains(kind)) {
                continue;
            }
            events.push(BattleEvent::Message(format!(
                "{} is buffeted by the {name}!",
                p.name
            )));
            self.damage(*id, fractional(p.max_hp, 16, 1), events);
        }
    }
    pub(crate) fn resolve_outcome(&mut self, events: &mut Vec<BattleEvent>) {
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
