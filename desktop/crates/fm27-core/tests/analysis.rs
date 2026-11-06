use fm27_core::{
    analysis::{role_fit, ROLES},
    demo_snapshot, parse_snapshot, Formation, Store,
};

#[test]
fn perfect_attributes_score_one_hundred_and_missing_attributes_do_not_become_zero() {
    let mut player = demo_snapshot().players.remove(0);
    for value in player.attributes.values_mut() {
        *value = 20;
    }
    assert_eq!(role_fit(&player, &ROLES[0]).score, Some(100.0));
    player.attributes.clear();
    let result = role_fit(&player, &ROLES[0]);
    assert_eq!(result.score, None);
    assert_eq!(result.coverage, 0.0);
    assert!(!result.missing.is_empty());
}

#[test]
fn lineup_is_complete_unique_eligible_and_excludes_injured_players() {
    let snapshot = demo_snapshot();
    for formation in [
        Formation::FourThreeThree,
        Formation::FourTwoThreeOne,
        Formation::FourFourTwo,
    ] {
        let result = fm27_core::analyze(&snapshot, formation);
        assert_eq!(result.filled, 11);
        let mut ids = std::collections::HashSet::new();
        for slot in result.slots {
            let player = slot.player.unwrap();
            assert!(ids.insert(player.id));
            assert!(!player.injured);
            let role = ROLES.iter().find(|r| r.id == slot.role_id).unwrap();
            assert!(player.positions.contains(&role.position));
        }
    }
}

#[test]
fn scarce_specialist_is_not_lost_to_greedy_selection() {
    let mut snapshot = demo_snapshot();
    snapshot.players.retain(|p| [10, 11].contains(&p.id));
    // The only AMC must remain there in 4-2-3-1, even when also a better MC.
    snapshot.players[0].positions = vec![fm27_core::model::Position::MC];
    snapshot.players[1].positions = vec![
        fm27_core::model::Position::MC,
        fm27_core::model::Position::AMC,
    ];
    let result = fm27_core::analyze(&snapshot, Formation::FourTwoThreeOne);
    assert_eq!(
        result
            .slots
            .iter()
            .find(|s| s.key == "am")
            .unwrap()
            .player
            .as_ref()
            .unwrap()
            .id,
        11
    );
    assert_eq!(result.filled, 2);
    assert_eq!(result.score, None);
}

#[test]
fn unavailable_or_unknown_players_leave_visible_gaps() {
    let mut snapshot = demo_snapshot();
    for p in &mut snapshot.players {
        p.attributes.clear();
    }
    let result = fm27_core::analyze(&snapshot, Formation::FourThreeThree);
    assert_eq!(result.filled, 0);
    assert_eq!(result.score, None);
    assert!(result.insights.iter().any(|i| i.kind == "warning"));
}

#[test]
fn malformed_import_preserves_previous_snapshot() {
    let mut store = Store::memory().unwrap();
    let before = store.snapshot().clone();
    assert!(store.import(b"not JSON").is_err());
    let mut bad = before.clone();
    bad.players[0].condition = 101;
    assert!(store.import(&serde_json::to_vec(&bad).unwrap()).is_err());
    bad = before.clone();
    bad.players[1].id = bad.players[0].id;
    assert!(store.import(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(store.snapshot().club, before.club);
    assert_eq!(store.snapshot().players.len(), before.players.len());
}

#[test]
fn persisted_snapshot_and_formation_survive_reopen() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("assistant.sqlite");
    {
        let mut store = Store::open(&path).unwrap();
        let mut snapshot = demo_snapshot();
        snapshot.club = "Test Kulübü".into();
        snapshot.source = fm27_core::Source::Manual;
        store
            .import(&serde_json::to_vec(&snapshot).unwrap())
            .unwrap();
        store.set_formation(Formation::FourTwoThreeOne).unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(store.snapshot().club, "Test Kulübü");
    assert_eq!(store.snapshot().source, fm27_core::Source::Manual);
    assert_eq!(store.formation(), Formation::FourTwoThreeOne);
}

#[test]
fn oversized_import_and_unknown_attributes_are_rejected() {
    assert!(parse_snapshot(&vec![b' '; fm27_core::model::MAX_IMPORT_BYTES + 1]).is_err());
    let mut snapshot = demo_snapshot();
    snapshot.players[0]
        .attributes
        .insert("madeUpField".into(), 20);
    assert!(snapshot.validate().is_err());
    snapshot.players[0].attributes.remove("madeUpField");
    snapshot.players[0].attributes.insert("passing".into(), 0);
    assert!(snapshot.validate().is_err());
}

#[test]
fn pagination_is_bounded_and_search_and_position_are_combined() {
    let store = Store::memory().unwrap();
    let page = store.players(fm27_core::store::PlayerQuery {
        search: Some("deniz".into()),
        position: Some(fm27_core::model::Position::GK),
        ..Default::default()
    });
    assert_eq!(page.total, 1);
    assert_eq!(page.players[0].name, "Deniz Arslan");
    let page = store.players(fm27_core::store::PlayerQuery {
        offset: Some(5),
        limit: Some(3),
        ..Default::default()
    });
    assert_eq!(page.players.len(), 3);
    assert_eq!(page.total, 24);
}

#[test]
fn demo_provenance_and_unknown_form_are_preserved_in_ai_context() {
    let snapshot = demo_snapshot();
    let context = fm27_core::analysis::prompt_context(&snapshot, Formation::FourThreeThree);
    assert!(context.contains("Demo"));
    assert!(context.contains("Canlı bağlantı YOK"));
    assert!(context.contains("\"passing\":"));
    assert!(context.contains("\"pace\":"));
    assert!(context.len() < 64 * 1024);
    assert!(snapshot.players.iter().any(|p| p.form.is_none()));
    assert!(!context.contains("/Users/"));
}

#[test]
fn database_failure_does_not_publish_new_state() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("assistant.sqlite");
    let mut store = Store::open(&path).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER deny_insert BEFORE INSERT ON snapshots BEGIN SELECT RAISE(FAIL, 'write failed'); END;").unwrap();
    let mut snapshot = demo_snapshot();
    snapshot.club = "Must not publish".into();
    assert!(store.replace(snapshot).is_err());
    assert_eq!(store.snapshot().club, "KUTRA FC");
}

#[test]
fn oversized_persisted_snapshot_is_rejected_before_restore() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("assistant.sqlite");
    drop(Store::open(&path).unwrap());
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE snapshots SET body=zeroblob(?)",
            [fm27_core::model::MAX_IMPORT_BYTES as i64 + 1],
        )
        .unwrap();
    assert!(Store::open(&path).is_err());
}
