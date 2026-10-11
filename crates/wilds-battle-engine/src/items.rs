//! Item consumption and activation. Taking or giving an item does not consume it.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId};

impl Battle {
    pub(crate) fn consume_item(&mut self, who: ParticipantId) -> Option<String> {
        let pokemon = self.get_mut(who);
        if pokemon.hp == 0 {
            return None;
        }
        let item = pokemon.item.take()?;
        pokemon.last_consumed_item = Some(item.clone());
        pokemon.consumed_items.insert(item.clone());
        Some(item)
    }

    /// Called only when a move has actually entered its charging phase.
    pub(crate) fn try_skip_charge(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if !matches!(
            self.get(who).item.as_deref(),
            Some("power-herb" | "power_herb")
        ) {
            return false;
        }
        if self.consume_item(who).is_none() {
            return false;
        }
        events.push(BattleEvent::Message(format!(
            "{} became fully charged due to its Power Herb!",
            self.get(who).name
        )));
        true
    }
}
