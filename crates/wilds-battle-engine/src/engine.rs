use crate::model::{
    ActionSelection, AdvanceResult, AdvanceStatus, AppliedStatus, BattleError, BattleEvent, Bound,
    Choice, CombatStat, ContinuationTarget, LockedMove, NextMovePowerBoost, ParticipantId, Pokemon,
    PokemonType, Prompt, ScriptContinuation, Side, Stat, Status, Weather, WeatherKind,
};
use crate::moves::{
    Accuracy, Category, Effect, MoveCatalog, MoveSpec, ScriptContext, ScriptHook, ScriptOperation,
    Target,
};
use crate::rng::{BattleRng, SeededRng};
use crate::type_chart;
use mlua::Value;
use mlua::thread::ThreadStatus;

pub struct Battle {
    catalog: MoveCatalog,
    sides: [Vec<Pokemon>; 2],
    weather: Option<Weather>,
    rng: Box<dyn BattleRng>,
    pending: Option<Prompt>,
    selections: Vec<(ParticipantId, SelectedAction)>,
    next_prompt_id: u64,
    winner: Option<Option<Side>>,
    turn: u64,
    active_power_multiplier: f32,
    hit_reactions: Vec<HitReaction>,
}

#[derive(Clone)]
struct HitReaction {
    owner: ParticipantId,
    move_id: String,
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
    fn get(&self, id: ParticipantId) -> &Pokemon {
        &self.sides[id.side.index()][id.index]
    }
    fn get_mut(&mut self, id: ParticipantId) -> &mut Pokemon {
        &mut self.sides[id.side.index()][id.index]
    }
    fn actors(&self) -> Vec<ParticipantId> {
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
                match spec.target {
                    Target::User => vec![actor],
                    Target::Field | Target::AllOthers | Target::RandomOpponent => self.sides
                        [actor.side.opposite().index()]
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
                let target = match continuation.target_policy {
                    ContinuationTarget::SameTarget => continuation.target,
                    ContinuationTarget::RandomOpponent => continuation.target,
                };
                self.selections.push((
                    actor,
                    SelectedAction::Move(MoveAction {
                        move_id: continuation.move_id,
                        target,
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
                let speed = self.get(actor).effective_stat(CombatStat::Speed) as i32;
                (actor, choice, priority, speed, self.rng.next_u64())
            })
            .collect();
        ordered.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)).then(b.4.cmp(&a.4)));
        let mut events = Vec::new();
        for (actor, choice, _, _, _) in &ordered {
            if self.winner.is_some() {
                break;
            }
            self.hit_reactions
                .retain(|reaction| reaction.owner != *actor);
            self.check_sleep(*actor, &mut events);
            match choice {
                SelectedAction::Move(choice) => self.execute_move(*actor, choice, &mut events)?,
                SelectedAction::Recharge => {
                    self.get_mut(*actor).recharging = false;
                    if self.get(*actor).hp > 0 {
                        events.push(BattleEvent::Message(format!(
                            "{} must recharge!",
                            self.get(*actor).name
                        )));
                    }
                }
            }
            self.resolve_outcome(&mut events);
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
        if self.get(actor).hp == 0 {
            return Ok(());
        }
        let spec = self
            .catalog
            .get(&choice.move_id)
            .expect("validated choice")
            .clone();
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
                return Ok(());
            }
            Some(Status::Paralyzed) if self.rng.chance(0.25) => {
                events.push(BattleEvent::Message(format!(
                    "{user_name} is paralyzed! It can't move!"
                )));
                self.clear_consecutive(actor);
                self.interrupt_script(actor, choice.target, &spec, events)?;
                return Ok(());
            }
            Some(Status::Asleep) => {
                events.push(BattleEvent::Message(format!("{user_name} is fast asleep.")));
                self.clear_consecutive(actor);
                self.interrupt_script(actor, choice.target, &spec, events)?;
                return Ok(());
            }
            _ => {}
        }
        if self.get(actor).flinched {
            events.push(BattleEvent::Message(format!("{user_name} flinched!")));
            self.get_mut(actor).flinched = false;
            self.clear_consecutive(actor);
            self.interrupt_script(actor, choice.target, &spec, events)?;
            return Ok(());
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
                    return Ok(());
                }
            }
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
            self.clear_consecutive(actor);
            return Ok(());
        }
        let mut target = choice.target;
        if spec.target == Target::RandomOpponent {
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
                return Ok(());
            }
            target = foes[self.rng.index(foes.len())];
        }
        if self.get(target).hp == 0 && spec.target == Target::Selected {
            if let Some(index) = self.sides[target.side.index()]
                .iter()
                .position(|p| p.hp > 0)
            {
                target.index = index;
            } else {
                return Ok(());
            }
        }
        if spec.fail_on_full_hp && self.get(target).hp == self.get(target).max_hp {
            events.push(BattleEvent::Message(format!(
                "{user_name} used {}!",
                spec.name
            )));
            events.push(BattleEvent::Message("But it failed!".into()));
            return Ok(());
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
                return Ok(());
            }
            let pokemon = self.get_mut(actor);
            pokemon.consecutive_count = if pokemon.last_move.as_deref() == Some(&spec.id) {
                pokemon.consecutive_count.saturating_add(1)
            } else {
                1
            };
            pokemon.last_move = Some(spec.id);
            return Ok(());
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
        if !first_charge && !self.hit_check(actor, target, &spec) {
            events.push(BattleEvent::Message(format!(
                "{user_name}'s attack missed!"
            )));
            self.clear_consecutive(actor);
            return Ok(());
        }
        if self.get(target).protected && actor != target {
            events.push(BattleEvent::Message(format!(
                "{} protected itself!",
                self.get(target).name
            )));
            return Ok(());
        }
        if !first_charge && self.get(target).semi_invulnerable && actor != target {
            events.push(BattleEvent::Message(format!(
                "{} avoided the attack!",
                self.get(target).name
            )));
            return Ok(());
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
        Ok(())
    }
    fn execute_script(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        events: &mut Vec<BattleEvent>,
        hook: ScriptHook,
    ) -> Result<bool, BattleError> {
        let continuation = self.get(actor).script_continuation.as_ref();
        let (thread, input) = self.catalog.start_script(
            spec,
            ScriptContext {
                user_hp: self.get(actor).hp,
                user_name: self.get(actor).name.clone(),
                target_hp: self.get(target).hp,
                target_status: self.get(target).status,
                weather: self.weather.as_ref().map(|weather| weather.kind),
                turn: continuation.map_or(1, |lock| lock.turn),
                total_turns: continuation.map(|lock| lock.total_turns),
            },
            hook,
        )?;
        let mut yielded = thread.resume::<Value>(input)?;
        let mut any_hit = false;
        let mut operations = 0;
        while thread.status() == ThreadStatus::Resumable {
            operations += 1;
            if operations > 64 {
                return Err(BattleError::InvalidSetup(format!(
                    "script for {} exceeded 64 operations",
                    spec.id
                )));
            }
            let operation = match yielded {
                Value::UserData(ref userdata) => userdata.borrow::<ScriptOperation>()?.clone(),
                _ => {
                    return Err(BattleError::InvalidSetup(format!(
                        "script for {} yielded a value outside the battle API",
                        spec.id
                    )));
                }
            };
            if hook == ScriptHook::Hit
                && !matches!(
                    operation,
                    ScriptOperation::Message(_) | ScriptOperation::ChangeSelfStat { .. }
                )
            {
                return Err(BattleError::InvalidSetup(format!(
                    "hit hook for {} yielded an unsupported operation",
                    spec.id
                )));
            }
            let response = match operation {
                ScriptOperation::WatchHitsUntilNextAction => {
                    if spec.on_hit.is_none() {
                        return Err(BattleError::InvalidSetup(format!(
                            "move {} has no hit hook",
                            spec.id
                        )));
                    }
                    self.hit_reactions
                        .retain(|reaction| reaction.owner != actor || reaction.move_id != spec.id);
                    self.hit_reactions.push(HitReaction {
                        owner: actor,
                        move_id: spec.id.clone(),
                    });
                    Value::Nil
                }
                ScriptOperation::ForceMove {
                    total_turns,
                    target_policy,
                } => {
                    if self.get(actor).script_continuation.is_some() {
                        return Err(BattleError::InvalidSetup("move is already forced".into()));
                    }
                    self.get_mut(actor).script_continuation = Some(ScriptContinuation {
                        move_id: spec.id.clone(),
                        target,
                        turn: 1,
                        total_turns,
                        target_policy,
                    });
                    Value::Nil
                }
                ScriptOperation::BreakSequence => {
                    self.get_mut(actor).script_continuation = None;
                    Value::Nil
                }
                ScriptOperation::RandomInt { min, max } => {
                    Value::Integer(self.rng.range(min, max) as i64)
                }
                ScriptOperation::Message(message) => {
                    events.push(BattleEvent::Message(message));
                    Value::Nil
                }
                ScriptOperation::Announce => {
                    events.push(BattleEvent::Message(format!(
                        "{} used {}!",
                        self.get(actor).name,
                        spec.name
                    )));
                    Value::Nil
                }
                ScriptOperation::ConfuseSelf => {
                    if self.get(actor).hp > 0 && self.get(actor).confused_turns.is_none() {
                        let turns = self.rng.range(1, 4);
                        self.get_mut(actor).confused_turns = Some(turns);
                        events.push(BattleEvent::Message(format!(
                            "{} became confused!",
                            self.get(actor).name
                        )));
                    }
                    Value::Nil
                }
                ScriptOperation::RecoilMaxHp(fraction) => {
                    let max_hp = self.get(actor).max_hp;
                    let amount = ((f32::from(max_hp) * fraction).round() as u16).max(1);
                    self.damage(actor, amount, events);
                    events.push(BattleEvent::Message(format!(
                        "{} was damaged by the recoil!",
                        self.get(actor).name
                    )));
                    Value::Nil
                }
                ScriptOperation::ChangeSelfStat { stat, stages } => {
                    self.change_stat(actor, stat, stages, events);
                    Value::Nil
                }
                ScriptOperation::BoostNextMove {
                    move_type,
                    multiplier,
                } => {
                    self.get_mut(actor).next_move_power_boost = Some(NextMovePowerBoost {
                        move_type,
                        multiplier,
                    });
                    Value::Nil
                }
                ScriptOperation::Fail => {
                    events.push(BattleEvent::Message("But it failed!".into()));
                    break;
                }
                ScriptOperation::FaintUser => {
                    let remaining = self.get(actor).hp;
                    self.damage(actor, remaining, events);
                    Value::Nil
                }
                ScriptOperation::Recharge => {
                    if self.get(actor).hp > 0 {
                        self.get_mut(actor).recharging = true;
                    }
                    Value::Nil
                }
                ScriptOperation::Damage {
                    power,
                    accuracy,
                    drain,
                    min_target_hp,
                    typeless,
                } => {
                    let targets = if spec.target == Target::AllOthers {
                        self.actors()
                            .into_iter()
                            .filter(|id| *id != actor)
                            .collect()
                    } else {
                        vec![target]
                    };
                    let mut operation_hit = false;
                    let mut total_damage = 0u16;
                    for recipient in targets {
                        if self.get(recipient).hp == 0 {
                            continue;
                        }
                        if self.get(recipient).protected {
                            events.push(BattleEvent::Message(format!(
                                "{} protected itself!",
                                self.get(recipient).name
                            )));
                            continue;
                        }
                        if self.get(recipient).semi_invulnerable {
                            events.push(BattleEvent::Message(format!(
                                "{} avoided the attack!",
                                self.get(recipient).name
                            )));
                            continue;
                        }
                        let hit = if let Some(value) = accuracy {
                            self.hit_check_chance(actor, recipient, value)
                        } else {
                            self.hit_check(actor, recipient, spec)
                        };
                        if !hit {
                            events.push(BattleEvent::Message(format!(
                                "{}'s attack missed!",
                                self.get(actor).name
                            )));
                            continue;
                        }
                        let mut details = Vec::new();
                        if let Some(amount) = self.calculate_damage(
                            actor,
                            recipient,
                            spec,
                            power,
                            false,
                            false,
                            0,
                            typeless,
                            &mut details,
                        ) {
                            let actual =
                                amount.min(self.get(recipient).hp.saturating_sub(min_target_hp));
                            if actual > 0 {
                                self.damage_from_move(actor, recipient, actual, events)?;
                            }
                            events.extend(details);
                            operation_hit = true;
                            any_hit = true;
                            total_damage = total_damage.saturating_add(actual);
                            if let Some(fraction) = drain {
                                if actual > 0 && self.get(actor).hp < self.get(actor).max_hp {
                                    self.heal(
                                        actor,
                                        ((actual as f32 * fraction).floor() as u16).max(1),
                                        events,
                                    );
                                }
                                events.push(BattleEvent::Message(format!(
                                    "{} had its energy drained!",
                                    self.get(recipient).name
                                )));
                            }
                        }
                    }
                    Value::Table(self.catalog.damage_result(operation_hit, total_damage)?)
                }
            };
            yielded = thread.resume::<Value>(response)?;
        }
        Ok(any_hit)
    }
    fn interrupt_script(
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
    fn change_stat(
        &mut self,
        recipient: ParticipantId,
        stat: Stat,
        delta: i8,
        events: &mut Vec<BattleEvent>,
    ) {
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
    }
    fn hit_check(&mut self, actor: ParticipantId, target: ParticipantId, spec: &MoveSpec) -> bool {
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
            Accuracy::Chance(value) => self.hit_check_chance(actor, target, value),
        }
    }
    fn hit_check_chance(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        value: f32,
    ) -> bool {
        let probability =
            value * self.get(actor).accuracy_factor() * self.get(target).evasion_factor();
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
                    self.damage_from_move(actor, target, amount, events)?;
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
                    self.damage_from_move(actor, target, amount, events)?;
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
                if !self.rng.chance(*chance)
                    || (has_damage && self.effectiveness(target, spec.move_type) == 0.0)
                {
                    return Ok(());
                }
                let recipient = if *self_target { actor } else { target };
                if self.get(recipient).hp == 0 {
                    return Ok(());
                }
                for (stat, delta) in stages {
                    self.change_stat(recipient, *stat, *delta, events);
                }
            }
            Effect::Status {
                status,
                chance,
                replace,
            } => {
                if !self.rng.chance(*chance)
                    || (has_damage && self.effectiveness(target, spec.move_type) == 0.0)
                {
                    return Ok(());
                }
                let new_status = status.battle_status();
                let current = self.get(target).status;
                let immune =
                    type_chart::status_immune(new_status, spec.move_type, &self.get(target).types);
                if (current.is_none() || *replace) && !immune {
                    let turns = match status {
                        AppliedStatus::RestSleep => 2,
                        AppliedStatus::Asleep => self.rng.range(1, 3),
                        AppliedStatus::BadlyPoisoned => 1,
                        _ => 0,
                    };
                    let p = self.get_mut(target);
                    p.status = Some(new_status);
                    p.status_turns = turns;
                    events.push(BattleEvent::Status {
                        target,
                        status: Some(new_status),
                    });
                    let message = match status {
                        AppliedStatus::RestSleep => {
                            format!("{} slept and became healthy!", self.get(target).name)
                        }
                        AppliedStatus::Asleep => format!("{} fell asleep!", self.get(target).name),
                        AppliedStatus::Frozen => {
                            format!("{} was frozen solid!", self.get(target).name)
                        }
                        _ => format!("{} was {}!", self.get(target).name, status.description()),
                    };
                    events.push(BattleEvent::Message(message));
                } else if single_effect {
                    let text = if immune {
                        format!("It doesn't affect {}...", self.get(target).name)
                    } else if current == Some(new_status) {
                        format!(
                            "{} was already {}!",
                            self.get(target).name,
                            status.description()
                        )
                    } else {
                        "But it failed!".into()
                    };
                    events.push(BattleEvent::Message(text));
                }
            }
            Effect::Confuse => {
                self.rng.chance(1.0); // C# ConfuseEffect checks its default chance.
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
                if self.rng.chance(*chance) && self.effectiveness(target, spec.move_type) != 0.0 {
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
                if self.weather.as_ref().is_some_and(|w| w.kind == *kind) {
                    events.push(BattleEvent::Message("But it failed!".into()));
                } else {
                    let weather = Weather {
                        kind: *kind,
                        turns_left: *turns,
                    };
                    self.weather = Some(weather.clone());
                    events.push(BattleEvent::Message(
                        if *kind == WeatherKind::Sun {
                            "The sunlight turned harsh!"
                        } else {
                            "A sandstorm kicked up!"
                        }
                        .into(),
                    ));
                    events.push(BattleEvent::Weather(Some(weather)));
                }
            }
            Effect::TwoTurn {
                power,
                charge_message,
                semi_invulnerable,
                skip_in_sun,
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
                        self.damage_from_move(actor, target, amount, events)?;
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
                    self.damage_from_move(actor, target, amount, events)?;
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
                    self.damage_from_move(actor, target, amount, events)?;
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
    fn effectiveness(&self, target: ParticipantId, move_type: PokemonType) -> f32 {
        type_chart::multiplier(move_type, &self.get(target).types)
    }
    fn immune_message(&self, target: ParticipantId, events: &mut Vec<BattleEvent>) {
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
            self.damage_from_move(actor, target, amount, events)?;
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
        let defender = self.get(target).clone();
        let attacker = self.get(actor).clone();
        let crit_stage = attacker.crit_stage.saturating_add(u8::from(high_crit));
        let chance = match crit_stage {
            0 => 1.0 / 24.0,
            1 => 1.0 / 8.0,
            2 => 0.5,
            _ => 1.0,
        };
        let crit = always_crit || self.rng.chance(chance);
        let effectiveness = if typeless {
            1.0
        } else {
            self.effectiveness(target, spec.move_type)
        };
        if effectiveness == 0.0 {
            self.immune_message(target, events);
            return None;
        }
        let (atk_name, def_name) = match spec.category {
            Category::Physical => (CombatStat::Attack, CombatStat::Defense),
            _ => (CombatStat::SpAttack, CombatStat::SpDefense),
        };
        let attack = if crit && attacker.stage(atk_name.into()) < 0 {
            (match atk_name {
                CombatStat::Attack => attacker.attack,
                CombatStat::SpAttack => attacker.sp_attack,
                _ => unreachable!(),
            }) as f32
        } else {
            attacker.effective_stat(atk_name)
        };
        let mut defense = if crit && defender.stage(def_name.into()) > 0 {
            (match def_name {
                CombatStat::Defense => defender.defense,
                CombatStat::SpDefense => defender.sp_defense,
                _ => unreachable!(),
            }) as f32
        } else {
            defender.effective_stat(def_name)
        };
        if self
            .weather
            .as_ref()
            .is_some_and(|w| w.kind == WeatherKind::Sandstorm)
            && def_name == CombatStat::SpDefense
            && defender.types.contains(&PokemonType::Rock)
        {
            defense *= 1.5;
        }
        defense = defense.max(1.0);
        let adjusted_power =
            f32::from(power) * 2f32.powi(consecutive_boost as i32) * self.active_power_multiplier;
        let mut damage = (2.0 * f32::from(attacker.level) * 0.2 + 2.0) * adjusted_power * attack
            / defense
            / 50.0
            + 2.0;
        if attacker.status == Some(Status::Burned) && spec.category == Category::Physical {
            damage /= 2.0;
        }
        if self
            .weather
            .as_ref()
            .is_some_and(|w| w.kind == WeatherKind::Sun)
        {
            if !typeless && spec.move_type == PokemonType::Fire {
                damage *= 1.5;
            }
            if !typeless && spec.move_type == PokemonType::Water {
                damage *= 0.5;
            }
        }
        damage *= effectiveness;
        if !typeless && attacker.types.iter().any(|t| t == &spec.move_type) {
            damage *= 1.5;
        }
        if crit {
            damage *= 1.5;
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
        Some(amount)
    }
    fn damage_from_move(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        amount: u16,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), BattleError> {
        let before = self.get(target).hp;
        self.damage(target, amount, events);
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

    fn damage(&mut self, target: ParticipantId, amount: u16, events: &mut Vec<BattleEvent>) {
        let p = self.get_mut(target);
        let was_alive = p.hp > 0;
        p.hp = p.hp.saturating_sub(amount);
        let hp = p.hp;
        let name = p.name.clone();
        events.push(BattleEvent::Damage { target, amount, hp });
        if was_alive && hp == 0 {
            self.hit_reactions
                .retain(|reaction| reaction.owner != target);
            events.push(BattleEvent::Message(format!("{name} fainted!")));
            events.push(BattleEvent::Fainted(target));
        }
    }
    fn heal(&mut self, target: ParticipantId, amount: u16, events: &mut Vec<BattleEvent>) {
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
        if weather.turns_left == 0 {
            events.push(BattleEvent::Message(
                if weather.kind == WeatherKind::Sun {
                    "The harsh sunlight faded."
                } else {
                    "The sandstorm subsided."
                }
                .into(),
            ));
            self.weather = None;
            events.push(BattleEvent::Weather(None));
            return;
        }
        self.weather.as_mut().unwrap().turns_left -= 1;
        events.push(BattleEvent::Message(
            if weather.kind == WeatherKind::Sun {
                "The sunlight is strong."
            } else {
                "The sandstorm rages."
            }
            .into(),
        ));
        if weather.kind == WeatherKind::Sandstorm {
            for (id, _, _, _, _) in order {
                let p = self.get(*id).clone();
                if p.hp == 0
                    || p.types.iter().any(|t| {
                        matches!(
                            t,
                            PokemonType::Rock | PokemonType::Ground | PokemonType::Steel
                        )
                    })
                {
                    continue;
                }
                events.push(BattleEvent::Message(format!(
                    "{} is buffeted by the sandstorm!",
                    p.name
                )));
                self.damage(*id, fractional(p.max_hp, 16, 1), events);
            }
        }
    }
    fn resolve_outcome(&mut self, events: &mut Vec<BattleEvent>) {
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
