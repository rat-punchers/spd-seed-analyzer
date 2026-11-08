use super::*;

use std::ffi::OsStr;

const WELL: u16 = 24;

/// Replays `fixture_name`'s floor and compares the painted SecretWellRoom
/// (walls, chasm ring, path, well) cell for cell with the oracle.
fn assert_secret_well_terrain(fixture_name: &str) {
    let name = OsStr::new(fixture_name);
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .unwrap_or_else(|| panic!("missing {fixture_name}"));
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("oracle floor facts");
    let room = expected
        .room_bounds
        .iter()
        .find(|room| room.class_name == "SecretWellRoom")
        .expect("fixture has a SecretWellRoom");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=expected.depth {
        dungeon.depth = depth as i32;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let map = actual.expect("replay").map.expect("floor map");
    let terrain = expected.terrain.as_ref().expect("terrain");
    let width = expected.width as i32;
    let mut wells = 0;
    for y in room.top..=room.bottom {
        for x in room.left..=room.right {
            let cell = (y * width + x) as usize;
            assert_eq!(
                map.tiles[cell], terrain[cell],
                "{fixture_name} SecretWellRoom ({x}, {y})"
            );
            wells += usize::from(terrain[cell] == WELL);
        }
    }
    assert_eq!(wells, 1, "{fixture_name}: one well");
}

#[test]
fn secret_well_rooms_match_oracle_terrain() {
    let _journal = crate::fresh_journal();
    assert_secret_well_terrain("gfx-pzh-dch-final-heaps-floor-17.json");
    assert_secret_well_terrain("aaa-aaa-aaa-final-heaps-floor-23.json");
}
