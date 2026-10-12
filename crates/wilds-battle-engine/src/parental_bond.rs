//! A prepared two-hit attack survives ability changes caused by its first hit.
use crate::engine::Battle;
use crate::model::{BattleError, BattleEvent, ParticipantId, Status};
use crate::moves::{Category, Effect, MoveSpec};

pub(crate) struct ParentalHit {
    pub actor: ParticipantId,
    pub target: ParticipantId,
    pub move_id: String,
    pub hit: u8,
    pub connected: bool,
    pub recoil_damage: u16,
    pub recoil_fraction: Option<f32>,
    pub state: mlua::Table,
    pub native_lock: Option<(u8, u8)>,
}

impl Battle {
    pub(crate) fn parental_hit(&self, actor: ParticipantId, move_id: &str) -> Option<&ParentalHit> {
        self.parental_hits
            .last()
            .filter(|p| p.actor == actor && p.move_id == move_id)
    }
    pub(crate) fn begin_parental_hit(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        spec: &MoveSpec,
    ) -> Result<bool, BattleError> {
        if !self.has_ability(actor, "parentalbond")
            || spec.category == Category::Status
            || spec.auto_only
            || [
                "multi_hit",
                "charge",
                "future_move",
                "no_parental_bond",
                "calls_move",
            ]
            .iter()
            .any(|f| spec.has_flag(f))
            || spec
                .effects
                .iter()
                .any(|e| matches!(e, Effect::MultiHit { .. } | Effect::TwoTurn { .. }))
        {
            return Ok(false);
        }
        let targets = self.move_recipients(actor, target, spec).len();
        if targets != 1 {
            return Ok(false);
        }
        self.parental_hits.push(ParentalHit {
            actor,
            target,
            move_id: spec.id.clone(),
            hit: 1,
            connected: false,
            recoil_damage: 0,
            recoil_fraction: None,
            state: self.catalog.lua().create_table()?,
            native_lock: None,
        });
        Ok(true)
    }
    pub(crate) fn parental_has_next(&self, actor: ParticipantId, move_id: &str) -> bool {
        self.parental_hit(actor, move_id).is_some_and(|p| {
            p.hit == 1
                && p.connected
                && self.get(actor).hp > 0
                && self.get(p.target).hp > 0
                && self.get(actor).status != Some(Status::Asleep)
        })
    }
    pub(crate) fn next_parental_hit(&mut self, actor: ParticipantId, move_id: &str) -> bool {
        if !self.parental_has_next(actor, move_id) {
            return false;
        }
        let p = self.parental_hits.last_mut().unwrap();
        p.hit = 2;
        p.connected = false;
        self.substitute_hits.remove(&p.target);
        true
    }
    pub(crate) fn finish_parental_hit(&mut self, events: &mut Vec<BattleEvent>) {
        let p = self.parental_hits.pop().unwrap();
        if let Some(fraction) = p.recoil_fraction {
            self.recoil_from_hit(p.actor, &p.move_id, p.recoil_damage, fraction, events);
        }
        if p.connected {
            events.push(BattleEvent::Message(format!("Hit {} times!", p.hit)));
        }
    }
    pub(crate) fn recoil_from_hit(
        &mut self,
        actor: ParticipantId,
        move_id: &str,
        damage: u16,
        fraction: f32,
        events: &mut Vec<BattleEvent>,
    ) {
        if damage == 0 {
            return;
        }
        if let Some(p) = self
            .parental_hits
            .last_mut()
            .filter(|p| p.actor == actor && p.move_id == move_id)
        {
            p.recoil_damage = p.recoil_damage.saturating_add(damage);
            p.recoil_fraction = Some(fraction);
            return;
        }
        if !self.has_ability(actor, "magicguard") && !self.has_ability(actor, "rockhead") {
            self.damage(
                actor,
                ((f32::from(damage) * fraction).floor() as u16).max(1),
                events,
            );
            events.push(BattleEvent::Message(format!(
                "{} was damaged by the recoil!",
                self.get(actor).display_name()
            )));
        }
    }
}
