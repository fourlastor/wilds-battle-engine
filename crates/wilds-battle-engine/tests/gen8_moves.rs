//! Behavior regressions for the additional Gen 1–8 moves outside the local JSON catalog.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, MoveCatalog, Pokemon,
    PokemonType, Prompt, Side,
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

fn game(user: Pokemon, foes: Vec<Pokemon>, dice: f32) -> Battle {
    Battle::with_rng(
        Fixed(dice),
        [vec![user], foes],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap()
}

fn turn_with(battle: &mut Battle, mut select: impl FnMut(&Prompt) -> u32) -> Vec<BattleEvent> {
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
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: select(&prompt),
            })
            .unwrap();
    }
    panic!("turn did not advance");
}

fn turn(battle: &mut Battle) -> Vec<BattleEvent> {
    turn_with(battle, |prompt| {
        prompt
            .choices
            .iter()
            .find(|c| matches!(c, Choice::UseMove { .. }))
            .unwrap()
            .id()
    })
}

#[test]
fn whirlwind_phazes_through_protect_and_substitute_but_respects_anchoring() {
    for (ability, switched) in [(None, true), (Some("suction-cups"), false)] {
        let mut foe = mon("Foe", &["protect"]);
        foe.substitute_hp = 50;
        foe.ability = ability.map(str::to_owned);
        let mut b = game(
            mon("User", &["whirlwind"]),
            vec![foe, mon("Bench", &["splash"])],
            0.5,
        );
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        assert_eq!(b.active_indices(Side::Foes), &[usize::from(switched)]);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
    let mut b = game(
        mon("User", &["whirlwind"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
}

fn check_binding(move_id: &str, immune: PokemonType) {
    for (kind, response, sub, binds) in [
        (PokemonType::Normal, "splash", 0, true),
        (immune, "splash", 0, false),
        (PokemonType::Normal, "protect", 0, false),
        (PokemonType::Normal, "splash", 500, false),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(mon("User", &[move_id]), vec![foe], 0.5);
        turn(&mut b);
        let foe = &b.participants(Side::Foes)[0];
        assert_eq!(foe.bound.is_some(), binds);
        if binds {
            assert!((2..=4).contains(&foe.bound.as_ref().unwrap().turns_left));
            assert!(foe.hp < 875);
        }
    }
}

#[test]
fn wrap_binds_for_four_to_five_turns_only_after_reaching_the_target() {
    check_binding("wrap", PokemonType::Ghost);
}

#[test]
fn roar_bypasses_protect_and_substitute_but_soundproof_blocks_it() {
    for (ability, switched) in [
        (None, true),
        (Some("soundproof"), false),
        (Some("suction-cups"), false),
    ] {
        let mut foe = mon("Foe", &["protect"]);
        foe.substitute_hp = 50;
        foe.ability = ability.map(str::to_owned);
        let mut b = game(
            mon("User", &["roar"]),
            vec![foe, mon("Bench", &["splash"])],
            0.5,
        );
        b.set_active([vec![0], vec![0]]).unwrap();
        turn(&mut b);
        assert_eq!(b.active_indices(Side::Foes), &[usize::from(switched)]);
        assert_eq!(b.participants(Side::Foes)[0].hp, 1000);
    }
}
