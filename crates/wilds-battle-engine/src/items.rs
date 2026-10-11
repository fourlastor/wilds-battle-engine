//! Item consumption and activation. Taking or giving an item does not consume it.
use crate::engine::Battle;
use crate::model::{BattleEvent, ParticipantId};

impl Battle {
    pub(crate) fn symbiosis_after_use(&mut self, who: ParticipantId) {
        if self.get(who).item.is_some() || self.pivot == Some(who) {
            return;
        }
        let mut donors: Vec<_> = self
            .actors()
            .into_iter()
            .filter(|&donor| {
                donor.side == who.side
                    && self.adjacent(who, donor)
                    && self.has_ability(donor, "symbiosis")
                    && self.item_removable(donor)
                    && self
                        .get(donor)
                        .item
                        .as_deref()
                        .is_some_and(|item| self.can_receive_item(who, item))
            })
            .collect();
        donors.sort_by_key(|&donor| std::cmp::Reverse(self.action_speed(donor)));
        if let Some(donor) = donors.first() {
            self.get_mut(who).item = self.take_item(*donor, *donor);
        }
    }

    pub(crate) fn ability_lost_item(&mut self, who: ParticipantId) {
        if self.has_ability(who, "unburden") {
            self.set_mark(crate::model::Scope::Pokemon(who), "unburden", 1, None);
        }
    }
    /// Whether the held item may provide its ordinary effects. Physical possession is separate.
    pub fn item_active(&self, who: ParticipantId) -> bool {
        let pokemon = self.get(who);
        let Some(item) = pokemon.item.as_deref() else {
            return false;
        };
        if pokemon.hp == 0 || !self.is_active(who) {
            return false;
        }
        matches!(item_key(item).as_str(), "redorb" | "blueorb")
            || (!self.has_ability(who, "klutz")
                && !self
                    .rules(who)
                    .any(|rule| matches!(rule, crate::model::Rule::BlockItems)))
    }

    pub(crate) fn consume_item(&mut self, who: ParticipantId) -> Option<String> {
        self.spend_item(who, true)
    }

    pub(crate) fn spend_item(&mut self, who: ParticipantId, eaten: bool) -> Option<String> {
        let turn = self.turn;
        let pokemon = self.get_mut(who);
        if pokemon.hp == 0 {
            return None;
        }
        let item = pokemon.item.take()?;
        pokemon.ate_berry |= eaten && crate::berries::is_berry(&item);
        pokemon.recyclable_item = Some(item.clone());
        pokemon.item_used_turn = Some(turn);
        pokemon.last_consumed_item = Some(item.clone());
        pokemon.consumed_items.insert(item.clone());
        self.ability_lost_item(who);
        self.symbiosis_after_use(who);
        Some(item)
    }

    pub(crate) fn recycle_item(&mut self, who: ParticipantId) -> bool {
        let pokemon = self.get_mut(who);
        if pokemon.hp == 0 || pokemon.item.is_some() {
            return false;
        }
        let Some(item) = pokemon.recyclable_item.take() else {
            return false;
        };
        pokemon.item = Some(item);
        true
    }

    /// Called only when a move has actually entered its charging phase.
    pub(crate) fn try_skip_charge(
        &mut self,
        who: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        if !self.item_active(who)
            || !matches!(
                self.get(who).item.as_deref(),
                Some("power-herb" | "power_herb")
            )
        {
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

/// Canonical comparison accepts public hyphenated IDs and legacy underscore IDs.
pub(crate) fn item_key(id: &str) -> String {
    id.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

/// Species-bound items from the generations covered by the move catalog.
/// Source: Pokemon Showdown 942cda71, data/items.ts and data/mods/gen8/items.ts.
fn bound_to(item: &str, pokemon: &crate::model::Pokemon) -> bool {
    let (number, name) = match item {
        "abomasite" => (460, "abomasnow"),
        "absolite" => (359, "absol"),
        "aerodactylite" => (142, "aerodactyl"),
        "aggronite" => (306, "aggron"),
        "alakazite" => (65, "alakazam"),
        "altarianite" => (334, "altaria"),
        "ampharosite" => (181, "ampharos"),
        "audinite" => (531, "audino"),
        "banettite" => (354, "banette"),
        "beedrillite" => (15, "beedrill"),
        "blastoisinite" => (9, "blastoise"),
        "blazikenite" => (257, "blaziken"),
        "cameruptite" => (323, "camerupt"),
        "charizarditex" => (6, "charizard"),
        "charizarditey" => (6, "charizard"),
        "diancite" => (719, "diancie"),
        "galladite" => (475, "gallade"),
        "garchompite" => (445, "garchomp"),
        "gardevoirite" => (282, "gardevoir"),
        "gengarite" => (94, "gengar"),
        "glalitite" => (362, "glalie"),
        "gyaradosite" => (130, "gyarados"),
        "heracronite" => (214, "heracross"),
        "houndoominite" => (229, "houndoom"),
        "kangaskhanite" => (115, "kangaskhan"),
        "latiasite" => (380, "latias"),
        "latiosite" => (381, "latios"),
        "lopunnite" => (428, "lopunny"),
        "lucarionite" => (448, "lucario"),
        "manectite" => (310, "manectric"),
        "mawilite" => (303, "mawile"),
        "medichamite" => (308, "medicham"),
        "metagrossite" => (376, "metagross"),
        "mewtwonitex" => (150, "mewtwo"),
        "mewtwonitey" => (150, "mewtwo"),
        "pidgeotite" => (18, "pidgeot"),
        "pinsirite" => (127, "pinsir"),
        "sablenite" => (302, "sableye"),
        "salamencite" => (373, "salamence"),
        "sceptilite" => (254, "sceptile"),
        "scizorite" => (212, "scizor"),
        "sharpedonite" => (319, "sharpedo"),
        "slowbronite" => (80, "slowbro"),
        "steelixite" => (208, "steelix"),
        "swampertite" => (260, "swampert"),
        "tyranitarite" => (248, "tyranitar"),
        "venusaurite" => (3, "venusaur"),
        "blueorb" => (382, "kyogre"),
        "redorb" => (383, "groudon"),
        "griseousorb" => (487, "giratina"),
        "rustedsword" => (888, "zacian"),
        "rustedshield" => (889, "zamazenta"),
        "burndrive" | "chilldrive" | "dousedrive" | "shockdrive" => (649, "genesect"),
        "bugmemory" | "darkmemory" | "dragonmemory" | "electricmemory" | "fairymemory"
        | "fightingmemory" | "firememory" | "flyingmemory" | "ghostmemory" | "grassmemory"
        | "groundmemory" | "icememory" | "poisonmemory" | "psychicmemory" | "rockmemory"
        | "steelmemory" | "watermemory" => (773, "silvally"),
        "dracoplate" | "dreadplate" | "earthplate" | "fistplate" | "flameplate" | "icicleplate"
        | "insectplate" | "ironplate" | "meadowplate" | "mindplate" | "pixieplate" | "skyplate"
        | "splashplate" | "spookyplate" | "stoneplate" | "toxicplate" | "zapplate" => {
            (493, "arceus")
        }
        _ => return false,
    };
    pokemon.species_id == number
        || pokemon
            .species
            .to_ascii_lowercase()
            .replace('_', "-")
            .split('-')
            .next()
            == Some(name)
}

impl Battle {
    /// Whether the item itself permits removal, independent of Sticky Hold.
    pub fn item_removable(&self, who: ParticipantId) -> bool {
        self.get(who)
            .item
            .as_ref()
            .is_some_and(|item| !bound_to(&item_key(item), self.get(who)))
    }

    pub(crate) fn take_item(
        &mut self,
        who: ParticipantId,
        source: ParticipantId,
    ) -> Option<String> {
        let pokemon = self.get(who);
        let item = pokemon.item.as_ref()?;
        let sticky_hold =
            source != who && self.has_ability(who, "stickyhold") && item_key(item) != "stickybarb";
        if sticky_hold || !self.item_removable(who) {
            return None;
        }
        let item = self.get_mut(who).item.take();
        self.ability_lost_item(who);
        item
    }

    pub(crate) fn can_receive_item(&self, who: ParticipantId, item: &str) -> bool {
        let pokemon = self.get(who);
        pokemon.hp > 0 && pokemon.item.is_none() && !bound_to(&item_key(item), pokemon)
    }

    /// Exchange atomically. A rejected item on either side leaves both held items untouched.
    pub(crate) fn swap_items(&mut self, source: ParticipantId, target: ParticipantId) -> bool {
        let mine = self.get(source);
        let theirs = self.get(target);
        if source == target
            || mine.hp == 0
            || theirs.hp == 0
            || (mine.item.is_none() && theirs.item.is_none())
            || self.has_ability(target, "stickyhold")
            || mine.item.as_ref().is_some_and(|item| {
                !self.item_removable(source) || bound_to(&item_key(item), theirs)
            })
            || theirs
                .item
                .as_ref()
                .is_some_and(|item| !self.item_removable(target) || bound_to(&item_key(item), mine))
        {
            return false;
        }
        let my_item = self.get_mut(source).item.take();
        let your_item = self.get_mut(target).item.take();
        if my_item.is_some() {
            self.ability_lost_item(source);
        }
        if your_item.is_some() {
            self.ability_lost_item(target);
        }
        self.get_mut(source).item = your_item;
        self.get_mut(target).item = my_item;
        true
    }

    pub(crate) fn steal_item(&mut self, from: ParticipantId, to: ParticipantId) -> Option<String> {
        let receiver = self.get(to);
        let item = self.get(from).item.as_ref()?;
        if receiver.hp == 0
            || receiver.item.is_some()
            || bound_to(&item_key(item), receiver)
            || self.active_gem.is_some_and(|(owner, _)| owner == to)
        {
            return None;
        }
        let item = self.take_item(from, to)?;
        self.get_mut(to).item = Some(item.clone());
        Some(item)
    }
}

pub(crate) fn gem_type(item: &str) -> Option<crate::model::PokemonType> {
    use crate::model::PokemonType::*;
    Some(match item_key(item).as_str() {
        "normalgem" => Normal,
        "firegem" => Fire,
        "watergem" => Water,
        "electricgem" => Electric,
        "grassgem" => Grass,
        "icegem" => Ice,
        "fightinggem" => Fighting,
        "poisongem" => Poison,
        "groundgem" => Ground,
        "flyinggem" => Flying,
        "psychicgem" => Psychic,
        "buggem" => Bug,
        "rockgem" => Rock,
        "ghostgem" => Ghost,
        "dragongem" => Dragon,
        "darkgem" => Dark,
        "steelgem" => Steel,
        "fairygem" => Fairy,
        _ => return Option::None,
    })
}

impl Battle {
    pub(crate) fn gem_factor(
        &mut self,
        actor: ParticipantId,
        kind: crate::model::PokemonType,
        events: &mut Vec<BattleEvent>,
    ) -> f32 {
        if self.active_gem == Some((actor, kind)) {
            return 1.3;
        }
        if !self.item_active(actor)
            || self.get(actor).item.as_deref().and_then(gem_type) != Some(kind)
        {
            return 1.0;
        }
        if let Some(item) = self.consume_item(actor) {
            self.active_gem = Some((actor, kind));
            events.push(BattleEvent::Message(format!(
                "{} used its {item}!",
                self.get(actor).name
            )));
            1.3
        } else {
            1.0
        }
    }
}
