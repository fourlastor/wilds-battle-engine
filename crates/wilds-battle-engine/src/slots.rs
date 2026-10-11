//! Effects attached to a field position, independent of the Pokémon currently occupying it.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId, Side};

pub(crate) struct DelayedHeal {
    side: Side,
    position: usize,
    due: u64,
    amount: u16,
}

impl Battle {
    pub(crate) fn schedule_heal(&mut self, who: ParticipantId, amount: u16, delay: u8) -> bool {
        let Some(position) = self.position(who) else {
            return false;
        };
        if self.get(who).hp == 0
            || self
                .delayed_heals
                .iter()
                .any(|effect| effect.side == who.side && effect.position == position)
        {
            return false;
        }
        self.delayed_heals.push(DelayedHeal {
            side: who.side,
            position,
            due: self.turn + u64::from(delay),
            amount,
        });
        true
    }

    pub(crate) fn resolve_delayed_heals(&mut self, events: &mut Vec<BattleEvent>) {
        for effect in std::mem::take(&mut self.delayed_heals) {
            if effect.due > self.turn {
                self.delayed_heals.push(effect);
                continue;
            }
            let Some(&index) = self.active[effect.side.index()].get(effect.position) else {
                continue;
            };
            let target = ParticipantId {
                side: effect.side,
                index,
            };
            self.heal(target, effect.amount, events);
        }
    }
}
