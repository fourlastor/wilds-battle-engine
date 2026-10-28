//! Apparent identities are presentation data, separate from real battle traits.
use crate::engine::Battle;
use crate::model::say;
use crate::model::{BattleEvent, ParticipantId, Pokemon, PokemonAppearance};

impl Pokemon {
    pub fn display_name(&self) -> &str {
        self.appearance().map_or(&self.name, |p| &p.name)
    }
    /// What the Pokémon looks like when not itself: an Illusion, else a host disguise.
    pub fn appearance(&self) -> Option<&PokemonAppearance> {
        self.illusion.as_ref().or(self.disguise.as_ref())
    }
}

impl Battle {
    pub(crate) fn start_illusion(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        self.get_mut(who).illusion = None;
        if !self.has_ability(who, "illusion") || self.get(who).transformed() {
            return;
        }
        let Some(index) = self.party_order[who.side.index()]
            .iter()
            .rev()
            .copied()
            .find(|index| {
                self.get(ParticipantId {
                    side: who.side,
                    index: *index,
                })
                .hp > 0
            })
        else {
            return;
        };
        if index == who.index {
            return;
        }
        let source = self.get(ParticipantId {
            side: who.side,
            index,
        });
        let appearance = PokemonAppearance {
            name: source.name.clone(),
            species_id: source.species_id,
            species: source.species.clone(),
            form: source.form.clone(),
            gender: source.gender,
        };
        self.get_mut(who).illusion = Some(appearance.clone());
        events.push(BattleEvent::AppearanceChanged {
            target: who,
            illusion: Some(appearance),
        });
    }

    pub(crate) fn end_illusion(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        if self.get_mut(who).illusion.take().is_some() {
            events.push(BattleEvent::AppearanceChanged {
                target: who,
                illusion: None,
            });
            events.push(say("{0}'s illusion wore off!", [self.who(who)]));
        }
    }
}
