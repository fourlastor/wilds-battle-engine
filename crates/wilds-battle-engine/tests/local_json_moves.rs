//! Behavior regressions for the local JSON moves outside the saved sheet.
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
fn sonic_boom_deals_twenty_regardless_of_stats_and_resistance() {
    for kind in [PokemonType::Normal, PokemonType::Rock] {
        let mut user = mon("User", &["sonic-boom"]);
        user.level = 1;
        user.sp_attack = 1;
        let mut foe = mon("Foe", &["splash"]);
        foe.sp_defense = 999;
        foe.types = vec![kind];
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 980);
    }
}

#[test]
fn sonic_boom_respects_immunity_accuracy_and_protect() {
    for (kind, response, dice) in [
        (PokemonType::Ghost, "splash", 0.5),
        (PokemonType::Normal, "splash", 0.95),
        (PokemonType::Normal, "protect", 0.5),
    ] {
        let mut foe = mon("Foe", &[response]);
        foe.types = vec![kind];
        let mut battle = game(mon("User", &["sonic-boom"]), vec![foe], dice);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
    }
}

#[test]
fn seismic_toss_uses_user_level_without_type_or_stat_scaling() {
    for (level, kind) in [(7, PokemonType::Rock), (73, PokemonType::Flying)] {
        let mut user = mon("User", &["seismic-toss"]);
        user.level = level;
        user.attack = 1;
        user.types = vec![PokemonType::Fighting];
        let mut foe = mon("Foe", &["splash"]);
        foe.defense = 999;
        foe.types = vec![kind];
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(
            battle.participants(Side::Foes)[0].hp,
            1000 - u16::from(level)
        );
    }
}

#[test]
fn seismic_toss_respects_ghost_immunity_and_substitute() {
    for ghost in [false, true] {
        let mut user = mon("User", &["seismic-toss"]);
        user.level = 50;
        let mut foe = mon("Foe", &["splash"]);
        if ghost {
            foe.types = vec![PokemonType::Ghost];
        } else {
            foe.substitute_hp = 20;
        }
        let mut battle = game(user, vec![foe], 0.5);
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
        assert_eq!(battle.participants(Side::Foes)[0].substitute_hp, 0);
    }
}

#[test]
fn dragon_darts_splits_two_hits_without_spread_damage_reduction() {
    let mut single = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut single);
    let double_damage = 1000 - single.participants(Side::Foes)[0].hp;
    assert!(double_damage > 0);
    let mut double = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Left", &["splash"]), mon("Right", &["splash"])],
        0.5,
    );
    turn(&mut double);
    for foe in double.participants(Side::Foes) {
        assert_eq!(2 * (1000 - foe.hp), double_damage);
    }
    assert_eq!(
        double.participants(Side::Allies)[0].move_pp["dragon-darts"],
        9
    );
}

#[test]
fn dragon_darts_concentrates_on_the_eligible_opponent() {
    use wilds_battle_engine::Stat;
    let mut single = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut single);
    let expected = single.participants(Side::Foes)[0].hp;
    for block in ["fairy", "protect", "evasion", "wonderguard", "fly"] {
        let mut blocked = mon(
            "Blocked",
            &[if block == "protect" {
                "protect"
            } else if block == "fly" {
                "fly"
            } else {
                "splash"
            }],
        );
        match block {
            "fairy" => blocked.types = vec![PokemonType::Fairy],
            "evasion" => {
                blocked.stages.insert(Stat::Evasion, 6);
            }
            "wonderguard" => blocked.ability = Some("wonder-guard".into()),
            "fly" => blocked.speed = 999,
            _ => {}
        }
        let mut battle = game(
            mon("User", &["dragon-darts"]),
            vec![blocked, mon("Other", &["splash"])],
            0.5,
        );
        turn(&mut battle);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 1000, "{block}");
        assert_eq!(battle.participants(Side::Foes)[1].hp, expected, "{block}");
        assert!(!battle.participants(Side::Allies)[0].last_move_failed);
    }
}

#[test]
fn dragon_darts_breaks_substitute_and_fails_if_every_target_is_immune() {
    let mut foe = mon("Foe", &["splash"]);
    foe.substitute_hp = 1;
    let mut battle = game(mon("User", &["dragon-darts"]), vec![foe], 0.5);
    turn(&mut battle);
    assert_eq!(battle.participants(Side::Foes)[0].substitute_hp, 0);
    assert!(battle.participants(Side::Foes)[0].hp < 1000);
    let foes = (0..2)
        .map(|_| {
            let mut p = mon("Fairy", &["splash"]);
            p.types = vec![PokemonType::Fairy];
            p
        })
        .collect();
    let mut battle = game(mon("User", &["dragon-darts"]), foes, 0.5);
    turn(&mut battle);
    assert!(battle.participants(Side::Allies)[0].last_move_failed);
    assert!(battle.participants(Side::Foes).iter().all(|p| p.hp == 1000));
}

#[test]
fn dragon_darts_respects_redirection_and_does_not_target_the_bench() {
    use wilds_battle_engine::{Condition, ParticipantId, Rule};
    let mut selected = mon("Selected", &["splash"]);
    selected.conditions.push(Condition {
        name: "redirect".into(),
        value: 1,
        turns_left: None,
        group: None,
        end_message: None,
        rules: vec![Rule::Redirect {
            to: ParticipantId {
                side: Side::Foes,
                index: 1,
            },
            move_type: None,
            priority: 1,
        }],
    });
    let mut battle = game(
        mon("User", &["dragon-darts"]),
        vec![
            selected,
            mon("Redirector", &["splash"]),
            mon("Bench", &["splash"]),
        ],
        0.5,
    );
    battle.set_active([vec![0], vec![0, 1]]).unwrap();
    turn(&mut battle);
    let foes = battle.participants(Side::Foes);
    assert_eq!(foes[0].hp, 1000);
    assert!(foes[1].hp < 1000);
    assert_eq!(foes[2].hp, 1000);
    let mut single = game(
        mon("User", &["dragon-darts"]),
        vec![mon("Foe", &["splash"])],
        0.5,
    );
    turn(&mut single);
    assert_eq!(foes[1].hp, single.participants(Side::Foes)[0].hp);
}

fn order_damage(move_id: &str, speed: u16, recharge: bool) -> u16 {
    let mut user = mon("User", &[move_id]);
    user.speed = speed;
    let mut foe = mon("Foe", &["splash"]);
    foe.recharging = recharge;
    let mut battle = game(user, vec![foe], 0.5);
    turn(&mut battle);
    1000 - battle.participants(Side::Foes)[0].hp
}

fn switched_damage(move_id: &str) -> u16 {
    let mut battle = game(
        mon("User", &[move_id]),
        vec![mon("Outgoing", &["splash"]), mon("Incoming", &["splash"])],
        0.5,
    );
    battle.set_active([vec![0], vec![0]]).unwrap();
    turn_with(&mut battle, |p| {
        p.choices
            .iter()
            .find(|c| {
                if p.actor.side == Side::Foes {
                    matches!(c, Choice::Switch { to, .. } if to.index == 1)
                } else {
                    matches!(c, Choice::UseMove { .. })
                }
            })
            .unwrap()
            .id()
    });
    assert_eq!(battle.participants(Side::Foes)[0].hp, 1000);
    1000 - battle.participants(Side::Foes)[1].hp
}

#[test]
fn bolt_beak_doubles_before_a_move_or_after_a_switch_but_not_recharge() {
    let base = order_damage("bolt-beak", 10, false);
    let boosted = order_damage("bolt-beak", 200, false);
    assert!(base > 0 && boosted > base * 18 / 10 && boosted <= base * 2);
    assert_eq!(order_damage("bolt-beak", 200, true), base);
    assert_eq!(switched_damage("bolt-beak"), boosted);
}
