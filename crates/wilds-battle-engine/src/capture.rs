//! Catching wild Pokémon with balls: Gen 8 catch rates, shake checks and Ball Fetch.
use crate::engine::Battle;
use crate::model::{BattleEvent, ContextValue, ParticipantId, Status, msg, say};

/// What a ball adds to the catch: see `ctx:attempt_capture`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Throw {
    pub(crate) modifier: f32,
    pub(crate) guaranteed: bool,
    /// Added to the target's catch rate (Heavy Ball), within 1..=255.
    pub(crate) catch_rate_bonus: i32,
}

impl Battle {
    fn context_number(&self, key: &str) -> Option<f64> {
        match self.context.get(key) {
            Some(ContextValue::Number(number)) => Some(*number),
            _ => None,
        }
    }

    /// Throws `ball` at `target`. A Pokémon that is uncatchable, has no catch rate, has
    /// fainted or is on the thrower's side refuses it, and the ball is not used up.
    pub(crate) fn attempt_capture(
        &mut self,
        actor: ParticipantId,
        target: ParticipantId,
        ball: &str,
        throw: Throw,
        events: &mut Vec<BattleEvent>,
    ) -> bool {
        let pokemon = self.get(target);
        let Some(rate) = pokemon
            .catch_rate
            .filter(|_| !pokemon.uncatchable && pokemon.hp > 0 && target.side != actor.side)
        else {
            events.push(BattleEvent::CaptureRefused {
                actor,
                target,
                ball: ball.to_owned(),
            });
            events.push(say("{0} can't be caught!", [self.who(target)]));
            self.item_kept = true;
            return false;
        };
        // Gen 8: X = (3M - 2H) * C * B / 3M * L * S * D.
        let max_hp = f64::from(pokemon.max_hp);
        let hp = f64::from(pokemon.hp);
        let catch_rate = f64::from((i32::from(rate) + throw.catch_rate_bonus).clamp(1, 255));
        let low_level = if pokemon.level < 21 {
            (30.0 - f64::from(pokemon.level)) / 10.0
        } else {
            1.0
        };
        let status = match pokemon.status {
            Some(Status::Asleep | Status::Frozen) => 2.5,
            Some(_) => 1.5,
            None => 1.0,
        };
        let difficulty = self.context_number("capture_difficulty").unwrap_or(1.0);
        let x = (3.0 * max_hp - 2.0 * hp) * catch_rate * f64::from(throw.modifier) / (3.0 * max_hp)
            * low_level
            * status
            * difficulty;
        let (shakes, critical, caught) = if throw.guaranteed || x >= 255.0 {
            (4, false, true)
        } else {
            let factor = self.context_number("critical_capture").unwrap_or(0.0);
            let threshold = (x.min(255.0) * factor / 6.0).floor() as usize;
            let critical = threshold > 0 && self.rng.index(256) < threshold;
            let hold = (65536.0 / (255.0 / x).powf(3.0 / 16.0)).floor() as usize;
            let checks = if critical { 1 } else { 4 };
            let mut passed = 0;
            while passed < checks && self.rng.index(65536) < hold {
                passed += 1;
            }
            (passed, critical, passed == checks)
        };
        events.push(BattleEvent::Capture {
            actor,
            target,
            ball: ball.to_owned(),
            shakes,
            critical,
            caught,
        });
        if caught {
            events.push(say("Gotcha! {0} was caught!", [self.who(target)]));
            self.captured = Some(target);
        } else {
            events.push(say(
                match shakes {
                    0 => msg("Oh no! The Pokémon broke free!"),
                    1 => msg("Aww! It appeared to be caught!"),
                    2 => msg("Aargh! Almost had it!"),
                    _ => msg("Gah! It was so close, too!"),
                },
                [],
            ));
            self.failed_ball.get_or_insert_with(|| ball.to_owned());
        }
        caught
    }

    /// Ball Fetch: at the end of a turn, an empty-handed holder picks up the first ball that
    /// failed this battle, once.
    pub(crate) fn ball_fetch(&mut self, events: &mut Vec<BattleEvent>) {
        let Some(ball) = self.failed_ball.clone() else {
            return;
        };
        if self.ball_fetched {
            return;
        }
        for who in self.actors() {
            if self.has_ability(who, "ballfetch") && self.get(who).item.is_none() {
                self.get_mut(who).item = Some(ball.clone());
                self.ball_fetched = true;
                events.push(say(
                    "{0} found a {1}!",
                    [
                        self.who(who),
                        crate::model::MessageArg::Text(ball.replace('-', " ").into()),
                    ],
                ));
                return;
            }
        }
    }
}
