//! Linked carrying is engine state, so interruptions cannot strand either participant.
use crate::model::say;
use crate::{Battle, BattleEvent, HiddenKind, ParticipantId};

impl Battle {
    pub(crate) fn carry(&mut self, actor: ParticipantId, target: ParticipantId) -> bool {
        if actor.side == target.side || self.effective_weight(target) >= 200.0 {
            return false;
        }
        for id in [actor, target] {
            let p = self.get(id);
            if !self.is_active(id)
                || p.hp == 0
                || p.carrying.is_some()
                || p.carried_by.is_some()
                || p.semi_invulnerable
            {
                return false;
            }
        }
        self.get_mut(actor).carrying = Some(target);
        self.get_mut(target).carried_by = Some(actor);
        for id in [actor, target] {
            let p = self.get_mut(id);
            p.semi_invulnerable = true;
            p.hidden_kind = HiddenKind::Air;
        }
        true
    }
    pub(crate) fn release_carry(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let carrier = if self.get(who).carrying.is_some() {
            who
        } else if let Some(carrier) = self.get(who).carried_by {
            carrier
        } else {
            return false;
        };
        let Some(target) = self.get_mut(carrier).carrying.take() else {
            return false;
        };
        self.get_mut(target).carried_by = None;
        self.get_mut(carrier).script_continuation = None;
        for id in [carrier, target] {
            self.get_mut(id).semi_invulnerable = false;
        }
        events.push(say("{0} was released!", [self.who(target)]));
        true
    }
}
