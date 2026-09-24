mod engine;
mod model;
mod moves;
mod type_chart;

pub use engine::Battle;
pub use model::{
    ActionSelection, AdvanceResult, AdvanceStatus, AppliedStatus, BattleError, BattleEvent, Bound,
    Choice, CombatStat, LockedMove, ParticipantId, Pokemon, PokemonType, Prompt, Side, Stat,
    Status, Weather, WeatherKind,
};
pub use moves::{Accuracy, Category, Effect, MoveCatalog, MoveSpec, Target};

#[cfg(test)]
mod tests {
    use super::*;

    fn id(side: Side) -> ParticipantId {
        ParticipantId { side, index: 0 }
    }
    fn one_on_one(seed: u64, ally: &str, foe: &str) -> Battle {
        Battle::new(
            seed,
            [
                vec![Pokemon::new("Ally", vec![ally.into()])],
                vec![Pokemon::new("Foe", vec![foe.into()])],
            ],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap()
    }
    fn pick(battle: &mut Battle, move_id: &str) {
        let result = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = result.status else {
            panic!("expected prompt")
        };
        let choice = prompt
            .choices
            .iter()
            .find(|c| c.move_id == move_id)
            .unwrap();
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: choice.id,
            })
            .unwrap();
    }
    #[test]
    fn loads_all_csharp_move_definitions() {
        let catalog = MoveCatalog::builtin().unwrap();
        assert_eq!(catalog.len(), 29);
        let kiss = catalog.get("draining_kiss").unwrap();
        assert_eq!(kiss.category, Category::Physical);
        assert_eq!(catalog.get("protect").unwrap().priority, 4);
    }
    #[test]
    fn rejects_unknown_typed_names_in_lua() {
        let base = r#"return {{ id = 'test', name = 'Test', type = 'normal', category = 'status', pp = 1, effects = {{ kind = 'status', status = 'poisoned' }} }}"#;
        for (source, expected) in [
            (
                base.replace("type = 'normal'", "type = 'typo'"),
                "unknown Pokemon type typo",
            ),
            (
                base.replace("status = 'poisoned'", "status = 'typo'"),
                "unknown status typo",
            ),
            (
                base.replace(
                    "kind = 'status', status = 'poisoned'",
                    "kind = 'weather', weather = 'typo', turns = 5",
                ),
                "unknown weather typo",
            ),
            (
                base.replace(
                    "kind = 'status', status = 'poisoned'",
                    "kind = 'stats', stages = { typo = 1 }",
                ),
                "unknown stat typo",
            ),
        ] {
            let error = MoveCatalog::from_lua(&source).unwrap_err();
            assert!(matches!(error, BattleError::InvalidSetup(_)));
            assert_eq!(error.to_string(), expected);
        }
    }
    #[test]
    fn prompts_both_sides_and_rejects_stale_responses() {
        let mut battle = one_on_one(10, "tackle", "tackle");
        let first = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("expected prompt")
        };
        assert_eq!(prompt.actor, id(Side::Allies));
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        assert!(
            battle
                .set_response(ActionSelection {
                    prompt_id: prompt.id,
                    choice_id: 0
                })
                .is_err()
        );
        let second = battle.advance().unwrap();
        let AdvanceStatus::Awaiting(prompt) = second.status else {
            panic!("expected prompt")
        };
        assert_eq!(prompt.actor, id(Side::Foes));
    }
    #[test]
    fn damage_and_recoil_resolve_in_order() {
        let mut battle = one_on_one(3, "double_edge", "splash");
        pick(&mut battle, "double_edge");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(
            result.events.iter().any(
                |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Foes))
            )
        );
        assert!(result.events.iter().any(
            |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Allies))
        ));
    }

    #[test]
    fn every_builtin_move_executes_a_turn() {
        let ids: Vec<String> = MoveCatalog::builtin()
            .unwrap()
            .ids()
            .map(str::to_owned)
            .collect();
        for move_id in ids {
            let mut battle = one_on_one(31, &move_id, "splash");
            pick(&mut battle, &move_id);
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            assert!(!result.events.is_empty(), "{move_id}");
        }
    }

    #[test]
    fn protect_runs_before_a_faster_attack() {
        let mut battle = one_on_one(1, "protect", "tackle");
        pick(&mut battle, "protect");
        pick(&mut battle, "tackle");
        let result = battle.advance().unwrap();
        assert!(!result.events.iter().any(
            |e| matches!(e, BattleEvent::Damage { target, .. } if *target == id(Side::Allies))
        ));
    }

    #[test]
    fn fixed_damage_and_multi_hit_keep_their_csharp_values() {
        let mut fixed = one_on_one(9, "dragon_rage", "splash");
        pick(&mut fixed, "dragon_rage");
        pick(&mut fixed, "splash");
        let result = fixed.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 40, .. } if *target == id(Side::Foes))));

        let mut multi = one_on_one(9, "double_kick", "splash");
        pick(&mut multi, "double_kick");
        pick(&mut multi, "splash");
        let result = multi.advance().unwrap();
        assert_eq!(
            result
                .events
                .iter()
                .filter(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes)))
                .count(),
            2
        );
    }

    #[test]
    fn toxic_damages_at_turn_end_and_rest_replaces_status() {
        let mut battle = one_on_one(7, "toxic", "splash");
        pick(&mut battle, "toxic");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert_eq!(
            battle.participants(Side::Foes)[0].status,
            Some(Status::BadlyPoisoned)
        );
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));

        let mut ally = Pokemon::new("Ally", vec!["rest".into()]);
        ally.hp = 35;
        ally.status = Some(Status::Poisoned);
        let mut rest = Battle::new(
            9,
            [vec![ally], vec![Pokemon::new("Foe", vec!["splash".into()])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut rest, "rest");
        pick(&mut rest, "splash");
        rest.advance().unwrap();
        let ally = &rest.participants(Side::Allies)[0];
        assert_eq!(ally.status, Some(Status::Asleep));
        assert_eq!(ally.hp, ally.max_hp);
        assert_eq!(ally.status_turns, 2);
    }

    #[test]
    fn solar_beam_charges_then_acts_without_a_second_choice() {
        let mut battle = one_on_one(5, "solar_beam", "splash");
        pick(&mut battle, "solar_beam");
        pick(&mut battle, "splash");
        let first = battle.advance().unwrap();
        assert!(
            first
                .events
                .iter()
                .any(|event| matches!(event, BattleEvent::Message(message)
            if message.contains("took in sunlight")))
        );
        assert!(battle.participants(Side::Allies)[0].locked_move.is_some());
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("foe should choose")
        };
        assert_eq!(prompt.actor, id(Side::Foes));
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        let second = battle.advance().unwrap();
        assert!(second.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));
    }

    #[test]
    fn sandstorm_damages_unprotected_types_at_turn_end() {
        let mut battle = one_on_one(4, "sandstorm", "splash");
        pick(&mut battle, "sandstorm");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert_eq!(battle.weather().unwrap().kind, WeatherKind::Sandstorm);
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 6, .. } if *target == id(Side::Allies))));
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 6, .. } if *target == id(Side::Foes))));
    }

    #[test]
    fn self_stat_effect_survives_a_target_ko() {
        let mut found = false;
        for seed in 1..2000 {
            let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
            foe.hp = 1;
            let mut battle = Battle::new(
                seed,
                [
                    vec![Pokemon::new("Ally", vec!["ancient_power".into()])],
                    vec![foe],
                ],
                MoveCatalog::builtin().unwrap(),
            )
            .unwrap();
            pick(&mut battle, "ancient_power");
            pick(&mut battle, "splash");
            let result = battle.advance().unwrap();
            if result.events.iter().any(
                |event| matches!(event, BattleEvent::StatChange { target, .. } if *target == id(Side::Allies)),
            ) {
                assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Attack), 1);
                assert_eq!(battle.participants(Side::Allies)[0].stage(Stat::Speed), 1);
                assert_eq!(battle.participants(Side::Foes)[0].hp, 0);
                found = true;
                break;
            }
        }
        assert!(found, "expected at least one seeded secondary effect");
    }

    #[test]
    fn ohko_respects_level_and_full_health_recovery_fails() {
        let mut ally = Pokemon::new("Ally", vec!["fissure".into()]);
        ally.level = 10;
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.level = 50;
        let mut battle =
            Battle::new(1, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "fissure");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(!result.events.iter().any(
            |event| matches!(event, BattleEvent::Damage { target, .. } if *target == id(Side::Foes))
        ));

        let mut battle = one_on_one(1, "recover", "splash");
        pick(&mut battle, "recover");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(
            |event| matches!(event, BattleEvent::Message(message) if message == "But it failed!")
        ));
    }

    #[test]
    fn dive_avoids_an_attack_while_charging_and_hits_next_turn() {
        let mut user = Pokemon::new("Ally", vec!["dive".into()]);
        user.speed = 200;
        let mut battle = Battle::new(
            13,
            [vec![user], vec![Pokemon::new("Foe", vec!["tackle".into()])]],
            MoveCatalog::builtin().unwrap(),
        )
        .unwrap();
        pick(&mut battle, "dive");
        pick(&mut battle, "tackle");
        let first = battle.advance().unwrap();
        assert!(battle.participants(Side::Allies)[0].semi_invulnerable);
        assert!(!first.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Allies))));
        let AdvanceStatus::Awaiting(prompt) = first.status else {
            panic!("foe should choose")
        };
        battle
            .set_response(ActionSelection {
                prompt_id: prompt.id,
                choice_id: 0,
            })
            .unwrap();
        let second = battle.advance().unwrap();
        assert!(second.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));
        assert!(!battle.participants(Side::Allies)[0].semi_invulnerable);
    }

    #[test]
    fn sunny_day_skips_solar_beam_charge() {
        let ally = Pokemon::new("Ally", vec!["sunny_day".into(), "solar_beam".into()]);
        let foe = Pokemon::new("Foe", vec!["splash".into()]);
        let mut battle =
            Battle::new(9, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "sunny_day");
        pick(&mut battle, "splash");
        battle.advance().unwrap();
        pick(&mut battle, "solar_beam");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, .. } if *target == id(Side::Foes))));
        assert!(battle.participants(Side::Allies)[0].locked_move.is_none());
    }

    #[test]
    fn bind_applies_residual_damage() {
        let mut battle = one_on_one(8, "bind", "splash");
        pick(&mut battle, "bind");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(battle.participants(Side::Foes)[0].bound.is_some());
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Damage { target, amount: 12, .. } if *target == id(Side::Foes))));
    }

    #[test]
    fn drain_heals_and_storm_throw_always_crits() {
        let mut ally = Pokemon::new("Ally", vec!["draining_kiss".into()]);
        ally.hp = 50;
        let foe = Pokemon::new("Foe", vec!["splash".into()]);
        let mut battle =
            Battle::new(3, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "draining_kiss");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Heal { target, .. } if *target == id(Side::Allies))));

        let mut battle = one_on_one(3, "storm_throw", "splash");
        pick(&mut battle, "storm_throw");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert!(result.events.iter().any(|event| matches!(event,
            BattleEvent::Message(message) if message == "A critical hit!")));
    }

    #[test]
    fn recoil_can_faint_both_and_uses_current_csharp_tie_outcome() {
        let mut ally = Pokemon::new("Ally", vec!["double_edge".into()]);
        ally.hp = 1;
        ally.speed = 200;
        let mut foe = Pokemon::new("Foe", vec!["splash".into()]);
        foe.hp = 1;
        let mut battle =
            Battle::new(2, [vec![ally], vec![foe]], MoveCatalog::builtin().unwrap()).unwrap();
        pick(&mut battle, "double_edge");
        pick(&mut battle, "splash");
        let result = battle.advance().unwrap();
        assert_eq!(
            result.status,
            AdvanceStatus::End {
                winner: Some(Side::Allies)
            }
        );
        assert_eq!(battle.participants(Side::Allies)[0].hp, 0);
        assert_eq!(battle.participants(Side::Foes)[0].hp, 0);
    }
}
