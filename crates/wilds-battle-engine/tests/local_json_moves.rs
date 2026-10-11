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
