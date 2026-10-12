//! Gen 8 target choices and spread damage, with exact integer reference vectors.
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, BattleRng, Choice, HiddenKind,
    MoveCatalog, ParticipantId, Pokemon, PokemonType, Prompt, Side, Status,
};

struct Fixed(u8);
impl BattleRng for Fixed {
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fraction(&mut self) -> f32 {
        0.5
    }
    fn damage_roll(&mut self) -> f32 {
        f32::from(self.0) / 100.0
    }
}
fn id(side: Side, index: usize) -> ParticipantId {
    ParticipantId { side, index }
}
fn mon(name: &str, moves: &[&str]) -> Pokemon {
    let mut p = Pokemon::new(name, moves.iter().map(|s| s.to_string()).collect());
    p.hp = 1000;
    p.max_hp = 1000;
    p.level = 53;
    p.attack = 103;
    p.sp_attack = 103;
    p.defense = 97;
    p.sp_defense = 97;
    p.types = vec![PokemonType::Normal];
    p
}
fn catalog(target: &str, script: &str, extra: &str) -> MoveCatalog {
    MoveCatalog::from_lua(&format!(
        r#"return {{
        {{id='test', name='Test', type=Type.Ice, category=Category.Physical,
          pp=10, accuracy=Accuracy.Always, target=Target.{target},
          script=function(ctx) {script} end}},
        {{id='idle', name='Idle', type=Type.Normal, category=Category.Status,
          pp=40, target=Target.User, script=function(ctx) end}},
        {{id='kill', name='Kill', type=Type.Normal, category=Category.Physical,
          pp=10, accuracy=Accuracy.Always,
          script=function(ctx) ctx:direct_damage(ctx.target, 10000, {{typeless=true}}) end}},
        {extra}
    }}"#
    ))
    .unwrap()
}
fn battle(target: &str, script: &str) -> Battle {
    Battle::with_rng(
        Fixed(100),
        [
            vec![mon("User", &["test"]), mon("Partner", &["idle"])],
            vec![mon("Foe A", &["idle"]), mon("Foe B", &["idle"])],
        ],
        catalog(target, script, ""),
    )
    .unwrap()
}
fn turn(b: &mut Battle, mut select: impl FnMut(&Prompt) -> u32) -> Vec<BattleEvent> {
    let start = b.turn();
    let mut events = vec![];
    for _ in 0..20 {
        let result = b.advance().unwrap();
        events.extend(result.events);
        if b.turn() > start {
            return events;
        }
        let AdvanceStatus::Awaiting(p) = result.status else {
            return events;
        };
        b.set_response(ActionSelection {
            prompt_id: p.id,
            choice_id: select(&p),
        })
        .unwrap();
    }
    panic!("turn did not finish")
}
fn first(p: &Prompt) -> u32 {
    p.choices
        .iter()
        .find(|c| matches!(c, Choice::UseMove { .. }))
        .unwrap()
        .id()
}
fn aimed(p: &Prompt, who: ParticipantId) -> u32 {
    p.choices
        .iter()
        .find(|c| matches!(c, Choice::UseMove {target, ..} if *target == who))
        .unwrap()
        .id()
}
fn hits(events: &[BattleEvent]) -> Vec<(ParticipantId, u16)> {
    events
        .iter()
        .filter_map(|e| match e {
            BattleEvent::Damage { target, amount, .. } => Some((*target, *amount)),
            _ => None,
        })
        .collect()
}

#[test]
fn selected_moves_offer_foes_then_active_ally_and_can_hit_that_ally() {
    let mut b = battle("Selected", "ctx:damage(73)");
    let user = id(Side::Allies, 0);
    let partner = id(Side::Allies, 1);
    let events = turn(&mut b, |p| {
        if p.actor != user {
            return first(p);
        }
        let targets: Vec<_> = p
            .choices
            .iter()
            .filter_map(|c| match c {
                Choice::UseMove { target, .. } => Some(*target),
                _ => None,
            })
            .collect();
        assert_eq!(targets, [id(Side::Foes, 0), id(Side::Foes, 1), partner]);
        aimed(p, partner)
    });
    assert_eq!(hits(&events), [(partner, 37)]);
}

#[test]
fn selected_moves_exclude_bench_and_follow_an_allied_switch_slot() {
    let mut b = Battle::with_rng(
        Fixed(100),
        [
            vec![
                mon("User", &["test"]),
                mon("Partner", &["idle"]),
                mon("Reserve", &["idle"]),
            ],
            vec![mon("Foe", &["idle"])],
        ],
        catalog("Selected", "ctx:damage(73)", ""),
    )
    .unwrap();
    b.set_active([vec![0, 1], vec![0]]).unwrap();
    let events = turn(&mut b, |p| {
        if p.actor == id(Side::Allies, 0) {
            assert!(
                !p.choices
                    .iter()
                    .any(|c| matches!(c, Choice::UseMove {target, ..} if target.index == 2))
            );
            aimed(p, id(Side::Allies, 1))
        } else if p.actor == id(Side::Allies, 1) {
            p.choices
                .iter()
                .find(|c| matches!(c, Choice::Switch {to, ..} if to.index == 2))
                .unwrap()
                .id()
        } else {
            first(p)
        }
    });
    assert_eq!(hits(&events), [(id(Side::Allies, 2), 37)]);
}

#[test]
fn shared_targeting_rules_preserve_adjacency_on_wider_fields() {
    for kind in ["Selected", "AnyOther", "AllOthers", "AllOpponents"] {
        let mut b = Battle::with_rng(
            Fixed(100),
            [
                vec![
                    mon("User", &["test"]),
                    mon("Near Ally", &["idle"]),
                    mon("Far Ally", &["idle"]),
                ],
                vec![
                    mon("Far Foe", &["idle"]),
                    mon("Near Foe", &["idle"]),
                    mon("Across", &["idle"]),
                ],
            ],
            catalog(kind, "ctx:damage(73)", ""),
        )
        .unwrap();
        let events = turn(&mut b, |p| {
            if p.actor == id(Side::Allies, 0) && matches!(kind, "Selected" | "AnyOther") {
                let offered = |who| {
                    p.choices
                        .iter()
                        .any(|c| matches!(c, Choice::UseMove {target, ..} if *target == who))
                };
                assert_eq!(offered(id(Side::Allies, 2)), kind == "AnyOther");
                assert_eq!(offered(id(Side::Foes, 0)), kind == "AnyOther");
                assert!(offered(id(Side::Allies, 1)));
                assert!(offered(id(Side::Foes, 1)));
                return aimed(p, id(Side::Foes, 1));
            }
            first(p)
        });
        let expected = match kind {
            "AllOthers" => vec![
                (id(Side::Allies, 1), 28),
                (id(Side::Foes, 1), 28),
                (id(Side::Foes, 2), 28),
            ],
            "AllOpponents" => vec![(id(Side::Foes, 1), 28), (id(Side::Foes, 2), 28)],
            _ => vec![(id(Side::Foes, 1), 37)],
        };
        assert_eq!(hits(&events), expected, "{kind}");
    }
}

#[test]
fn ally_switch_cannot_make_an_ordinary_selected_move_hit_its_user() {
    let mut b = Battle::with_rng(
        Fixed(100),
        [
            vec![mon("User", &["ice-beam"]), mon("Partner", &["ally-switch"])],
            vec![mon("Foe A", &["recover"]), mon("Foe B", &["recover"])],
        ],
        MoveCatalog::builtin().unwrap(),
    )
    .unwrap();
    let events = turn(&mut b, |p| {
        if p.actor == id(Side::Allies, 0) {
            aimed(p, id(Side::Allies, 1))
        } else {
            first(p)
        }
    });
    assert_eq!(b.active_indices(Side::Allies), &[1, 0]);
    assert!(hits(&events).is_empty());
    assert_eq!(b.participants(Side::Allies)[0].move_pp["ice-beam"], 9);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, BattleEvent::Message(m) if m == "But there is no target!"))
    );
}

#[test]
fn fainted_ally_target_fails_and_spends_pp_but_fainted_foe_retargets() {
    for ally_target in [true, false] {
        let mut user = mon("User", &["test"]);
        user.speed = 100;
        let mut killer = mon("Killer", &["kill"]);
        killer.speed = 200;
        let chosen = if ally_target {
            id(Side::Allies, 1)
        } else {
            id(Side::Foes, 1)
        };
        let mut b = Battle::with_rng(
            Fixed(100),
            [
                vec![user, mon("Partner", &["idle"])],
                vec![killer, mon("Foe B", &["idle"])],
            ],
            catalog("Selected", "ctx:damage(73)", ""),
        )
        .unwrap();
        let events = turn(&mut b, |p| {
            if p.actor.index == 0 {
                aimed(p, chosen)
            } else {
                first(p)
            }
        });
        let expected = if ally_target {
            vec![(chosen, 1000)]
        } else {
            vec![(chosen, 1000), (id(Side::Foes, 0), 37)]
        };
        assert_eq!(hits(&events), expected);
        assert_eq!(b.participants(Side::Allies)[0].move_pp["test"], 9);
        if ally_target {
            assert!(
                events.iter().any(
                    |e| matches!(e, BattleEvent::Message(m) if m == "But there is no target!")
                )
            );
        }
    }
}

#[test]
fn spread_damage_matches_for_implicit_recipients_and_per_target_lua_loops() {
    for script in [
        "ctx:damage(73)",
        "for _, p in ipairs(ctx.targets) do ctx:damage(73, {target=p}) end",
    ] {
        let mut b = battle("AllOpponents", script);
        assert_eq!(
            hits(&turn(&mut b, first)),
            [(id(Side::Foes, 0), 28), (id(Side::Foes, 1), 28)]
        );
        assert_eq!(b.participants(Side::Allies)[1].hp, 1000);
        let mut b = battle("AllOthers", script);
        assert_eq!(
            hits(&turn(&mut b, first)),
            [
                (id(Side::Allies, 1), 28),
                (id(Side::Foes, 0), 28),
                (id(Side::Foes, 1), 28),
            ]
        );
    }
}

#[test]
fn only_living_active_recipients_count_toward_spread() {
    for target in ["AllOpponents", "AllOthers"] {
        for partner in [false, true] {
            for fainted in [false, true] {
                let mut absent = mon("Absent", &["idle"]);
                if fainted {
                    absent.hp = 0;
                }
                let mut b = Battle::with_rng(
                    Fixed(100),
                    [
                        vec![mon("User", &["test"]), mon("Partner", &["idle"])],
                        vec![mon("Foe", &["idle"]), absent],
                    ],
                    catalog(target, "ctx:damage(73)", ""),
                )
                .unwrap();
                b.set_active([
                    if partner { vec![0, 1] } else { vec![0] },
                    if fainted { vec![0, 1] } else { vec![0] },
                ])
                .unwrap();
                let events = turn(&mut b, first);
                let expected = if target == "AllOthers" && partner {
                    28
                } else {
                    37
                };
                assert!(hits(&events).contains(&(id(Side::Foes, 0), expected)));
                assert!(!hits(&events).iter().any(|(p, _)| *p == id(Side::Foes, 1)));
            }
        }
    }
}

#[test]
fn protected_immune_hidden_and_missed_recipients_still_count_for_spread() {
    for blocked in ["protected", "immune", "hidden", "missed"] {
        let mut other = mon("Blocked", &["idle"]);
        match blocked {
            "protected" => other.protected = true,
            "immune" => other.ability = Some("wonderguard".into()),
            "hidden" => {
                other.semi_invulnerable = true;
                other.hidden_kind = HiddenKind::Vanished;
            }
            _ => {}
        }
        let script = if blocked == "missed" {
            "for _, p in ipairs(ctx.targets) do ctx:damage(73, {target=p, accuracy=p.name == 'Blocked' and 0 or 1}) end"
        } else {
            "ctx:damage(73)"
        };
        let mut b = Battle::with_rng(
            Fixed(100),
            [
                vec![mon("User", &["test"]), mon("Partner", &["idle"])],
                vec![mon("Foe", &["idle"]), other],
            ],
            catalog("AllOpponents", script, ""),
        )
        .unwrap();
        assert_eq!(
            hits(&turn(&mut b, first)),
            [(id(Side::Foes, 0), 28)],
            "{blocked}"
        );
    }
}

#[test]
fn spread_penalty_survives_a_knockout_on_an_earlier_hit() {
    for script in [
        "ctx:damage(73)",
        "for _, p in ipairs(ctx.targets) do ctx:damage(73, {target=p}) end",
    ] {
        let mut doomed = mon("Doomed", &["idle"]);
        doomed.hp = 1;
        let mut b = Battle::with_rng(
            Fixed(100),
            [
                vec![mon("User", &["test"]), mon("Partner", &["idle"])],
                vec![doomed, mon("Foe", &["idle"])],
            ],
            catalog("AllOpponents", script, ""),
        )
        .unwrap();
        assert_eq!(
            hits(&turn(&mut b, first)),
            [(id(Side::Foes, 0), 1), (id(Side::Foes, 1), 28)]
        );
    }
}

#[test]
fn spread_rounds_before_weather_critical_random_stab_type_and_burn() {
    let mut user = mon("User", &["test"]);
    user.types = vec![PokemonType::Fire];
    user.status = Some(Status::Burned);
    let mut foe = mon("Foe", &["idle"]);
    foe.types = vec![PokemonType::Grass];
    foe.ability = Some("drought".into());
    let mut b = Battle::with_rng(
        Fixed(85),
        [vec![user], vec![foe.clone(), foe]],
        catalog(
            "AllOpponents",
            "ctx:damage(73, {type=Type.Fire, always_crit=true})",
            "",
        ),
    )
    .unwrap();
    // 37 -> spread 28 -> sun 42 -> crit 63 -> random 53 -> STAB 79 -> type 158 -> burn 79.
    assert_eq!(
        hits(&turn(&mut b, first)),
        [
            (id(Side::Foes, 0), 79),
            (id(Side::Foes, 1), 79),
            (id(Side::Allies, 0), 62), // End-of-turn burn damage.
        ]
    );
}

#[test]
fn catalog_spread_moves_reduce_damage_but_dragon_darts_does_not() {
    for (move_id, expected, hits_partner) in [
        ("surf", 34, true),
        ("earthquake", 37, true),
        ("rock-slide", 28, false),
        ("dragon-darts", 26, false),
    ] {
        let mut b = Battle::with_rng(
            Fixed(100),
            [
                vec![mon("User", &[move_id]), mon("Partner", &["recover"])],
                vec![mon("Foe A", &["recover"]), mon("Foe B", &["recover"])],
            ],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        let mut expected_hits = vec![];
        if hits_partner {
            expected_hits.push((id(Side::Allies, 1), expected));
        }
        expected_hits.extend([(id(Side::Foes, 0), expected), (id(Side::Foes, 1), expected)]);
        assert_eq!(hits(&turn(&mut b, first)), expected_hits, "{move_id}");
    }
}

#[test]
fn fixed_spread_damage_is_not_reduced() {
    let mut b = battle(
        "AllOpponents",
        "for _, p in ipairs(ctx.targets) do ctx:direct_damage(p, 73) end",
    );
    assert_eq!(
        hits(&turn(&mut b, first)),
        [(id(Side::Foes, 0), 73), (id(Side::Foes, 1), 73)]
    );
}

#[test]
fn called_spread_and_single_target_moves_keep_separate_damage_contexts() {
    let extra = r#"
        {id='spread', name='Spread', type=Type.Ice, category=Category.Physical,
         pp=10, accuracy=Accuracy.Always, target=Target.AllOpponents,
         script=function(ctx) ctx:damage(73) end},
        {id='single', name='Single', type=Type.Ice, category=Category.Physical,
         pp=10, accuracy=Accuracy.Always,
         script=function(ctx) ctx:damage(73) end},
    "#;
    for (target, expected) in [("Selected", 37), ("AllOpponents", 28)] {
        let mut b = Battle::with_rng(Fixed(100), [
            vec![mon("User", &["test"])],
            vec![mon("Foe A", &["idle"]), mon("Foe B", &["idle"])],
        ], catalog(target,
            "ctx:call_move('spread'); ctx:call_move('single', ctx.target); ctx:damage(73, {target=ctx.target})", extra)).unwrap();
        assert_eq!(
            hits(&turn(&mut b, first)),
            [
                (id(Side::Foes, 0), 28),
                (id(Side::Foes, 1), 28),
                (id(Side::Foes, 0), 37),
                (id(Side::Foes, 0), expected),
            ]
        );
    }
}
