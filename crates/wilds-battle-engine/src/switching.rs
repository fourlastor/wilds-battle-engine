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
        self.restore_form(from, events);
        let outgoing = self.get_mut(from);
        if let Some((form, base)) = outgoing.battle_base_form.clone() {
            base.apply(outgoing);
            outgoing.form = form;
        }
        outgoing.last_position = Some(position);
        outgoing.stages.clear();
        outgoing.conditions.clear();
        outgoing.confused_turns = None;
        outgoing.bound = None;
        outgoing.protected = false;
        outgoing.flinched = false;
        outgoing.locked_move = None;
        outgoing.script_continuation = None;
        outgoing.next_move_power_boost = None;
        outgoing.last_move = None;
        outgoing.consecutive_count = 0;
        outgoing.crit_stage = 0;
        outgoing.semi_invulnerable = false;
        outgoing.recharging = false;
        outgoing.ability_suppressed = false;
        outgoing.last_hit = None;
        outgoing.damage_taken = 0;
        outgoing.hurt_this_turn = false;
        outgoing.stats_raised_this_turn = false;
        outgoing.stats_lowered_this_turn = false;
        outgoing.moves_used.clear();
        outgoing.last_used = None;
        outgoing.last_move_failed = false;
        outgoing.streak_move = None;
        outgoing.streak = 0;
        if outgoing.status == Some(Status::BadlyPoisoned) {
            outgoing.status_turns = 1;
        }
        self.hit_reactions.retain(|reaction| reaction.owner != from);
        self.active[from.side.index()][position] = to.index;
        let turn = self.turn;
        let incoming = self.get_mut(to);
        incoming.entered_turn = turn;
        incoming.last_position = Some(position);
        incoming.acted = true;
        events.push(BattleEvent::Switched { from, to, position });
        events.push(BattleEvent::Message(format!(
            "{} was replaced by {}!",
            self.get(from).name,
            self.get(to).name
        )));
        true
    }
}
