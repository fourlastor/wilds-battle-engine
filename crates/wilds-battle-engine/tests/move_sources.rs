//! Catalogs that resolve moves through a host source before the built-in moves.
use std::cell::RefCell;
use std::rc::Rc;
use wilds_battle_engine::{Battle, MoveCatalog, MoveFile, Pokemon};

fn file(text: &str) -> Option<MoveFile> {
    Some(MoveFile {
        origin: "mods/test".into(),
        text: text.into(),
    })
}

#[test]
fn every_builtin_move_loads_by_id() {
    let catalog = MoveCatalog::builtin().unwrap();
    assert_eq!(catalog.len(), 754);
    for id in catalog.ids() {
        assert_eq!(catalog.load(id).unwrap().expect(id).id, id);
    }
    assert!(catalog.load("no-such-move").unwrap().is_none());
}

#[test]
fn a_source_adds_and_replaces_moves_and_is_asked_once() {
    let asked = Rc::new(RefCell::new(Vec::new()));
    let log = asked.clone();
    let catalog = MoveCatalog::with_source(move |id| {
        log.borrow_mut().push(id.to_owned());
        match id {
            "tackle" => file(
                r#"return {id="tackle", name="Modded Tackle", type=Type.Normal,
                    category=Category.Physical, pp=5, effects={{kind=Effect.Damage, power=90}}}"#,
            ),
            "new-move" => file(
                r#"return {id="new-move", name="New Move", type=Type.Fire,
                    category=Category.Special, pp=15, effects={{kind=Effect.Damage, power=60}}}"#,
            ),
            _ => None,
        }
    })
    .unwrap();
    assert_eq!(catalog.get("tackle").unwrap().name, "Modded Tackle");
    assert_eq!(catalog.get("new-move").unwrap().pp, 15);
    assert_eq!(catalog.get("growl").unwrap().name, "Growl");
    catalog.get("tackle");
    assert_eq!(*asked.borrow(), ["tackle", "new-move", "growl"]);
    assert!(
        !catalog.ids().any(|id| id == "new-move"),
        "sources are not listed"
    );

    let battle = Battle::new(
        1,
        [
            vec![Pokemon::new("Ally", vec!["new-move".into()])],
            vec![Pokemon::new("Foe", vec!["tackle".into()])],
        ],
        catalog,
    )
    .unwrap();
    assert_eq!(
        battle.participants(wilds_battle_engine::Side::Foes)[0].move_pp["tackle"],
        5
    );
}

#[test]
fn broken_and_mismatched_sources_fail_when_the_battle_is_created() {
    let catalog = MoveCatalog::with_source(|id| match id {
        "broken" => file("return {id="),
        "renamed" => file(
            r#"return {id="other", name="Other", type=Type.Normal,
                category=Category.Physical, pp=5, effects={{kind=Effect.Damage, power=10}}}"#,
        ),
        _ => None,
    })
    .unwrap();
    for (id, expected) in [
        ("broken", "mods/test"),
        ("renamed", r#"declares move id "other", expected "renamed""#),
    ] {
        let Err(error) = Battle::new(
            1,
            [
                vec![Pokemon::new("Ally", vec![id.into()])],
                vec![Pokemon::new("Foe", vec!["tackle".into()])],
            ],
            catalog.clone(),
        ) else {
            panic!("{id} loaded");
        };
        let error = error.to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn ids_that_are_not_slugs_never_reach_the_source() {
    let catalog = MoveCatalog::with_source(|id| panic!("asked for {id}")).unwrap();
    for id in ["../tackle", "Tackle", "", "a/b", "tackle.lua"] {
        assert!(catalog.load(id).unwrap().is_none(), "{id}");
    }
}

fn load_error(text: impl Into<String>) -> String {
    let text = text.into();
    let catalog =
        MoveCatalog::with_source(move |id| (id == "modded").then(|| file(&text)).flatten())
            .unwrap();
    catalog.load("modded").unwrap_err().to_string()
}

#[test]
fn move_files_cannot_reach_the_system() {
    for global in [
        "os",
        "io",
        "require",
        "package",
        "debug",
        "dofile",
        "loadfile",
        "load",
        "collectgarbage",
        "math.random",
        "math.randomseed",
    ] {
        let error = load_error(format!(
            "assert({global} == nil, 'available'); error('sandboxed')"
        ));
        assert!(error.contains("sandboxed"), "{global}: {error}");
    }
    let rules = MoveCatalog::rules_from_lua("return {os.exit(1)}").unwrap_err();
    assert!(rules.to_string().contains("os"), "{rules}");
}

#[test]
fn a_move_file_that_never_finishes_loading_fails() {
    let error = load_error("while true do end");
    assert!(error.contains("loading exceeded"), "{error}");
    assert!(error.contains("mods/test"), "{error}");
}
