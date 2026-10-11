//! Party identities stay stable; only the occupant of a field position changes.
use crate::engine::Battle;
use crate::model::{BattleError, BattleEvent, Choice, ParticipantId, Side, Status};

impl Battle {
    /// Configure field positions before the first prompt. Omitted configuration keeps all
    /// participants active, preserving the original multi-participant setup behavior.
    pub fn set_active(&mut self, active: [Vec<usize>; 2]) -> Result<(), BattleError> {
        if self.turn() != 0 || self.has_started() {
            return Err(BattleError::InvalidSetup(
                "active positions must be set before advancing".into(),
            ));
        }
        for side in Side::ALL {
            let row = &active[side.index()];
            if row.is_empty()
                || row.iter().any(|&i| i >= self.participants(side).len())
                || row.iter().collect::<std::collections::BTreeSet<_>>().len() != row.len()
            {
                return Err(BattleError::InvalidSetup(
                    "active positions must be distinct valid party indices".into(),
                ));
            }
        }
        for side in Side::ALL {
            self.party_order[side.index()] = active[side.index()]
                .iter()
                .copied()
                .chain(
                    (0..self.sides[side.index()].len())
                        .filter(|i| !active[side.index()].contains(i)),
                )
                .collect();
        }
        self.active = active;
        Ok(())
    }
    pub fn active_indices(&self, side: Side) -> &[usize] {
        &self.active[side.index()]
    }
    pub fn position(&self, who: ParticipantId) -> Option<usize> {
        self.active[who.side.index()]
            .iter()
            .position(|&index| index == who.index)
    }
    pub fn is_active(&self, who: ParticipantId) -> bool {
        self.position(who).is_some()
    }
    pub(crate) fn bench(&self, side: Side) -> Vec<ParticipantId> {
        self.participants(side)
            .iter()
            .enumerate()
            .filter(|(index, pokemon)| pokemon.hp > 0 && !self.active[side.index()].contains(index))
            .map(|(index, _)| ParticipantId { side, index })
            .collect()
    }
    pub(crate) fn replacement(&self) -> Option<(ParticipantId, Vec<Choice>)> {
        for side in Side::ALL {
            let bench = self.bench(side);
            if bench.is_empty() {
                continue;
            }
            for &index in &self.active[side.index()] {
                let actor = ParticipantId { side, index };
                if self.get(actor).hp == 0 {
                    return Some((
                        actor,
                        bench
                            .iter()
                            .enumerate()
                            .map(|(id, &to)| Choice::Switch { id: id as u32, to })
                            .collect(),
                    ));
                }
            }
        }
        None
    }
    pub(crate) fn switch_member(
        &mut self,
        from: ParticipantId,
        to: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let Some(position) = self.position(from) else {
            return false;
        };
        if from.side != to.side || !self.bench(from.side).contains(&to) {
            return false;
        }
        let passed = self.take_baton_pass(from);
        self.ability_on_exit(from, events);
        self.release_carry(from, events);
        self.release_links(from);
        // Draining effects attach their recovery to a field slot (for example Leech Seed).
        for condition in self
            .sides
            .iter_mut()
            .flatten()
            .flat_map(|p| p.conditions.iter_mut())
            .chain(self.side_conditions.iter_mut().flatten())
            .chain(self.field_conditions.iter_mut())
        {
            for rule in &mut condition.rules {
                if let crate::model::Rule::DrainEachTurn { to: recipient, .. } = rule
                    && *recipient == from
                {
                    *recipient = to;
                }
            }
        }
        self.restore_transform(from, events);
        self.restore_form(from, events);
        self.get_mut(from).illusion = None;
        let outgoing = self.get_mut(from);
        if let Some((form, base)) = outgoing.battle_base_form.clone() {
            base.apply(outgoing);
            outgoing.form = form;
        }
        outgoing.restore_moves();
        outgoing.last_position = Some(position);
        outgoing.stages.clear();
        outgoing.conditions.clear();
        outgoing.ability_move_lock = None;
        outgoing.confused_turns = None;
        outgoing.bound = None;
        outgoing.protected = false;
        outgoing.substitute_hp = 0;
        outgoing.flinched = false;
        outgoing.locked_move = None;
        outgoing.script_continuation = None;
        outgoing.next_move_power_boost = None;
        outgoing.last_move = None;
        outgoing.consecutive_count = 0;
        outgoing.protection_streak = 0;
        outgoing.last_protection_turn = None;
        outgoing.crit_stage = 0;
        outgoing.semi_invulnerable = false;
        outgoing.recharging = false;
        outgoing.ability_suppressed = false;
        if let Some(original) = outgoing.original_ability.take() {
            outgoing.ability = original.id;
            outgoing.ability_rules = original.rules;
            outgoing.ability_bypassable = original.bypassable;
        }
        outgoing.last_hit = None;
        outgoing.damage_taken = 0;
        outgoing.hurt_this_turn = false;
        outgoing.stats_raised_this_turn = false;
        outgoing.stats_lowered_this_turn = false;
        outgoing.moves_used.clear();
        outgoing.last_used = None;
        outgoing.last_move_target = None;
        outgoing.last_move_type = None;
        outgoing.last_move_failed = false;
        outgoing.streak_move = None;
        outgoing.streak = 0;
        if outgoing.status == Some(Status::BadlyPoisoned) {
            outgoing.status_turns = 1;
        }
        self.hit_reactions.retain(|reaction| reaction.owner != from);
        self.active[from.side.index()][position] = to.index;
        let order = &mut self.party_order[from.side.index()];
        let a = order.iter().position(|i| *i == from.index).unwrap();
        let b = order.iter().position(|i| *i == to.index).unwrap();
        order.swap(a, b);
        self.start_illusion(to, events);
        let turn = self.turn;
        let incoming = self.get_mut(to);
        incoming.entered_turn = turn;
        incoming.last_position = Some(position);
        incoming.acted = false;
        events.push(BattleEvent::Switched { from, to, position });
        events.push(BattleEvent::Message(format!(
            "{} was replaced by {}!",
            self.get(from).display_name(),
            self.get(to).display_name()
        )));
        // Move-scoped ability bypass must not leak into the incoming Pokémon's entry.
        let ignored = self.ignored_ability.take();
        if let Some(passed) = passed {
            self.receive_baton_pass(to, passed, events);
        }
        self.heal_replacement(to, events);
        let hp_before = self.get(to).hp;
        self.entry_hazards(to, events);
        self.refresh_neutralizing_gas(events);
        if !self.imposter_on_entry(to, events) {
            self.ability_on_entry(to, events);
        }
        self.update_berries(events);
        self.check_ability_exits(&[(to, hp_before)], false, None, events);
        self.ignored_ability = ignored;
        true
    }
}

impl Battle {
    /// A source which leaves or faints cannot trap its old targets again on reentry.
    pub(crate) fn release_links(&mut self, source: ParticipantId) {
        use crate::model::Rule;
        for pokemon in self.sides.iter_mut().flatten() {
            if pokemon
                .bound
                .as_ref()
                .is_some_and(|bound| bound.by == Some(source))
            {
                pokemon.bound = None;
            }
        }
        for conditions in self
            .sides
            .iter_mut()
            .flatten()
            .map(|p| &mut p.conditions)
            .chain(self.side_conditions.iter_mut())
            .chain(std::iter::once(&mut self.field_conditions))
        {
            conditions.retain_mut(|condition| {
                let before = condition.rules.len();
                condition.rules.retain(|rule| {
                    !matches!(rule,
                        Rule::InfatuatedBy(by)
                        | Rule::TrappedBy { source: by }
                        | Rule::StatEachTurn { source: by, .. }
                        | Rule::AlwaysHit { against: Some(by) } if *by == source)
                });
                condition.rules.len() == before || !condition.rules.is_empty()
            });
        }
    }
}

impl Battle {
    pub(crate) fn request_forced_switch(
        &mut self,
        actor: ParticipantId,
        who: ParticipantId,
    ) -> bool {
        if self.get(actor).hp == 0
            || self.get(who).hp == 0
            || !self.is_active(who)
            || self.get(who).carrying.is_some()
            || self.get(who).carried_by.is_some()
            || self.bench(who.side).is_empty()
            || self.has_ability(who, "suctioncups")
            || self.has_ability(who, "guarddog")
            || self
                .rules(who)
                .any(|rule| matches!(rule, crate::model::Rule::BlockForcedSwitch))
        {
            return false;
        }
        if !self
            .forced_switches
            .iter()
            .any(|(_, target)| *target == who)
        {
            self.forced_switches.push((actor, who));
        }
        true
    }
    pub(crate) fn finish_forced_switches(&mut self, events: &mut Vec<BattleEvent>) {
        for (actor, who) in std::mem::take(&mut self.forced_switches) {
            if self.get(actor).hp == 0 || self.get(who).hp == 0 || !self.is_active(who) {
                continue;
            }
            let choices = self.bench(who.side);
            if !choices.is_empty() {
                let to = choices[self.rng.index(choices.len())];
                self.switch_member(who, to, events);
            }
        }
    }
}

impl Battle {
    pub(crate) fn request_pivot(&mut self, who: ParticipantId) -> bool {
        if self.get(who).hp == 0
            || !self.is_active(who)
            || self.bench(who.side).is_empty()
            || self.get(who).carrying.is_some()
            || self.get(who).carried_by.is_some()
        {
            return false;
        }
        self.pivot = Some(who);
        true
    }
}
