//! Transfer volatile battle state only when the host selects the Baton Pass replacement.
use crate::engine::Battle;
use crate::model::{BattleEvent, Bound, Condition, NextMovePowerBoost, ParticipantId, Rule, Stat};
use std::collections::BTreeMap;

pub(crate) struct PassedState {
    source: ParticipantId,
    stages: BTreeMap<Stat, i8>,
    conditions: Vec<Condition>,
    confusion: Option<u8>,
    bound: Option<Bound>,
    substitute: u16,
    crit_stage: u8,
    protected: bool,
    next_move_power_boost: Option<NextMovePowerBoost>,
    suppressed: bool,
}
impl Battle {
    pub(crate) fn request_baton_pass(&mut self, who: ParticipantId, effects: Vec<String>) -> bool {
        if !self.request_pivot(who) {
            return false;
        }
        self.baton_pass = Some((who, effects));
        true
    }
    pub(crate) fn take_baton_pass(&mut self, who: ParticipantId) -> Option<PassedState> {
        if self
            .baton_pass
            .as_ref()
            .is_none_or(|(source, _)| *source != who)
        {
            return None;
        }
        let (_, effects) = self.baton_pass.take()?;
        let p = self.get(who);
        if p.hp == 0 {
            return None;
        }
        Some(PassedState {
            source: who,
            stages: p.stages.clone(),
            conditions: p
                .conditions
                .iter()
                .filter(|c| effects.contains(&c.name))
                .cloned()
                .collect(),
            confusion: p.confused_turns,
            bound: p.bound.clone(),
            substitute: p.substitute_hp,
            crit_stage: p.crit_stage,
            protected: p.protected,
            next_move_power_boost: p.next_move_power_boost,
            suppressed: p.ability_suppressed,
        })
    }
    pub(crate) fn receive_baton_pass(
        &mut self,
        who: ParticipantId,
        mut passed: PassedState,
        events: &mut Vec<BattleEvent>,
    ) {
        for condition in &mut passed.conditions {
            for rule in &mut condition.rules {
                if let Rule::DrainEachTurn { to, .. } = rule
                    && *to == passed.source
                {
                    *to = who;
                }
            }
        }
        let p = self.get_mut(who);
        p.stages = passed.stages;
        p.conditions = passed.conditions;
        p.confused_turns = passed.confusion;
        p.bound = passed.bound;
        p.substitute_hp = passed.substitute;
        p.crit_stage = passed.crit_stage;
        p.protected = passed.protected;
        p.next_move_power_boost = passed.next_move_power_boost;
        if p.conditions.iter().any(|c| c.name == "power_trick") {
            std::mem::swap(&mut p.attack, &mut p.defense);
        }
        if passed.suppressed {
            self.suppress_ability(who, events);
        }
    }
}
