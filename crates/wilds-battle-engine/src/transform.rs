//! Transform copies battle traits while keeping the original party member recoverable.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId, Pokemon, PokemonForm};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct OriginalPokemon {
    species_id: u32,
    species: String,
    form: Option<String>,
    forms: BTreeMap<String, PokemonForm>,
    original_form: Option<(Option<String>, PokemonForm)>,
    battle_base_form: Option<(Option<String>, PokemonForm)>,
    stats: PokemonForm,
    moves: Vec<String>,
    pp: BTreeMap<String, u8>,
}

impl Pokemon {
    pub(crate) fn original_types(&self) -> Vec<crate::model::PokemonType> {
        self.transformation
            .as_ref()
            .and_then(|p| p.battle_base_form.as_ref())
            .or(self.battle_base_form.as_ref())
            .map(|(_, form)| form.types.clone())
            .unwrap_or_else(|| self.types.clone())
    }

    pub fn transformed(&self) -> bool {
        self.transformation.is_some()
    }
}

impl Battle {
    pub(crate) fn max_pp(&self, who: ParticipantId, id: &str) -> Option<u8> {
        self.catalog.get(id).map(|spec| {
            if self.get(who).transformed() {
                spec.pp.min(5)
            } else {
                spec.pp
            }
        })
    }

    pub(crate) fn transform(
        &mut self,
        who: ParticipantId,
        target: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if self.get(who).illusion.is_some()
            || self.get(target).illusion.is_some()
            || who == target
            || !self.is_active(who)
            || !self.is_active(target)
            || self.get(who).hp == 0
            || self.get(target).hp == 0
            || self.get(who).transformed()
            || self.get(target).transformed()
            || self.get(target).substitute_hp > 0
        {
            return false;
        }
        let source = self.get(target).clone();
        let ability = self.ability_state(target);
        let pp = source
            .moves
            .iter()
            .filter_map(|id| {
                self.catalog
                    .get(id)
                    .map(|spec| (id.clone(), spec.pp.min(5)))
            })
            .collect();
        let p = self.get_mut(who);
        p.transformation = Some(OriginalPokemon {
            species_id: p.species_id,
            species: p.species.clone(),
            form: p.form.clone(),
            forms: p.forms.clone(),
            original_form: p.original_form.take(),
            battle_base_form: p.battle_base_form.clone(),
            stats: PokemonForm::of(p),
            moves: p.moves.clone(),
            pp: p.move_pp.clone(),
        });
        PokemonForm::of(&source).apply(p);
        p.species_id = source.species_id;
        p.species = source.species;
        p.form = source.form;
        p.forms = source.forms;
        p.battle_base_form = source.battle_base_form;
        p.added_type = source.added_type;
        p.stages = source.stages;
        p.moves = source.moves;
        p.move_pp = pp;
        let species = crate::items::item_key(&p.species);
        let form = crate::items::item_key(p.form.as_deref().unwrap_or_default());
        if species == "gengarmega"
            || ((p.species_id == 94 || species == "gengar")
                && (form == "mega" || form == "gengarmega"))
        {
            p.conditions.retain(|c| c.name != "telekinesis");
        }
        // Gen 6 onward also copies Focus Energy (including Lansat Berry) and Laser Focus.
        p.crit_stage = source.crit_stage;
        p.conditions
            .retain(|c| c.name != "focus_energy" && c.name != "laser_focus");
        for mut condition in source
            .conditions
            .into_iter()
            .filter(|c| c.name == "focus_energy" || c.name == "laser_focus")
        {
            if condition.name == "laser_focus" {
                condition.turns_left = Some(2);
            }
            p.conditions.push(condition);
        }
        events.push(BattleEvent::FormChanged {
            target: who,
            form: p.form.clone(),
        });
        events.push(BattleEvent::Message(format!(
            "{} transformed into {}!",
            p.display_name(),
            source.name
        )));
        self.install_ability(who, ability, events);
        self.ability_on_entry(who, events);
        true
    }

    pub(crate) fn restore_transform(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        let p = self.get_mut(who);
        let Some(original) = p.transformation.take() else {
            return;
        };
        original.stats.apply(p);
        p.species_id = original.species_id;
        p.species = original.species;
        p.form = original.form;
        p.forms = original.forms;
        p.original_form = original.original_form;
        p.battle_base_form = original.battle_base_form;
        p.moves = original.moves;
        p.move_pp = original.pp;
        p.crit_stage = 0;
        if let Some(ability) = p.original_ability.take() {
            p.ability = ability.id;
            p.ability_rules = ability.rules;
            p.ability_bypassable = ability.bypassable;
        }
        p.ability_suppressed = false;
        events.push(BattleEvent::FormChanged {
            target: who,
            form: p.form.clone(),
        });
    }
}
