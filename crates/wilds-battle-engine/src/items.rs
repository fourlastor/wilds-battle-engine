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
        let sticky_hold = source != who
            && pokemon.hp > 0
            && !pokemon.ability_suppressed
            && pokemon
                .ability
                .as_deref()
                .is_some_and(|id| item_key(id) == "stickyhold")
            && item_key(item) != "stickybarb";
        if sticky_hold || !self.item_removable(who) {
            return None;
        }
        self.get_mut(who).item.take()
    }

    pub(crate) fn steal_item(&mut self, from: ParticipantId, to: ParticipantId) -> Option<String> {
        let receiver = self.get(to);
        let item = self.get(from).item.as_ref()?;
        if receiver.hp == 0 || receiver.item.is_some() || bound_to(&item_key(item), receiver) {
            return None;
        }
        let item = self.take_item(from, to)?;
        self.get_mut(to).item = Some(item.clone());
        Some(item)
    }
}
