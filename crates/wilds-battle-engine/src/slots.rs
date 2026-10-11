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

pub(crate) struct ReplacementHeal {
    side: Side,
    position: usize,
    restore_pp: bool,
}

impl Battle {
    pub(crate) fn prepare_replacement_heal(
        &mut self,
        who: ParticipantId,
        restore_pp: bool,
    ) -> bool {
        let Some(position) = self.position(who) else {
            return false;
        };
        if self.get(who).hp == 0
            || self.bench(who.side).is_empty()
            || self.replacement_heals.iter().any(|effect| {
                effect.side == who.side
                    && effect.position == position
                    && effect.restore_pp == restore_pp
            })
        {
            return false;
        }
        self.replacement_heals.push(ReplacementHeal {
            side: who.side,
            position,
            restore_pp,
        });
        true
    }

    pub(crate) fn heal_replacement(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        let Some(position) = self.position(who) else {
            return;
        };
        if self.get(who).hp == 0 {
            return;
        }
        for effect in std::mem::take(&mut self.replacement_heals) {
            let p = self.get(who);
            let pp_missing = p.moves.iter().any(|id| {
                self.catalog
                    .get(id)
                    .is_some_and(|spec| p.move_pp.get(id).copied().unwrap_or(0) < spec.pp)
            });
            if effect.side != who.side
                || effect.position != position
                || (p.hp == p.max_hp && p.status.is_none() && !(effect.restore_pp && pp_missing))
            {
                self.replacement_heals.push(effect);
                continue;
            }
            let hp = p.max_hp;
            self.cure_status(who, None, events);
            self.heal_unchecked(who, hp, events);
            if effect.restore_pp {
                let pp: Vec<_> = self
                    .get(who)
                    .moves
                    .iter()
                    .filter_map(|id| self.catalog.get(id).map(|spec| (id.clone(), spec.pp)))
                    .collect();
                self.get_mut(who).move_pp.extend(pp);
            }
        }
    }
}
