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

#[test]
fn mist_prevents_opposing_stat_drops_for_five_turns_without_refreshing() {
    use wilds_battle_engine::Stat;
    let mut user = mon("User", &["mist", "splash"]);
    user.speed = 200;
    let mut b = game(user, vec![mon("Foe", &["growl"])], 0.5);
    turn(&mut b);
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    turn(&mut b);
    assert!(b.participants(Side::Allies)[0].last_move_failed);
    for _ in 0..3 {
        turn_with(&mut b, |p| {
            p.choices.iter().find(|c| matches!(c, Choice::UseMove {move_id,..} if move_id == if p.actor.side == Side::Allies {"splash"} else {"growl"})).unwrap().id()
        });
    }
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), 0);
    assert!(b.side_conditions(Side::Allies).is_empty());
    turn_with(&mut b, |p| {
        p.choices.iter().find(|c| matches!(c, Choice::UseMove {move_id,..} if move_id == if p.actor.side == Side::Allies {"splash"} else {"growl"})).unwrap().id()
    });
    assert_eq!(b.participants(Side::Allies)[0].stage(Stat::Attack), -1);
}

fn control_damage(
    power: u16,
    kind: &str,
    category: &str,
    mut user: Pokemon,
    mut foe: Pokemon,
) -> u16 {
    let source = format!(
        "return {{{{id='control', name='Control', type=Type.{kind}, category=Category.{category}, pp=40, accuracy=Accuracy.Always, script=function(ctx) ctx:damage({power}) end}}, {{id='wait', name='Wait', type=Type.Normal, category=Category.Status, pp=40, target=Target.User, script=function(_) end}}}}"
    );
    user.moves = vec!["control".into()];
    foe.moves = vec!["wait".into()];
    let mut b = Battle::with_rng(
        Fixed(0.5),
        [vec![user], vec![foe]],
        MoveCatalog::from_lua(&source).unwrap(),
    )
    .unwrap();
    turn(&mut b);
    1000 - b.participants(Side::Foes)[0].hp
}

#[test]
fn low_kick_obeys_all_six_weight_brackets() {
    for (weight, power) in [
        (0.1, 20),
        (9.9, 20),
        (10.0, 40),
        (24.9, 40),
        (25.0, 60),
        (49.9, 60),
        (50.0, 80),
        (99.9, 80),
        (100.0, 100),
        (199.9, 100),
        (200.0, 120),
    ] {
        let user = mon("User", &["low-kick"]);
        let mut foe = mon("Foe", &["splash"]);
        foe.weight = weight;
        let expected = control_damage(power, "Fighting", "Physical", user.clone(), foe.clone());
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            1000 - b.participants(Side::Foes)[0].hp,
            expected,
            "weight {weight}"
        );
    }
}

#[test]
fn leech_seed_drains_one_eighth_and_heals_its_source_but_respects_grass_and_substitute() {
    for (kind, sub, response, seeded) in [
        (PokemonType::Normal, 0, "splash", true),
        (PokemonType::Grass, 0, "splash", false),
        (PokemonType::Normal, 50, "splash", false),
        (PokemonType::Normal, 0, "protect", false),
    ] {
        let mut user = mon("User", &["leech-seed"]);
        user.hp = 500;
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        foe.substitute_hp = sub;
        let mut b = game(user, vec![foe], 0.5);
        turn(&mut b);
        assert_eq!(
            b.participants(Side::Foes)[0].hp,
            if seeded { 875 } else { 1000 }
        );
        assert_eq!(
            b.participants(Side::Allies)[0].hp,
            if seeded { 625 } else { 500 }
        );
        if seeded {
            turn(&mut b);
            assert!(b.participants(Side::Allies)[0].last_move_failed);
            assert_eq!(b.participants(Side::Foes)[0].hp, 750);
        }
    }
}

fn check_partial_trap(move_id: &str, power: u16, kind: &str, category: &str) {
    for (response, sub, dice, binds) in [
        ("splash", 0, 0.5, true),
        ("protect", 0, 0.5, false),
        ("splash", 500, 0.5, false),
    ] {
        let user = mon("User", &[move_id]);
        let mut foe = mon("Foe", &[response]);
        foe.substitute_hp = sub;
        let expected = control_damage(power, kind, category, user.clone(), foe.clone());
        let mut b = game(user, vec![foe], dice);
        turn(&mut b);
        let foe = &b.participants(Side::Foes)[0];
        assert_eq!(foe.bound.is_some(), binds, "{move_id}");
        if binds {
            assert_eq!(1000 - foe.hp, expected + 125, "{move_id}");
            assert!((2..=4).contains(&foe.bound.as_ref().unwrap().turns_left));
        } else {
            assert_eq!(foe.hp, 1000);
        }
    }
}

#[test]
fn fire_spin_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("fire-spin", 35, "Fire", "Special");
}

#[test]
fn clamp_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("clamp", 35, "Water", "Physical");
}

#[test]
fn sand_tomb_damages_and_binds_only_after_a_reaching_hit() {
    check_partial_trap("sand-tomb", 35, "Ground", "Physical");
}
