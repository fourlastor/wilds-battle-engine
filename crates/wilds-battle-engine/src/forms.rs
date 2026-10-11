//! Form changes use the host's computed stats; levels, IVs, EVs and natures stay host-owned.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId, Pokemon, PokemonForm};

impl PokemonForm {
    pub fn of(pokemon: &Pokemon) -> Self {
        Self {
            max_hp: Some(pokemon.max_hp),
            attack: pokemon.attack,
            defense: pokemon.defense,
            sp_attack: pokemon.sp_attack,
            sp_defense: pokemon.sp_defense,
            speed: pokemon.speed,
            types: pokemon.types.clone(),
            weight: pokemon.weight,
        }
    }
    fn apply_hp(&self, pokemon: &mut Pokemon) {
        if let Some(max_hp) = self.max_hp.filter(|hp| *hp > 0) {
            let missing = pokemon.max_hp.saturating_sub(pokemon.hp);
            pokemon.max_hp = max_hp;
            if pokemon.hp > 0 {
                pokemon.hp = max_hp.saturating_sub(missing).max(1);
            }
        }
    }
    pub(crate) fn apply(&self, pokemon: &mut Pokemon) {
        pokemon.attack = self.attack;
        pokemon.defense = self.defense;
        pokemon.sp_attack = self.sp_attack;
        pokemon.sp_defense = self.sp_defense;
        pokemon.speed = self.speed;
        pokemon.types.clone_from(&self.types);
        pokemon.added_type = None;
        pokemon.weight = self.weight;
    }
}

impl Battle {
    pub(crate) fn change_form(
        &mut self,
        who: ParticipantId,
        form: &str,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let pokemon = self.get(who);
        if pokemon.hp == 0 || pokemon.form.as_deref() == Some(form) {
            return false;
        }
        if pokemon
            .original_form
            .as_ref()
            .is_some_and(|(id, _)| id.as_deref() == Some(form))
        {
            return self.restore_form(who, events);
        }
        let Some(next) = pokemon.forms.get(form).cloned() else {
            events.push(BattleEvent::Message(format!(
                "{} has no battle data for form {form}.",
                pokemon.display_name()
            )));
            return false;
        };
        let original = pokemon
            .battle_base_form
            .clone()
            .unwrap_or_else(|| (pokemon.form.clone(), PokemonForm::of(pokemon)));
        let pokemon = self.get_mut(who);
        pokemon.original_form.get_or_insert(original);
        next.apply(pokemon);
        next.apply_hp(pokemon);
        pokemon.form = Some(form.into());
        events.push(BattleEvent::FormChanged {
            target: who,
            form: Some(form.into()),
        });
        events.push(BattleEvent::Message(format!(
            "{} changed form!",
            pokemon.display_name()
        )));
        true
    }

    pub(crate) fn restore_form(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let pokemon = self.get_mut(who);
        let Some((form, original)) = pokemon.original_form.take() else {
            return false;
        };
        original.apply(pokemon);
        original.apply_hp(pokemon);
        pokemon.form = form.clone();
        events.push(BattleEvent::FormChanged { target: who, form });
        events.push(BattleEvent::Message(format!(
            "{} returned to its original form!",
            pokemon.display_name()
        )));
        true
    }
}
