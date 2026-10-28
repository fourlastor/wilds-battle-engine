//! Every message a battle shows has its key in locale/battle.pot, so it can be translated.
use std::collections::BTreeSet;
use wilds_battle_engine::{
    ActionSelection, AdvanceStatus, Battle, BattleEvent, MoveCatalog, Pokemon, PokemonType,
};

fn pot_keys() -> BTreeSet<String> {
    let pot = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../locale/battle.pot"
    ))
    .expect("locale/battle.pot: run scripts/extract_messages.py");
    pot.lines()
        .filter_map(|line| {
            line.strip_prefix("msgid \"")?
                .strip_suffix('"')
                .map(str::to_owned)
        })
        .map(|key| {
            key.replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\\\", "\\")
        })
        .collect()
}

fn mon(name: &str, moves: Vec<String>, types: Vec<PokemonType>) -> Pokemon {
    let mut pokemon = Pokemon::new(name, moves);
    pokemon.types = types;
    pokemon
}

/// Plays a few turns of every built-in move, in singles and doubles, against foes that
/// answer with a spread of other moves, and returns the keys missing from the template.
fn missing_keys() -> BTreeSet<String> {
    let known = pot_keys();
    let catalog = MoveCatalog::builtin().unwrap();
    let foe_moves: Vec<String> = ["tackle", "toxic", "protect", "thunder-wave", "sandstorm"]
        .map(str::to_owned)
        .into();
    let mut missing = BTreeSet::new();
    let ids: Vec<String> = catalog.ids().map(str::to_owned).collect();
    for (index, move_id) in ids.iter().filter(|id| *id != "struggle").enumerate() {
        for doubles in [false, true] {
            let mut allies = vec![mon(
                "Ally",
                vec![move_id.clone(), "splash".into()],
                vec![PokemonType::Normal],
            )];
            let mut foes = vec![mon("Foe", foe_moves.clone(), vec![PokemonType::Grass])];
            if doubles {
                allies.push(mon("Partner", vec!["helping-hand".into()], vec![]));
                foes.push(mon("Other", vec!["growl".into()], vec![PokemonType::Steel]));
            }
            let mut battle = Battle::new(index as u64 + 1, [allies, foes], catalog.clone())
                .unwrap_or_else(|error| panic!("{move_id}: {error}"));
            for step in 0..24 {
                let Ok(result) = battle.advance() else { break };
                for event in &result.events {
                    if let BattleEvent::Message(message) = event
                        && !known.contains(message.key.as_ref())
                    {
                        missing.insert(format!("{move_id}: {}", message.key));
                    }
                }
                let AdvanceStatus::Awaiting(prompt) = result.status else {
                    break;
                };
                let choice = &prompt.choices[step % prompt.choices.len()];
                if battle
                    .set_response(ActionSelection {
                        prompt_id: prompt.id,
                        choice_id: choice.id(),
                    })
                    .is_err()
                {
                    break;
                }
            }
        }
    }
    missing
}

#[test]
fn every_message_key_is_in_the_translation_template() {
    let missing = missing_keys();
    assert!(
        missing.is_empty(),
        "keys missing from locale/battle.pot ({}):\n{}",
        missing.len(),
        missing.into_iter().collect::<Vec<_>>().join("\n")
    );
}
