//! Entry hazards are named side conditions, removable through the ordinary effect API.
use crate::engine::Battle;
use crate::model::msg;
use crate::model::{AppliedStatus, BattleEvent, ParticipantId, PokemonType, Scope, Side, Stat};
use crate::model::{MessageArg, say};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Hazard {
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
}
impl Hazard {
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "spikes" => Self::Spikes,
            "toxic_spikes" => Self::ToxicSpikes,
            "stealth_rock" => Self::StealthRock,
            "sticky_web" => Self::StickyWeb,
            _ => return None,
        })
    }
    fn name(self) -> &'static str {
        match self {
            Self::Spikes => "spikes",
            Self::ToxicSpikes => "toxic_spikes",
            Self::StealthRock => "stealth_rock",
            Self::StickyWeb => "sticky_web",
        }
    }
    fn max_layers(self) -> i32 {
        match self {
            Self::Spikes => 3,
            Self::ToxicSpikes => 2,
            _ => 1,
        }
    }
}
impl Battle {
    pub(crate) fn add_hazard(
        &mut self,
        side: Side,
        hazard: Hazard,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let scope = Scope::Side(side);
        let layers = self
            .conditions(scope)
            .iter()
            .find(|c| c.name == hazard.name())
            .map_or(0, |c| c.value);
        if layers >= hazard.max_layers() {
            return false;
        }
        self.set_mark(scope, hazard.name(), layers.max(0) + 1, None);
        events.push(say(
            match side {
                Side::Allies => msg("{0} was placed on your side!"),
                Side::Foes => msg("{0} was placed on the opposing side!"),
            },
            [MessageArg::Text(hazard.name().replace('_', " ").into())],
        ));
        true
    }

    pub(crate) fn entry_hazards(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        // Preserve placement order. A knockout prevents later hazards and entry abilities.
        let hazards: Vec<_> = self
            .conditions(Scope::Side(who.side))
            .iter()
            .filter_map(|c| Hazard::parse(&c.name).map(|h| (h, c.value.clamp(1, h.max_layers()))))
            .collect();
        let source = ParticipantId {
            side: who.side.opposite(),
            index: self.active_indices(who.side.opposite())[0],
        };
        for (hazard, layers) in hazards {
            if self.get(who).hp == 0 {
                break;
            }
            let grounded = self.on_ground(who);
            let fraction = match hazard {
                Hazard::Spikes if grounded => {
                    [0.0, 1.0 / 8.0, 1.0 / 6.0, 1.0 / 4.0][layers as usize]
                }
                Hazard::StealthRock => self.effectiveness(who, PokemonType::Rock) / 8.0,
                Hazard::ToxicSpikes if grounded => {
                    if self.types_of(who).contains(&PokemonType::Poison) {
                        self.end_condition(Scope::Side(who.side), hazard.name());
                        events.push(say("{0} absorbed the toxic spikes!", [self.who(who)]));
                    } else if !self.types_of(who).contains(&PokemonType::Steel) {
                        self.inflict_status_with_cause(
                            source,
                            who,
                            if layers == 2 {
                                AppliedStatus::BadlyPoisoned
                            } else {
                                AppliedStatus::Poisoned
                            },
                            false,
                            PokemonType::Poison,
                            false,
                            crate::statuses::StatusCause::Hazard,
                            events,
                        );
                    }
                    0.0
                }
                Hazard::StickyWeb if grounded => {
                    self.change_stat_from(source, who, Stat::Speed, -1, events);
                    0.0
                }
                _ => 0.0,
            };
            if fraction > 0.0 && !self.has_ability(who, "magicguard") {
                let amount = (f32::from(self.get(who).max_hp) * fraction)
                    .floor()
                    .max(1.0) as u16;
                self.damage(who, amount.min(self.get(who).hp), events);
            }
        }
    }
}
