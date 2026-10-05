//! Plays the moves of `sheet_moves.lua`: every one of them in a handful of battles, and a closer
//! look at the ones that show a feature best.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, Gender, HiddenKind,
    MoveCatalog, ParticipantId, Pokemon, PokemonType, Side, Stat, Status, WeatherKind,
};

const SOURCE: &str = include_str!("sheet_moves.lua");

fn catalog() -> MoveCatalog {
    MoveCatalog::from_lua(SOURCE).expect("the moves load")
}

fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut pokemon = Pokemon::new(name, moves.iter().map(|id| id.to_string()).collect());
    pokemon.hp = 300;
    pokemon.max_hp = 300;
    pokemon
}

/// Dice that always give the same number: `chance(p)` succeeds when `p` is at least that number.
struct FixedRng(f32);
impl BattleRng for FixedRng {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        self.0
    }
}

struct Game {
    battle: Battle,
}

impl Game {
    fn with_dice(dice: f32, allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Self {
        Self {
            battle: Battle::with_rng(FixedRng(dice), [allies, foes], catalog()).unwrap(),
        }
    }
    /// 50% chances succeed, nothing misses and nothing is a critical hit.
    fn new(allies: Vec<Pokemon>, foes: Vec<Pokemon>) -> Self {
        Self::with_dice(0.5, allies, foes)
    }
    fn duel(ally: &[&str], foe: &[&str]) -> Self {
        Self::new(vec![mon("Ally", ally)], vec![mon("Foe", foe)])
    }
    /// Plays a turn. Each Pokémon that is asked takes the next pick: a move id, `id>foes` or
    /// `id>allies` to aim at that side, or `id>2` for the second Pokémon on offer.
    fn turn(&mut self, picks: &[&str]) -> Vec<String> {
        let mut picks = picks.iter();
        loop {
            let result = self.battle.advance().unwrap();
            if !result.events.is_empty() {
                return result
                    .events
                    .into_iter()
                    .filter_map(|event| match event {
                        BattleEvent::Message(text) => Some(text),
                        _ => None,
                    })
                    .collect();
            }
            let AdvanceStatus::Awaiting(prompt) = result.status else {
                return Vec::new();
            };
            let pick = picks
                .next()
                .expect("a pick for every Pokémon that is asked");
            let (id, aim) = pick.split_once('>').unwrap_or((pick, ""));
            let offered: Vec<&Choice> = prompt
                .choices
                .iter()
                .filter(|choice| {
                    let Choice::UseMove {
                        move_id, target, ..
                    } = choice;
                    move_id == id
                        && match aim {
                            "foes" => target.side == Side::Foes,
                            "allies" => target.side == Side::Allies,
                            _ => true,
                        }
                })
                .collect();
            let position = aim.parse::<usize>().map_or(0, |number| number - 1);
            let choice = offered
                .get(position)
                .unwrap_or_else(|| panic!("{pick} is not on offer: {:?}", prompt.choices));
            self.battle
                .set_response(ActionSelection {
                    prompt_id: prompt.id,
                    choice_id: choice.id(),
                })
                .unwrap();
        }
    }
    fn ally(&self, index: usize) -> &Pokemon {
        &self.battle.participants(Side::Allies)[index]
    }
    fn foe(&self, index: usize) -> &Pokemon {
        &self.battle.participants(Side::Foes)[index]
    }
    fn dealt(&self) -> u16 {
        self.foe(0).max_hp - self.foe(0).hp
    }
}

fn said(messages: &[String], text: &str) -> bool {
    messages.iter().any(|message| message.contains(text))
}

/// Damage the first foe takes when a fresh Ally uses `id` once, after `before` turns of setup.
fn damage_after(setup: &[[&str; 2]], id: &str, ally: &[&str], foe: &[&str]) -> u16 {
    let mut game = Game::duel(ally, foe);
    for picks in setup {
        game.turn(picks);
    }
    let before = game.foe(0).hp;
    game.turn(&[id, "wait"]);
    before - game.foe(0).hp
}

// ---------------------------------------------------------------------------
// Every move, in battles where the dice are real

/// Who stands where in a battle that tries a move out, and what else is going on.
struct Scene {
    name: &'static str,
    partner: &'static str,
    foes: [&'static str; 2],
    dress: fn(&mut [Vec<Pokemon>; 2]),
    place: Option<&'static str>,
}

fn scenes() -> Vec<Scene> {
    vec![
        Scene {
            name: "two on two",
            partner: "wait",
            foes: ["scratch", "ember"],
            dress: |_| {},
            place: None,
        },
        Scene {
            name: "rain, items and abilities",
            partner: "rain_dance",
            foes: ["scratch", "wait"],
            dress: |sides| {
                sides[0][0].ability = Some("plus".into());
                sides[0][0].gender = Gender::Male;
                sides[0][0].species = "chatot".into();
                sides[0][0].ivs = Some([30, 31, 30, 31, 30, 31]);
                sides[0][0].types = vec![PokemonType::Fire, PokemonType::Flying];
                sides[1][0].item = Some("oran_berry".into());
                sides[1][0].gender = Gender::Female;
                sides[1][1].item = Some("flame_plate".into());
                sides[1][1].types = vec![PokemonType::Grass];
            },
            place: Some("cave"),
        },
        Scene {
            name: "hail and foes that hide",
            partner: "hail",
            foes: ["fly", "dig"],
            dress: |sides| {
                sides[0][0].item = Some("flame_plate".into());
                sides[1][0].types = vec![PokemonType::Flying];
                sides[1][0].speed = 150;
                sides[1][1].speed = 140;
            },
            place: Some("snow"),
        },
        Scene {
            name: "protection, screens and gravity",
            partner: "gravity",
            foes: ["guard", "reflect"],
            dress: |sides| {
                sides[1][0].status = Some(Status::Paralyzed);
                sides[1][1].status = Some(Status::Asleep);
                sides[1][1].status_turns = 2;
                sides[0][0].status = Some(Status::Burned);
            },
            place: None,
        },
        Scene {
            name: "frail foes, marks and seeds",
            partner: "minimize",
            foes: ["leech_seed", "minimize"],
            dress: |sides| {
                for foe in &mut sides[1] {
                    foe.hp = 60;
                    foe.max_hp = 60;
                }
                sides[0][0].hp = 120;
            },
            place: Some("grass"),
        },
    ]
}

#[test]
fn every_move_plays_through_without_an_error() {
    let catalog = catalog();
    // The engine adds Struggle to every list of moves; no Pokémon can know it.
    let mut ids: Vec<String> = catalog
        .ids()
        .filter(|id| *id != "struggle")
        .map(str::to_owned)
        .collect();
    ids.sort();
    assert!(ids.len() > 120, "{} moves", ids.len());
    let mut battles = 0;
    for id in &ids {
        for scene in scenes() {
            for seed in 1..=3 {
                let mut sides = [
                    vec![mon("Ally", &[id, "wait"]), mon("Partner", &[scene.partner])],
                    vec![mon("Foe", &[scene.foes[0]]), mon("Other", &[scene.foes[1]])],
                ];
                (scene.dress)(&mut sides);
                let mut battle = Battle::new(seed, sides, catalog.clone()).unwrap();
                battle.set_environment(scene.place.map(str::to_owned));
                let lead = ParticipantId {
                    side: Side::Allies,
                    index: 0,
                };
                for step in 0..200 {
                    let result = battle.advance().unwrap_or_else(|error| {
                        panic!("{id} in “{}”, seed {seed}: {error}", scene.name)
                    });
                    let AdvanceStatus::Awaiting(prompt) = result.status else {
                        break;
                    };
                    if battle.turn() >= 6 {
                        break;
                    }
                    // The lead keeps using the move, at each Pokémon it can be aimed at in turn.
                    let mine: Vec<&Choice> = prompt
                        .choices
                        .iter()
                        .filter(|choice| {
                            let Choice::UseMove { move_id, .. } = choice;
                            prompt.actor == lead && move_id == id
                        })
                        .collect();
                    let choice = mine
                        .get(step % mine.len().max(1))
                        .copied()
                        .unwrap_or(&prompt.choices[0]);
                    battle
                        .set_response(ActionSelection {
                            prompt_id: prompt.id,
                            choice_id: choice.id(),
                        })
                        .unwrap();
                }
                for pokemon in Side::ALL
                    .into_iter()
                    .flat_map(|side| battle.participants(side))
                {
                    assert!(pokemon.hp <= pokemon.max_hp, "{id}: {}", pokemon.name);
                }
                battles += 1;
            }
        }
    }
    assert_eq!(battles, ids.len() * 15);
}

// ---------------------------------------------------------------------------
// The moves the sheet marks "Major issues"

#[test]
fn skull_bash_raises_defense_then_hits_on_the_next_turn() {
    let mut game = Game::duel(&["skull_bash"], &["wait"]);
    let first = game.turn(&["skull_bash", "wait"]);
    assert!(said(&first, "Ally tucked in its head!") && !said(&first, "used Skull Bash"));
    assert_eq!((game.ally(0).stage(Stat::Defense), game.dealt()), (1, 0));
    // The second turn is not chosen: the foe is the only one asked.
    assert!(said(&game.turn(&["wait"]), "Ally used Skull Bash!"));
    assert!(game.dealt() > 0);
}

#[test]
fn high_jump_kick_hurts_its_user_when_it_does_not_land_and_fails_in_gravity() {
    let mut game = Game::duel(&["high_jump_kick", "gravity"], &["guard", "wait"]);
    let blocked = game.turn(&["high_jump_kick", "guard"]);
    assert!(said(&blocked, "kept going and crashed!"));
    assert_eq!((game.ally(0).hp, game.dealt()), (150, 0));
    game.turn(&["high_jump_kick", "wait"]);
    assert!(game.dealt() > 0 && game.ally(0).hp == 150);
    game.turn(&["gravity", "wait"]);
    let hp = game.foe(0).hp;
    assert!(said(
        &game.turn(&["high_jump_kick", "wait"]),
        "But it failed!"
    ));
    assert_eq!((game.foe(0).hp, game.ally(0).hp), (hp, 150));
}

#[test]
fn a_struggle_ignores_types_and_costs_a_quarter_of_max_hp() {
    let mut foe = mon("Foe", &["wait"]);
    foe.types = vec![PokemonType::Ghost];
    let mut game = Game::new(vec![mon("Ally", &["struggle_like"])], vec![foe]);
    game.turn(&["struggle_like", "wait"]);
    assert!(game.dealt() > 0);
    assert_eq!(game.ally(0).hp, 225);
}

#[test]
fn thief_takes_the_item_only_when_its_user_holds_none() {
    let mut foe = mon("Foe", &["wait"]);
    foe.item = Some("oran_berry".into());
    let mut game = Game::new(vec![mon("Ally", &["thief"])], vec![foe.clone()]);
    assert!(said(
        &game.turn(&["thief", "wait"]),
        "Ally stole Foe's oran_berry!"
    ));
    assert_eq!(game.ally(0).item.as_deref(), Some("oran_berry"));
    assert_eq!(game.foe(0).item, None);

    let mut holder = mon("Ally", &["thief"]);
    holder.item = Some("leftovers".into());
    let mut game = Game::new(vec![holder], vec![foe]);
    game.turn(&["thief", "wait"]);
    assert!(game.dealt() > 0);
    assert_eq!(game.foe(0).item.as_deref(), Some("oran_berry"));
}

#[test]
fn fury_cutter_doubles_each_time_it_lands_and_starts_over_after_a_miss() {
    let mut game = Game::duel(&["fury_cutter"], &["wait"]);
    let mut damage = Vec::new();
    for _ in 0..3 {
        let before = game.foe(0).hp;
        game.turn(&["fury_cutter", "wait"]);
        damage.push(f32::from(before - game.foe(0).hp));
    }
    assert!(damage[1] / damage[0] > 1.8 && damage[2] / damage[1] > 1.8);
    assert_eq!(game.ally(0).condition("fury_cutter").unwrap().value, 3);

    // Dice of 0.99 make a 95% move miss.
    let mut game = Game::with_dice(
        0.99,
        vec![mon("Ally", &["fury_cutter"])],
        vec![mon("Foe", &["wait"])],
    );
    game.turn(&["fury_cutter", "wait"]);
    assert!(game.ally(0).condition("fury_cutter").is_none());
}

#[test]
fn rapid_spin_frees_its_user_and_clears_its_seeds() {
    let mut game = Game::duel(&["rapid_spin", "wait"], &["leech_seed", "wait"]);
    // 90% accuracy lands with these dice.
    assert!(said(
        &game.turn(&["wait", "leech_seed"]),
        "Ally was seeded!"
    ));
    assert!(game.ally(0).hp < 300 && game.ally(0).condition("leech_seed").is_some());
    game.turn(&["rapid_spin", "wait"]);
    assert!(game.ally(0).condition("leech_seed").is_none());
    let hp = game.ally(0).hp;
    game.turn(&["wait", "wait"]);
    assert_eq!(game.ally(0).hp, hp);
}

#[test]
fn hidden_power_takes_its_type_and_power_from_the_ivs() {
    let run = |ivs: [u8; 6], foe_type: PokemonType| {
        let mut ally = mon("Ally", &["hidden_power"]);
        ally.ivs = Some(ivs);
        let mut foe = mon("Foe", &["wait"]);
        foe.types = vec![foe_type];
        let mut game = Game::new(vec![ally], vec![foe]);
        (game.turn(&["hidden_power", "wait"]), game.dealt())
    };
    // All odd: the last type of the list, Dark, at full power.
    let (dark, full) = run([31; 6], PokemonType::Psychic);
    assert!(said(&dark, "It's super effective!"));
    // All even: the first type, Fighting.
    let (fighting, _) = run([30; 6], PokemonType::Ghost);
    assert!(said(&fighting, "It doesn't affect Foe..."), "{fighting:?}");
    // Multiples of four have neither bit set: Fighting again, at the lowest power.
    let (_, weak) = run([28; 6], PokemonType::Psychic);
    let (_, strong) = run([30; 6], PokemonType::Psychic);
    assert!(weak < strong && strong < full);
}

#[test]
fn swallow_spends_what_stockpile_stored() {
    let mut ally = mon("Ally", &["stockpile", "swallow"]);
    ally.hp = 100;
    let mut game = Game::new(vec![ally], vec![mon("Foe", &["wait"])]);
    assert!(said(&game.turn(&["swallow", "wait"]), "But it failed!"));
    game.turn(&["stockpile", "wait"]);
    assert!(said(
        &game.turn(&["stockpile", "wait"]),
        "Ally stockpiled 2!"
    ));
    assert_eq!(game.ally(0).stage(Stat::Defense), 2);
    game.turn(&["swallow", "wait"]);
    assert_eq!(game.ally(0).hp, 250);
    assert_eq!(
        (
            game.ally(0).stage(Stat::Defense),
            game.ally(0).stage(Stat::SpDefense)
        ),
        (0, 0)
    );
    assert!(game.ally(0).condition("stockpile").is_none());
}

#[test]
fn focus_punch_needs_a_turn_without_being_hit() {
    let mut game = Game::duel(&["focus_punch"], &["scratch", "wait"]);
    let hit = game.turn(&["focus_punch", "scratch"]);
    assert_eq!(hit[0], "Ally is tightening its focus!");
    assert!(said(&hit, "lost its focus") && game.dealt() == 0);
    game.turn(&["focus_punch", "wait"]);
    assert!(game.dealt() > 0);
}

#[test]
fn secret_power_depends_on_where_the_battle_is() {
    let run = |place: &str| {
        // Low enough for a 30% effect, and too high for a frozen Pokémon to thaw at once.
        let mut game = Game::with_dice(
            0.28,
            vec![mon("Ally", &["secret_power"])],
            vec![mon("Foe", &["wait"])],
        );
        game.battle.set_environment(Some(place.into()));
        game.turn(&["secret_power", "wait"]);
        (game.foe(0).status, game.foe(0).stage(Stat::Accuracy))
    };
    assert_eq!(run("snow"), (Some(Status::Frozen), 0));
    assert_eq!(run("desert"), (None, -1));
    assert_eq!(run("building"), (Some(Status::Paralyzed), 0));
}

// ---------------------------------------------------------------------------
// What the sheet's notes said was missing

#[test]
fn stomp_is_stronger_against_a_pokemon_that_minimized() {
    let plain = damage_after(&[], "stomp", &["stomp"], &["wait"]);
    let small = damage_after(
        &[["stomp", "minimize"]],
        "stomp",
        &["stomp"],
        &["minimize", "wait"],
    );
    assert!(f32::from(small) > f32::from(plain) * 1.8);
}

#[test]
fn defense_curl_doubles_a_rolling_move_which_doubles_every_turn() {
    let mut game = Game::duel(&["ice_ball"], &["wait"]);
    game.turn(&["ice_ball", "wait"]);
    let first = game.dealt();
    game.turn(&["wait"]);
    let second = game.dealt() - first;
    assert!(f32::from(second) > f32::from(first) * 1.8);
    let curled = damage_after(
        &[["defense_curl", "wait"]],
        "ice_ball",
        &["defense_curl", "ice_ball"],
        &["wait"],
    );
    assert!(f32::from(curled) > f32::from(first) * 1.8);
}

#[test]
fn moves_reach_the_hiding_places_they_name_with_double_power() {
    for (hider, striker, place) in [
        ("fly", "gust", HiddenKind::Air),
        ("dig", "earthquake", HiddenKind::Underground),
    ] {
        let mut fast = mon("Foe", &[hider, "wait"]);
        fast.speed = 200;
        let mut game = Game::new(vec![mon("Ally", &[striker, "scratch"])], vec![fast.clone()]);
        game.turn(&[striker, "wait"]);
        let open = game.dealt();
        let mut game = Game::new(vec![mon("Ally", &[striker, "scratch"])], vec![fast]);
        let up = game.turn(&["scratch", hider]);
        assert_eq!(game.foe(0).hidden(), Some(place));
        assert!(said(&up, "Foe avoided the attack!"));
        let mut game = Game::new(
            vec![mon("Ally", &[striker, "scratch"])],
            vec![{
                let mut foe = mon("Foe", &[hider, "wait"]);
                foe.speed = 200;
                foe
            }],
        );
        game.turn(&[striker, hider]);
        assert!(
            f32::from(game.dealt()) > f32::from(open) * 1.8,
            "{striker} against {hider}"
        );
    }
}

#[test]
fn gravity_keeps_everyone_on_the_ground() {
    let mut bird = mon("Foe", &["fly", "wait"]);
    bird.types = vec![PokemonType::Flying];
    bird.speed = 200;
    let mut game = Game::new(vec![mon("Ally", &["gravity", "dig", "wait"])], vec![bird]);
    // The bird is brought down, cannot fly up again, and Ground moves now hit it.
    let lines = game.turn(&["gravity", "fly"]);
    assert!(said(
        &lines,
        "Foe couldn't stay airborne because of gravity!"
    ));
    assert_eq!(game.foe(0).hidden(), None);
    assert!(said(&game.turn(&["wait", "fly"]), "But it failed!"));
    game.turn(&["dig", "wait"]);
    game.turn(&["wait"]);
    assert!(game.dealt() > 0);
}

#[test]
fn weather_changes_thunder_and_weather_ball() {
    // 70% accuracy misses on dice of 0.8, except in rain.
    let thunder = |partner: &str| {
        let mut game = Game::with_dice(
            0.8,
            vec![mon("Ally", &["thunder"]), mon("Partner", &[partner])],
            vec![mon("Foe", &["wait"])],
        );
        game.turn(&["thunder", partner, "wait"]);
        game.turn(&["thunder", partner, "wait"]);
        game.dealt()
    };
    assert_eq!(thunder("wait"), 0);
    assert!(thunder("rain_dance") > 0);

    let mut foe = mon("Foe", &["wait"]);
    foe.types = vec![PokemonType::Fire];
    let mut game = Game::new(
        vec![
            mon("Ally", &["weather_ball", "wait"]),
            mon("Partner", &["rain_dance"]),
        ],
        vec![foe],
    );
    game.turn(&["weather_ball", "rain_dance", "wait"]);
    let normal = game.dealt();
    let wet = game.turn(&["weather_ball", "rain_dance", "wait"]);
    assert!(said(&wet, "It's super effective!"));
    assert!(game.dealt() - normal > normal * 3);
    assert_eq!(game.battle.weather().unwrap().kind, WeatherKind::Rain);
}

#[test]
fn a_screen_halves_hits_until_brick_break_shatters_it() {
    let mut game = Game::duel(&["reflect", "wait"], &["scratch", "brick_break"]);
    game.turn(&["wait", "scratch"]);
    let open = 300 - game.ally(0).hp;
    game.turn(&["reflect", "scratch"]);
    game.turn(&["wait", "scratch"]);
    let walled = 300 - open * 2 - game.ally(0).hp;
    // The second hit came before the screen went up; the third is the one it halves.
    assert!(walled * 2 <= open + 1, "{walled} against {open}");
    assert!(said(
        &game.turn(&["wait", "brick_break"]),
        "It shattered the barrier!"
    ));
    assert!(game.battle.side_conditions(Side::Allies).is_empty());
}

#[test]
fn an_uproar_wakes_everyone_and_keeps_them_awake() {
    let mut sleeper = mon("Partner", &["wait"]);
    sleeper.status = Some(Status::Asleep);
    sleeper.status_turns = 3;
    let spore = "wait";
    let mut game = Game::new(
        vec![mon("Ally", &["uproar"]), sleeper],
        vec![mon("Foe", &[spore])],
    );
    let lines = game.turn(&["uproar", "wait", "wait"]);
    assert!(said(&lines, "Partner woke up!"));
    assert_eq!(game.ally(1).status, None);
    assert!(
        game.battle
            .field_conditions()
            .iter()
            .any(|condition| condition.name == "uproar")
    );
}

#[test]
fn sucker_punch_and_fake_out_depend_on_the_turn() {
    let mut game = Game::duel(&["sucker_punch", "fake_out"], &["wait", "scratch"]);
    assert!(said(
        &game.turn(&["sucker_punch", "wait"]),
        "But it failed!"
    ));
    game.turn(&["sucker_punch", "scratch"]);
    assert!(game.dealt() > 0);
    // Not the first turn out any more.
    assert!(said(&game.turn(&["fake_out", "wait"]), "But it failed!"));

    let mut game = Game::duel(&["fake_out"], &["scratch"]);
    assert!(said(&game.turn(&["fake_out", "scratch"]), "Foe flinched!"));
    assert_eq!(game.ally(0).hp, 300);
}

#[test]
fn round_brings_its_other_users_forward_at_double_power() {
    let mut slow = mon("Partner", &["round"]);
    slow.speed = 1;
    let mut fast = mon("Ally", &["round"]);
    fast.speed = 200;
    let mut middle = mon("Foe", &["scratch"]);
    middle.speed = 100;
    middle.hp = 1000;
    middle.max_hp = 1000;
    let mut game = Game::new(vec![fast, slow], vec![middle]);
    let lines = game.turn(&["round", "round", "scratch"]);
    let order: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains(" used "))
        .map(String::as_str)
        .collect();
    assert_eq!(
        order,
        [
            "Ally used Round!",
            "Partner used Round!",
            "Foe used Scratch!"
        ]
    );
}

#[test]
fn two_pledges_in_one_turn_leave_something_behind() {
    let mut game = Game::new(
        vec![
            mon("Ally", &["grass_pledge"]),
            mon("Partner", &["water_pledge"]),
        ],
        vec![mon("Foe", &["wait"])],
    );
    let speed = game.foe(0).speed;
    let lines = game.turn(&["grass_pledge", "water_pledge", "wait"]);
    assert!(said(&lines, "A swamp enveloped the opposing team!"));
    let swamp = &game.battle.side_conditions(Side::Foes)[0];
    assert_eq!((swamp.name.as_str(), swamp.turns_left), ("swamp", Some(3)));
    assert_eq!(game.foe(0).speed, speed, "the stat itself is untouched");
}

#[test]
fn lock_on_makes_a_one_hit_knockout_sure() {
    let mut game = Game::duel(&["lock_on", "sheer_cold"], &["wait"]);
    assert!(said(
        &game.turn(&["sheer_cold", "wait"]),
        "Ally's attack missed!"
    ));
    game.turn(&["lock_on", "wait"]);
    assert!(said(&game.turn(&["sheer_cold", "wait"]), "one-hit KO"));
    assert_eq!(game.foe(0).hp, 0);
}

#[test]
fn endure_leaves_one_hp() {
    let mut frail = mon("Ally", &["endure"]);
    frail.hp = 5;
    let mut game = Game::new(vec![frail], vec![mon("Foe", &["scratch"])]);
    assert!(said(
        &game.turn(&["endure", "scratch"]),
        "Ally braced itself!"
    ));
    assert_eq!(game.ally(0).hp, 1);
}

#[test]
fn spectral_thief_takes_the_boosts_before_it_hits() {
    // A Ghost move does nothing to the Normal type a Pokémon has by default.
    let psychic = || {
        let mut foe = mon("Foe", &["wait"]);
        foe.types = vec![PokemonType::Fighting];
        foe
    };
    let mut game = Game::new(vec![mon("Ally", &["spectral_thief"])], vec![psychic()]);
    game.turn(&["spectral_thief", "wait"]);
    let plain = game.dealt();
    let mut foe = psychic();
    foe.change_stage(Stat::Attack, 2);
    let mut game = Game::new(vec![mon("Ally", &["spectral_thief"])], vec![foe]);
    game.turn(&["spectral_thief", "wait"]);
    assert_eq!(
        (
            game.ally(0).stage(Stat::Attack),
            game.foe(0).stage(Stat::Attack)
        ),
        (2, 0)
    );
    assert!(f32::from(game.dealt()) > f32::from(plain) * 1.8);
}

#[test]
fn roost_and_burn_up_change_what_type_their_user_is() {
    let mut bird = mon("Ally", &["roost", "burn_up", "wait"]);
    bird.types = vec![PokemonType::Fire, PokemonType::Flying];
    bird.hp = 200;
    bird.speed = 200;
    let mut game = Game::new(vec![bird], vec![mon("Foe", &["dig", "wait"])]);
    game.turn(&["burn_up", "wait"]);
    assert_eq!(game.ally(0).types, [PokemonType::Flying]);
    assert!(said(&game.turn(&["burn_up", "wait"]), "But it failed!"));
    // A Flying type is out of reach of Dig, unless it is roosting when the hit comes.
    game.turn(&["wait", "dig"]);
    let before = game.ally(0).hp;
    assert!(said(&game.turn(&["wait"]), "It doesn't affect Ally..."));
    assert_eq!(game.ally(0).hp, before);
    game.turn(&["wait", "dig"]);
    // Roost goes first and heals to full; the hit that follows gets through.
    let lines = game.turn(&["roost"]);
    assert!(!said(&lines, "It doesn't affect Ally..."));
    assert!(game.ally(0).hp < 300);
    assert!(
        game.ally(0).condition("roosting").is_none(),
        "over with the turn"
    );
}

#[test]
fn leech_seed_drains_every_turn_but_not_from_grass_types() {
    let mut game = Game::duel(&["leech_seed", "wait"], &["wait"]);
    game.turn(&["leech_seed", "wait"]);
    assert_eq!(game.foe(0).hp, 300 - 37);
    game.turn(&["wait", "wait"]);
    assert_eq!(game.foe(0).hp, 300 - 74);
    assert!(said(&game.turn(&["leech_seed", "wait"]), "But it failed!"));

    let mut grass = mon("Foe", &["wait"]);
    grass.types = vec![PokemonType::Grass];
    let mut game = Game::new(vec![mon("Ally", &["leech_seed"])], vec![grass]);
    assert!(said(&game.turn(&["leech_seed", "wait"]), "But it failed!"));
    assert_eq!(game.foe(0).hp, 300);
}

#[test]
fn moves_that_react_to_being_hit() {
    let mut game = Game::duel(&["shell_trap"], &["scratch", "ember", "wait"]);
    let sprung = game.turn(&["shell_trap", "scratch"]);
    assert_eq!(sprung[0], "Ally set a shell trap!");
    assert!(game.dealt() > 0);
    let hp = game.foe(0).hp;
    assert!(said(
        &game.turn(&["shell_trap", "ember"]),
        "shell trap didn't work!"
    ));
    assert_eq!(game.foe(0).hp, hp);

    let mut game = Game::duel(&["beak_blast"], &["scratch", "ember"]);
    game.turn(&["beak_blast", "ember"]);
    assert_eq!(game.foe(0).status, None);
    game.turn(&["beak_blast", "scratch"]);
    assert_eq!(game.foe(0).status, Some(Status::Burned));
}

#[test]
fn items_and_money() {
    let mut foe = mon("Foe", &["wait"]);
    foe.item = Some("sitrus_berry".into());
    let mut game = Game::new(
        vec![mon("Ally", &["belch", "bug_bite", "pay_day", "acrobatics"])],
        vec![foe],
    );
    assert!(said(&game.turn(&["belch", "wait"]), "But it failed!"));
    assert!(said(
        &game.turn(&["bug_bite", "wait"]),
        "stole and ate its target's sitrus_berry!"
    ));
    assert_eq!(game.foe(0).item, None);
    let before = game.foe(0).hp;
    game.turn(&["belch", "wait"]);
    assert!(game.foe(0).hp < before);
    game.turn(&["pay_day", "wait"]);
    game.turn(&["pay_day", "wait"]);
    assert_eq!(
        game.battle.payout(Side::Allies),
        u32::from(game.ally(0).level) * 10
    );
}

#[test]
fn a_move_can_help_an_ally_or_hurt_a_foe() {
    let mut partner = mon("Partner", &["wait"]);
    partner.hp = 100;
    let mut game = Game::new(
        vec![mon("Ally", &["pollen_puff"]), partner],
        vec![mon("Foe", &["wait"])],
    );
    game.turn(&["pollen_puff>allies", "wait", "wait"]);
    assert_eq!((game.ally(1).hp, game.dealt()), (250, 0));
    game.turn(&["pollen_puff>foes", "wait", "wait"]);
    assert!(game.dealt() > 0);
}
