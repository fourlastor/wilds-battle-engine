//! Ability activation and suppression gates shared by engine mechanics.
use crate::engine::Battle;
use crate::items::item_key;
use crate::model::{BattleEvent, ParticipantId};

/// Pokemon Showdown 942cda71, data/abilities.ts: cantsuppress metadata.
fn cannot_suppress(ability: &str) -> bool {
    matches!(
        ability,
        "asoneglastrier"
            | "asonespectrier"
            | "battlebond"
            | "comatose"
            | "disguise"
            | "gulpmissile"
            | "iceface"
            | "multitype"
            | "powerconstruct"
            | "rkssystem"
            | "schooling"
            | "shieldsdown"
            | "stancechange"
            | "terashift"
            | "zenmode"
            | "zerotohero"
    )
}

impl Battle {
    pub(crate) fn has_ability(&self, who: ParticipantId, ability: &str) -> bool {
        let pokemon = self.get(who);
        pokemon.hp > 0
            && !pokemon.ability_suppressed
            && pokemon
                .ability
                .as_deref()
                .is_some_and(|id| item_key(id) == ability)
    }

    pub(crate) fn suppress_ability(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let pokemon = self.get(who);
        if pokemon.hp == 0
            || pokemon.ability_suppressed
            || pokemon
                .ability
                .as_deref()
                .is_none_or(|id| id.is_empty() || cannot_suppress(&item_key(id)))
        {
            return false;
        }
        let pokemon = self.get_mut(who);
        pokemon.ability_suppressed = true;
        events.push(BattleEvent::Message(format!(
            "{}'s ability was suppressed!",
            pokemon.name
        )));
        true
    }
}
