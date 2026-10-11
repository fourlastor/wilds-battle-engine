//! Regression tests for the Sword/Shield status audit.
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
            .find(|c| matches!(c, Choice::UseMove { move_id, .. } if move_id == wanted))
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
fn freeze_fire_hits_thaw_before_the_targets_pending_action() {
    for attack in ["ember", "fire-punch", "fire-spin", "flamethrower"] {
        let mut user = mon("User", &[attack]);
        user.speed = 200;
        let mut target = mon("Target", &["tackle"]);
        target.status = Some(Status::Frozen);
        let mut battle = game(user, target, 0.5);
        let events = turn(&mut battle, attack, "tackle");
        assert_eq!(foe(&battle).status, None, "{attack}");
        assert!(
            ally(&battle).hp < 1000,
            "thawed target should act: {attack}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, BattleEvent::Message(m) if m == "Target thawed out!"))
        );
    }
}

#[test]
fn freeze_fire_status_misses_immunity_and_substitute_do_not_thaw() {
    for case in ["status", "miss", "flash-fire", "substitute"] {
        let attack = if case == "status" {
            "will-o-wisp"
        } else {
            "fire-blast"
        };
        let mut target = mon("Target", &["splash"]);
        target.status = Some(Status::Frozen);
        if case == "flash-fire" {
            target.ability = Some("flash-fire".into());
        }
        if case == "substitute" {
            target.substitute_hp = 500;
        }
        let mut battle = game(
            mon("User", &[attack]),
            target,
            if case == "miss" { 0.99 } else { 0.5 },
        );
        turn(&mut battle, attack, "splash");
        assert_eq!(foe(&battle).status, Some(Status::Frozen), "{case}");
    }
}

#[test]
fn freeze_uses_actual_attack_type_and_nonfire_thawing_moves() {
    for attack in ["weather-ball", "scald", "steam-eruption", "scorching-sands"] {
        let mut target = mon("Target", &["splash"]);
        target.status = Some(Status::Frozen);
        let mut battle = game(mon("User", &["sunny_day", attack]), target, 0.5);
        if attack == "weather-ball" {
            turn(&mut battle, "sunny_day", "splash");
            assert_eq!(
                foe(&battle).status,
                Some(Status::Frozen),
                "sun does not cure an existing freeze"
            );
        }
        turn(&mut battle, attack, "splash");
        assert_eq!(foe(&battle).status, None, "{attack}");
    }
}

#[test]
fn freeze_sun_and_magma_armor_prevent_freezing() {
    for case in ["plain", "sun", "magma", "suppressed", "ice"] {
        let mut target = mon("Target", &["splash"]);
        if case == "magma" || case == "suppressed" {
            target.ability = Some("magma-armor".into());
        }
        target.ability_suppressed = case == "suppressed";
        if case == "ice" {
            target.types = vec![PokemonType::Ice];
        }
        let mut battle = game(mon("User", &["sunny_day", "ice-beam"]), target, 0.05);
        if case == "sun" {
            turn(&mut battle, "sunny_day", "splash");
        }
        // Inspect the application event even if the target naturally thaws this turn.
        let events = turn(&mut battle, "ice-beam", "splash");
        let frozen = events.iter().any(|e| {
            matches!(
                e,
                BattleEvent::Status {
                    status: Some(Status::Frozen),
                    ..
                }
            )
        });
        assert_eq!(frozen, case == "plain" || case == "suppressed", "{case}");
    }
}

#[test]
fn freeze_magma_armor_cures_existing_freeze_without_consuming_a_berry() {
    let mut target = mon("Target", &["splash"]);
    target.status = Some(Status::Frozen);
    target.ability = Some("magma-armor".into());
    target.item = Some("aspear-berry".into());
    let mut battle = game(mon("User", &["splash"]), target, 0.5);
    turn(&mut battle, "splash", "splash");
    assert_eq!(foe(&battle).status, None);
    assert_eq!(foe(&battle).item.as_deref(), Some("aspear-berry"));
}

#[test]
fn freeze_burn_up_only_self_thaws_a_fire_type() {
    for fire in [false, true] {
        let mut user = mon("User", &["burn-up"]);
        user.status = Some(Status::Frozen);
        if fire {
            user.types = vec![PokemonType::Fire];
        }
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, "burn-up", "splash");
        assert_eq!(ally(&battle).status.is_none(), fire);
        assert_eq!(foe(&battle).hp < 1000, fire);
    }
}

#[test]
fn freeze_all_sword_shield_self_thawing_moves_work() {
    for attack in [
        "flame-wheel",
        "sacred-fire",
        "flare-blitz",
        "fusion-flare",
        "scald",
        "steam-eruption",
        "burn-up",
        "pyro-ball",
        "scorching-sands",
    ] {
        let mut user = mon("User", &[attack]);
        user.types = vec![PokemonType::Fire];
        user.status = Some(Status::Frozen);
        let mut battle = game(user, mon("Target", &["splash"]), 0.5);
        turn(&mut battle, attack, "splash");
        assert_eq!(ally(&battle).status, None, "{attack}");
        assert!(foe(&battle).hp < 1000, "{attack}");
    }
}

#[test]
fn freeze_sun_immunity_respects_weather_suppressing_abilities() {
    for ability in ["cloud-nine", "air-lock"] {
        let mut user = mon("User", &["sunny_day", "ice-beam"]);
        user.ability = Some(ability.into());
        let mut target = mon("Target", &["splash"]);
        target.speed = 200;
        let mut battle = game(user, target, 0.05);
        turn(&mut battle, "sunny_day", "splash");
        turn(&mut battle, "ice-beam", "splash");
        assert_eq!(foe(&battle).status, Some(Status::Frozen), "{ability}");
    }
}

#[test]
fn freeze_self_thaw_requires_getting_past_flinch() {
    let mut user = mon("User", &["pyro-ball"]);
    user.status = Some(Status::Frozen);
    let mut battle = game(user, mon("Target", &["fake-out"]), 0.5);
    turn(&mut battle, "pyro-ball", "fake-out");
    assert_eq!(ally(&battle).status, Some(Status::Frozen));
    assert_eq!(ally(&battle).move_pp["pyro-ball"], 5);
    assert_eq!(foe(&battle).hp, 1000);
}
