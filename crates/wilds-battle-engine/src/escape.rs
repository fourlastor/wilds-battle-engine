//! Wild-battle escape is a host-selected party action, never an AI decision.
use crate::model::say;
use crate::{Battle, BattleError, BattleEvent, ParticipantId, PokemonType, Side};

impl Battle {
    /// Enable allied escape choices for a wild encounter. Defaults to false.
    /// Configure this before the first call to `advance`; trainer/scripted
    /// encounters should leave it disabled, even when a participant has Run Away.
    pub fn set_wild_battle(&mut self, enabled: bool) -> Result<(), BattleError> {
        if self.has_started() {
            return Err(BattleError::InvalidSetup(
                "wild battle mode must be configured before advancing".into(),
            ));
        }
        self.wild_battle = enabled;
        Ok(())
    }

    pub(crate) fn can_escape(&self, actor: ParticipantId) -> bool {
        self.wild_battle
            && !self.no_escape
            && self.is_active(actor)
            && self.get(actor).hp > 0
            && (self.guaranteed_escape(actor) || !self.held_in_battle(actor))
    }

    /// Trapped, bound, carrying or carried, or held by an ability.
    fn held_in_battle(&self, actor: ParticipantId) -> bool {
        self.is_trapped(actor)
            || self.get(actor).bound.is_some()
            || self.get(actor).carrying.is_some()
            || self.get(actor).carried_by.is_some()
            || self.ability_traps(actor)
    }

    /// Teleport in a wild battle: leaves without a roll, unless held in battle.
    pub(crate) fn flee(&mut self, actor: ParticipantId, events: &mut Vec<BattleEvent>) -> bool {
        if !self.wild_battle
            || self.no_escape
            || !self.is_active(actor)
            || (!self.guaranteed_escape(actor) && self.held_in_battle(actor))
        {
            return false;
        }
        self.end_by_fleeing(actor, events);
        true
    }

    /// `who` leaves a wild battle, which ends as an escape for its side.
    pub(crate) fn end_by_fleeing(&mut self, who: ParticipantId, events: &mut Vec<BattleEvent>) {
        self.escaped_by = Some(who.side);
        events.push(BattleEvent::EscapeAttempt {
            actor: who,
            success: true,
        });
        events.push(say("{0} fled!", [self.who(who)]));
    }

    fn guaranteed_escape(&self, actor: ParticipantId) -> bool {
        self.has_ability(actor, "runaway") || self.types_of(actor).contains(&PokemonType::Ghost)
    }

    pub(crate) fn try_escape(
        &mut self,
        actor: ParticipantId,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let success =
            self.can_escape(actor) && (self.guaranteed_escape(actor) || self.roll_escape(actor));
        events.push(BattleEvent::EscapeAttempt { actor, success });
        if success {
            self.escaped_by = Some(actor.side);
            events.push(say("{0} escaped!", [self.who(actor)]));
        } else {
            events.push(BattleEvent::Message("Couldn't escape!".into()));
        }
        success
    }

    fn roll_escape(&mut self, actor: ParticipantId) -> bool {
        // Compare raw Speed with the opposite active slot, falling back to a
        // living opponent. Stages, paralysis, weather and Trick Room don't apply.
        let foes = &self.active[Side::Foes.index()];
        let opposite = self
            .position(actor)
            .and_then(|slot| foes.get(slot))
            .copied();
        let foe = opposite
            .filter(|&index| self.sides[Side::Foes.index()][index].hp > 0)
            .or_else(|| {
                foes.iter()
                    .copied()
                    .find(|&index| self.sides[Side::Foes.index()][index].hp > 0)
            });
        let Some(foe) = foe else {
            return true;
        };
        let speed = u32::from(self.get(actor).speed);
        let enemy_speed = u32::from(self.sides[Side::Foes.index()][foe].speed);
        if speed >= enemy_speed {
            return true;
        }
        // Classic main-series escape formula; see README for its verification
        // boundary. The counter starts at zero and survives moves and switches.
        let threshold = ((speed * 128 / enemy_speed) + 30 * u32::from(self.escape_attempts)) as u8;
        self.escape_attempts = self.escape_attempts.wrapping_add(1);
        self.rng.range(0, 255) < threshold
    }
}
