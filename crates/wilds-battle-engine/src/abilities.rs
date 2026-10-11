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
    pub(crate) fn ability_on_exit(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        // Run before volatile state and suppression are cleared, and only while alive.
        if self.has_ability(who, "regenerator") {
            self.heal(who, self.get(who).max_hp / 3, events);
        }
        if self.has_ability(who, "naturalcure") {
            self.cure_status(who, None, events);
        }
    }

    pub(crate) fn ability_on_entry(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        if !self.has_ability(who, "intimidate") {
            return;
        }
        events.push(BattleEvent::Message(format!(
            "{}'s Intimidate activated!",
            self.get(who).name
        )));
        for target in self.actors() {
            if target.side == who.side || !self.adjacent(who, target) {
                continue;
            }
            let immune = self.blocks_stat_drops(target)
                || [
                    "hypercutter",
                    "innerfocus",
                    "oblivious",
                    "owntempo",
                    "scrappy",
                ]
                .iter()
                .any(|ability| self.has_ability(target, ability));
            if immune {
                events.push(BattleEvent::Message(format!(
                    "{} resisted Intimidate!",
                    self.get(target).name
                )));
                continue;
            }
            let amount = if self.has_ability(target, "guarddog") {
                1
            } else {
                -1
            };
            self.change_stat(target, crate::model::Stat::Attack, amount, events);
            if self.has_ability(target, "rattled") {
                self.change_stat(target, crate::model::Stat::Speed, 1, events);
            }
        }
    }

    pub(crate) fn has_ability(&self, who: ParticipantId, ability: &str) -> bool {
        self.ability_active(who)
            && self
                .get(who)
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

/// Breakable metadata is distinct from suppression protection: Disguise is an example
/// of an ability which cannot be suppressed but can be bypassed by a move.
fn bypassable(ability: &str) -> bool {
    matches!(
        ability,
        "armortail"
            | "aromaveil"
            | "aurabreak"
            | "battlearmor"
            | "bigpecks"
            | "bulletproof"
            | "clearbody"
            | "contrary"
            | "damp"
            | "dazzling"
            | "disguise"
            | "dryskin"
            | "eartheater"
            | "filter"
            | "flashfire"
            | "flowergift"
            | "flowerveil"
            | "fluffy"
            | "friendguard"
            | "furcoat"
            | "goodasgold"
            | "grasspelt"
            | "guarddog"
            | "heatproof"
            | "heavymetal"
            | "hypercutter"
            | "iceface"
            | "icescales"
            | "illuminate"
            | "immunity"
            | "innerfocus"
            | "insomnia"
            | "keeneye"
            | "leafguard"
            | "levitate"
            | "lightmetal"
            | "lightningrod"
            | "limber"
            | "magicbounce"
            | "magmaarmor"
            | "marvelscale"
            | "mindseye"
            | "mirrorarmor"
            | "motordrive"
            | "multiscale"
            | "oblivious"
            | "overcoat"
            | "owntempo"
            | "pastelveil"
            | "punkrock"
            | "purifyingsalt"
            | "queenlymajesty"
            | "sandveil"
            | "sapsipper"
            | "shellarmor"
            | "shielddust"
            | "simple"
            | "snowcloak"
            | "solidrock"
            | "soundproof"
            | "stickyhold"
            | "stormdrain"
            | "sturdy"
            | "suctioncups"
            | "sweetveil"
            | "tangledfeet"
            | "telepathy"
            | "terashell"
            | "thermalexchange"
            | "thickfat"
            | "unaware"
            | "vitalspirit"
            | "voltabsorb"
            | "waterabsorb"
            | "waterbubble"
            | "waterveil"
            | "wellbakedbody"
            | "whitesmoke"
            | "windrider"
            | "wonderguard"
            | "wonderskin"
    )
}

impl Battle {
    pub(crate) fn ability_active(&self, who: ParticipantId) -> bool {
        let pokemon = self.get(who);
        pokemon.hp > 0
            && self.is_active(who)
            && !pokemon.ability_suppressed
            && pokemon.ability.as_deref().is_some_and(|id| {
                if id.is_empty() {
                    return false;
                }
                let ignored = self.ignored_ability == Some(who)
                    && pokemon
                        .ability_bypassable
                        .unwrap_or_else(|| bypassable(&item_key(id)));
                !ignored
            })
    }

    pub(crate) fn ability_stat_factor(
        &self,
        who: ParticipantId,
        stat: crate::model::CombatStat,
    ) -> f32 {
        use crate::model::CombatStat::*;
        if (stat == Defense && self.has_ability(who, "furcoat"))
            || (stat == Attack
                && (self.has_ability(who, "hugepower") || self.has_ability(who, "purepower")))
        {
            2.0
        } else if stat == Defense
            && self.get(who).status.is_some()
            && self.has_ability(who, "marvelscale")
        {
            1.5
        } else {
            1.0
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ability_blocks_damage(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        kind: crate::model::PokemonType,
        effectiveness: f32,
        sound: bool,
        bullet: bool,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        use crate::model::PokemonType::{Electric, Fire, Grass, Ground, Water};
        if actor == target {
            return false;
        }
        let heal = (kind == Water
            && (self.has_ability(target, "waterabsorb") || self.has_ability(target, "dryskin")))
            || (kind == Electric && self.has_ability(target, "voltabsorb"));
        let boost = if kind == Electric && self.has_ability(target, "motordrive") {
            Some(crate::model::Stat::Speed)
        } else if (kind == Electric && self.has_ability(target, "lightningrod"))
            || (kind == Water && self.has_ability(target, "stormdrain"))
        {
            Some(crate::model::Stat::SpAttack)
        } else if kind == Grass && self.has_ability(target, "sapsipper") {
            Some(crate::model::Stat::Attack)
        } else {
            None
        };
        let flash = kind == Fire && self.has_ability(target, "flashfire");
        let blocked = heal
            || boost.is_some()
            || flash
            || (kind == Ground
                && !self.is_grounded(target)
                && self.has_ability(target, "levitate"))
            || (effectiveness <= 1.0 && self.has_ability(target, "wonderguard"))
            || (sound && self.has_ability(target, "soundproof"))
            || (bullet && self.has_ability(target, "bulletproof"))
            || (actor.side == target.side && self.has_ability(target, "telepathy"));
        if !blocked {
            return false;
        }
        events.push(BattleEvent::Message(format!(
            "{}'s ability blocked the move!",
            self.get(target).name
        )));
        self.move_failed = true;
        if heal {
            self.heal(target, (self.get(target).max_hp / 4).max(1), events);
        }
        if let Some(stat) = boost {
            self.change_stat(target, stat, 1, events);
        }
        if flash {
            self.set_mark(crate::model::Scope::Pokemon(target), "flash_fire", 1, None);
        }
        true
    }

    pub(crate) fn ability_damage_factor(
        &self,
        actor: ParticipantId,
        target: ParticipantId,
        attack: &crate::engine::Attack,
        effectiveness: f32,
    ) -> f32 {
        use crate::model::PokemonType::*;
        let mut factor = 1.0;
        if self.get(target).hp == self.get(target).max_hp
            && (self.has_ability(target, "multiscale") || self.has_ability(target, "shadowshield"))
        {
            factor *= 0.5;
        }
        if effectiveness > 1.0
            && (self.has_ability(target, "filter")
                || self.has_ability(target, "solidrock")
                || self.has_ability(target, "prismarmor"))
        {
            factor *= 0.75;
        }
        if attack.category == crate::moves::Category::Special
            && self.has_ability(target, "icescales")
        {
            factor *= 0.5;
        }
        if self.has_ability(target, "fluffy") {
            if attack.contact {
                factor *= 0.5;
            }
            if attack.move_type == Fire {
                factor *= 2.0;
            }
        }
        if attack.sound && self.has_ability(target, "punkrock") {
            factor *= 0.5;
        }
        if attack.move_type == Fire
            && self.has_ability(actor, "flashfire")
            && self
                .get(actor)
                .conditions
                .iter()
                .any(|condition| condition.name == "flash_fire")
        {
            factor *= 1.5;
        }
        factor
    }
}
