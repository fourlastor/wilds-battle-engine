//! Substitute absorbs individual hits and shields the owner from their secondary effects.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId};
use crate::moves::MoveSpec;

impl Battle {
    pub(crate) fn create_substitute(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let cost = (self.get(who).max_hp / 4).max(1);
        if self.get(who).substitute_hp > 0 || self.get(who).hp <= cost {
            return false;
        }
        self.damage(who, cost, events);
        self.get_mut(who).substitute_hp = cost;
        self.get_mut(who).bound = None;
        events.push(BattleEvent::Substitute {
            target: who,
            amount: 0,
            hp: cost,
        });
        events.push(BattleEvent::Message(format!(
            "{} made a substitute!",
            self.get(who).display_name()
        )));
        true
    }
    pub(crate) fn substitute_intercepts(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        bypass: bool,
    ) -> bool {
        actor != target
            && self.get(target).substitute_hp > 0
            && !bypass
            && !self.has_ability(actor, "infiltrator")
    }
    pub(crate) fn substitute_blocks_effect(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
    ) -> bool {
        let bypass = spec.has_flag("sound") || spec.has_flag("bypass_substitute");
        actor != target
            && !bypass
            && !self.has_ability(actor, "infiltrator")
            && (self.get(target).substitute_hp > 0 || self.substitute_hits.contains(&target))
    }
    pub(crate) fn move_damage_limit(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
        min_hp: u16,
    ) -> u16 {
        if self.substitute_intercepts(
            actor,
            target,
            spec.has_flag("sound") || spec.has_flag("bypass_substitute"),
        ) {
            self.get(target).substitute_hp
        } else {
            self.get(target).hp.saturating_sub(min_hp)
        }
    }
}
