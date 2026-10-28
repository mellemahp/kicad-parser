use std::path::PathBuf;

use kicad_data_model::model::KiCadSchema;
use rstest::*;

/// Convenience to get the current file under test.
#[fixture]
fn root_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("examples");
    path
}

#[rstest]
#[case("buildacubesat_board.kicad_sch")]
fn test_valid_roundtrip(mut root_path: PathBuf, #[case] filename: &str) {
    root_path.push(filename);
    let loaded = KiCadSchema::load(root_path).unwrap();
    let mut buffer = Vec::new();
    loaded.write(&mut buffer).unwrap();
}

// TODO: Invalid parsing tests
