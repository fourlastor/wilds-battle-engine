//! Temporary and permanent changes to learned moves. The host still owns the saved roster.
use crate::engine::Battle;
use crate::model::ParticipantId;

impl Battle {
    pub(crate) fn replace_move(
        &mut self,
        who: ParticipantId,
        old: &str,
        new: &str,
        permanent: bool,
    ) -> bool {
        let Some(spec) = self.catalog.get(new) else {
            return false;
        };
        let pp = spec.pp;
        let p = self.get_mut(who);
        if p.hp == 0 || p.moves.iter().any(|id| id == new) {
            return false;
        }
        let Some(slot) = p.moves.iter().position(|id| id == old) else {
            return false;
        };
        if permanent {
            p.temporary_moves.remove(&slot);
        } else {
            let old_pp = p.move_pp.get(old).copied().unwrap_or(0);
            p.temporary_moves
                .entry(slot)
                .or_insert_with(|| (old.to_owned(), old_pp));
        }
        p.move_pp.remove(old);
        p.move_pp.insert(new.to_owned(), pp);
        p.moves[slot] = new.to_owned();
        true
    }
}

impl crate::model::Pokemon {
    pub(crate) fn restore_moves(&mut self) {
        for (slot, (id, pp)) in std::mem::take(&mut self.temporary_moves) {
            let temporary = std::mem::replace(&mut self.moves[slot], id.clone());
            self.move_pp.remove(&temporary);
            self.move_pp.insert(id, pp);
        }
    }
}
