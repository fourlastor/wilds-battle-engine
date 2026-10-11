//! Regression tests for the Sword/Shield ability audit.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog, Pokemon,
    PokemonType, Side, Status,
};

struct Fixed(f32);
impl BattleRng for Fixed {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        self.0
    }
}

fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|m| m.to_string()).collect());
    p.hp = 1000;
    p.max_hp = 1000;
    p.types = vec![PokemonType::Normal];
    p
}

fn game(user: Pokemon, target: Pokemon, dice: f32) -> Battle {
    Battle::with_rng(
        Fixed(dice),
        [vec![user], vec![target]],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap()
}

fn turn(battle: &mut Battle, ally: &str, foe: &str) -> Vec<BattleEvent> {
    let start = battle.turn();
    let mut events = vec![];
    for _ in 0..30 {
        let result = battle.advance().unwrap();
        events.extend(result.events);
        if battle.turn() > start {
            return events;
        }
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            return events;
        };
        let wanted = if prompt.actor.side == Side::Allies {
            ally
        } else {
            foe
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| match c {
                Choice::UseMove { move_id, .. } => move_id == wanted,
                Choice::Switch { to, .. } => wanted == format!("switch:{}", to.index),
            })
            .unwrap();
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: choice.id(),
            })
            .unwrap();
    }
    panic!("turn did not finish");
}

fn foe(battle: &Battle) -> &Pokemon {
    &battle.participants(Side::Foes)[0]
}
fn ally(battle: &Battle) -> &Pokemon {
    &battle.participants(Side::Allies)[0]
}

#[test]
fn volt_absorb_catches_status_moves_and_respects_suppression_and_bypass() {
    for (suppressed, breaker, hp, expected_hp, expected_status) in [
        (false, false, 500, 750, None),
        (false, false, 1000, 1000, None),
        (true, false, 500, 500, Some(Status::Paralyzed)),
        (false, true, 500, 500, Some(Status::Paralyzed)),
    ] {
        let mut user = mon("User", &["thunder-wave"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.hp = hp;
        target.ability = Some("volt-absorb".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "thunder-wave", "splash");
        assert_eq!(foe(&battle).hp, expected_hp);
        assert_eq!(foe(&battle).status, expected_status);
        assert_eq!(ally(&battle).hp, 1000);
    }
}

#[test]
fn water_absorb_catches_soak_without_changing_types() {
    for (suppressed, breaker) in [(false, false), (true, false), (false, true)] {
        let mut user = mon("User", &["soak"]);
        user.ability = breaker.then(|| "mold-breaker".into());
        let mut target = mon("Target", &["splash"]);
        target.hp = 500;
        target.ability = Some("water-absorb".into());
        target.ability_suppressed = suppressed;
        let mut battle = game(user, target, 0.5);
        turn(&mut battle, "soak", "splash");
        let active = !suppressed && !breaker;
        assert_eq!(foe(&battle).hp, if active { 750 } else { 500 });
        assert_eq!(
            foe(&battle).types,
            vec![if active {
                PokemonType::Normal
            } else {
                PokemonType::Water
            }]
        );
    }
}
