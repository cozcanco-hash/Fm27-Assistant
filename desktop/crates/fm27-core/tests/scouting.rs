use fm27_core::{
    demo_snapshot, parse_snapshot,
    scouting::{AttributeFilter, PlayerQuery, PlayerSort},
    Snapshot, Source, Store,
};

fn manual(career: &str, date: &str) -> Snapshot {
    let mut s = demo_snapshot();
    s.source = Source::Manual;
    s.career_id = Some(career.into());
    s.game_date = Some(date.into());
    s
}

#[test]
fn old_schema_remains_valid_and_unknown_values_are_not_invented() {
    let mut json = serde_json::to_value(demo_snapshot()).unwrap();
    json.as_object_mut().unwrap().remove("careerId");
    json.as_object_mut().unwrap().remove("gameDate");
    for p in json["players"].as_array_mut().unwrap() {
        for key in ["currentAbility", "potentialAbility", "hiddenAttributes"] {
            p.as_object_mut().unwrap().remove(key);
        }
    }
    let s = parse_snapshot(&serde_json::to_vec(&json).unwrap()).unwrap();
    assert!(s.players.iter().all(|p| p.current_ability.is_none()
        && p.potential_ability.is_none()
        && p.hidden_attributes.is_empty()));
    assert!(s.career_id.is_none());
}

#[test]
fn invalid_hidden_values_or_ability_do_not_replace_saved_data() {
    let mut store = Store::memory().unwrap();
    let before = serde_json::to_value(store.snapshot()).unwrap();
    let mut bad = demo_snapshot();
    bad.players[0].current_ability = Some(201);
    assert!(store.replace(bad.clone()).is_err());
    bad.players[0].current_ability = None;
    bad.players[0]
        .hidden_attributes
        .insert("professionalism".into(), 0);
    assert!(store.replace(bad.clone()).is_err());
    bad.players[0].hidden_attributes.clear();
    bad.players[0]
        .hidden_attributes
        .insert("madeUpAttribute".into(), 10);
    assert!(store.replace(bad).is_err());
    assert_eq!(serde_json::to_value(store.snapshot()).unwrap(), before);
}

#[test]
fn combined_filters_exclude_missing_values_and_invalid_ranges_are_rejected() {
    let mut store = Store::memory().unwrap();
    let mut s = demo_snapshot();
    s.players.truncate(3);
    for p in &mut s.players {
        p.age = 18;
        p.current_ability = Some(140);
        p.potential_ability = Some(180);
        p.hidden_attributes.insert("professionalism".into(), 16);
        p.attributes.insert("passing".into(), 15);
    }
    s.players[1].hidden_attributes.clear();
    s.players[2].potential_ability = None;
    store.replace(s).unwrap();
    let page = store
        .players(PlayerQuery {
            max_age: Some(21),
            min_ca: Some(130),
            min_pa: Some(170),
            attributes: vec![
                AttributeFilter {
                    key: "professionalism".into(),
                    min: Some(15),
                    hidden: true,
                    ..Default::default()
                },
                AttributeFilter {
                    key: "passing".into(),
                    min: Some(14),
                    ..Default::default()
                },
            ],
            ..Default::default()
        })
        .unwrap();
    assert_eq!(page.players.iter().map(|p| p.id).collect::<Vec<_>>(), [1]);
    assert!(store
        .players(PlayerQuery {
            min_age: Some(30),
            max_age: Some(20),
            ..Default::default()
        })
        .is_err());
}

#[test]
fn role_filter_reuses_existing_scores_and_respects_positions() {
    let store = Store::memory().unwrap();
    let page = store
        .players(PlayerQuery {
            role_id: Some("keeper".into()),
            min_role_score: Some(1.0),
            ..Default::default()
        })
        .unwrap();
    assert!(!page.players.is_empty());
    for row in page.players {
        let p = store
            .snapshot()
            .players
            .iter()
            .find(|p| p.id == row.id)
            .unwrap();
        assert!(p.positions.contains(&fm27_core::model::Position::GK));
        assert_eq!(
            row.role.unwrap().score,
            fm27_core::analysis::fits(p)
                .into_iter()
                .find(|r| r.role_id == "keeper")
                .unwrap()
                .score
        );
    }
}

#[test]
fn unknowns_sort_last_in_both_directions_and_turkish_i_search_works() {
    let mut store = Store::memory().unwrap();
    let mut s = demo_snapshot();
    s.players.truncate(3);
    s.players[0].current_ability = Some(110);
    s.players[1].current_ability = None;
    s.players[2].current_ability = Some(160);
    s.players[0].name = "ILGAZ IŞIK".into();
    store.replace(s).unwrap();
    assert_eq!(
        store
            .players(PlayerQuery {
                search: Some("ılgaz".into()),
                ..Default::default()
            })
            .unwrap()
            .total,
        1
    );
    for descending in [false, true] {
        let page = store
            .players(PlayerQuery {
                sort: PlayerSort::Ability,
                descending,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page.players.last().unwrap().id, 2);
        assert_eq!(page.players[0].id, if descending { 3 } else { 1 });
    }
}

#[test]
fn named_lists_survive_restart_and_removal_is_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("assistant.sqlite");
    let list_id;
    {
        let mut store = Store::open(&path).unwrap();
        list_id = store
            .create_shortlist("Genç adaylar")
            .unwrap()
            .last()
            .unwrap()
            .id;
        store.set_shortlisted(list_id, 1, true).unwrap();
        store.set_shortlisted(list_id, 1, true).unwrap();
    }
    let mut store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .players(PlayerQuery {
                list_id: Some(list_id),
                ..Default::default()
            })
            .unwrap()
            .total,
        1
    );
    assert!(store.delete_empty_shortlist(list_id).is_err());
    store.set_shortlisted(list_id, 1, false).unwrap();
    store.set_shortlisted(list_id, 1, false).unwrap();
    store.delete_empty_shortlist(list_id).unwrap();
    assert_eq!(store.shortlists().unwrap().len(), 1);
}

#[test]
fn same_player_ids_in_different_careers_cannot_share_lists_or_history() {
    let mut store = Store::memory().unwrap();
    store.replace(manual("first", "2026-08-01")).unwrap();
    let id = store.shortlists().unwrap()[0].id;
    store.set_shortlisted(id, 1, true).unwrap();
    store.replace(manual("second", "2026-09-01")).unwrap();
    assert!(store.shortlists().unwrap()[0].entries.is_empty());
    assert!(store.set_shortlisted(id, 1, true).is_err());
    assert_eq!(store.player_history(1).unwrap().points.len(), 1);
    store.replace(manual("first", "2026-10-01")).unwrap();
    assert_eq!(store.shortlists().unwrap()[0].entries.len(), 1);
    assert_eq!(store.player_history(1).unwrap().points.len(), 2);
}

#[test]
fn unidentified_imports_do_not_merge_history_or_lists() {
    let mut store = Store::memory().unwrap();
    let mut s = manual("first", "2026-08-01");
    s.career_id = None;
    store.replace(s.clone()).unwrap();
    let id = store.shortlists().unwrap()[0].id;
    store.set_shortlisted(id, 1, true).unwrap();
    assert!(store.player_history(1).unwrap().points.is_empty());
    store.replace(s).unwrap();
    assert_ne!(store.shortlists().unwrap()[0].id, id);
    assert!(store.shortlists().unwrap()[0].entries.is_empty());
}

#[test]
fn departed_players_stay_listed_and_reused_ids_do_not_inherit_history() {
    let mut store = Store::memory().unwrap();
    let mut s = manual("career", "2026-08-01");
    store.replace(s.clone()).unwrap();
    let id = store.shortlists().unwrap()[0].id;
    store.set_shortlisted(id, 1, true).unwrap();
    s.game_date = Some("2026-09-01".into());
    s.players[0].name = "Başka Oyuncu".into();
    store.replace(s).unwrap();
    assert!(!store.shortlists().unwrap()[0].entries[0].available);
    assert_eq!(
        store
            .players(PlayerQuery {
                list_id: Some(id),
                ..Default::default()
            })
            .unwrap()
            .total,
        0
    );
    assert_eq!(store.player_history(1).unwrap().points.len(), 1);
    store.set_shortlisted(id, 1, false).unwrap();
}

#[test]
fn history_uses_game_dates_deduplicates_days_and_hides_future_after_rollback() {
    let mut store = Store::memory().unwrap();
    let mut s = manual("career", "2026-08-01");
    store.replace(s.clone()).unwrap();
    s.game_date = Some("2026-09-01".into());
    s.players[0].current_ability = Some(150);
    store.replace(s.clone()).unwrap();
    s.players[0].current_ability = Some(151);
    store.replace(s.clone()).unwrap();
    let points = store.player_history(1).unwrap().points;
    assert_eq!(points.len(), 2);
    assert_eq!(points[1].current_ability, Some(151));
    s.game_date = Some("2026-08-01".into());
    store.replace(s).unwrap();
    assert_eq!(store.player_history(1).unwrap().points.len(), 1);
}

#[test]
fn development_outlives_snapshot_retention_and_is_bounded_to_24_dates() {
    let mut store = Store::memory().unwrap();
    let mut s = manual("career", "2026-08-01");
    s.players.truncate(1);
    for day in 1..=30 {
        s.game_date = Some(format!("2026-08-{day:02}"));
        store.replace(s.clone()).unwrap();
    }
    let points = store.player_history(1).unwrap().points;
    assert_eq!(points.len(), 24);
    assert_eq!(points[0].game_date, "2026-08-07");
}

#[test]
fn failed_history_write_rolls_back_snapshot_before_publishing_state() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("assistant.sqlite");
    let mut store = Store::open(&path).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER deny_history BEFORE INSERT ON player_history BEGIN SELECT RAISE(FAIL, 'write failed'); END;").unwrap();
    assert!(store.replace(manual("new-career", "2026-09-01")).is_err());
    assert_eq!(store.snapshot().source, Source::Demo);
    drop(store);
    assert_eq!(Store::open(&path).unwrap().snapshot().source, Source::Demo);
}
