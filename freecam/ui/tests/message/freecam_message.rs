use std::path::PathBuf;

use freecam_ui::message::FreecamMessage;
use levi_i18n::message::Message;
use levi_i18n_tool::pot::PotWriter;

#[test]
fn every_message_has_a_dense_index() {
    for (position, message) in FreecamMessage::ALL.iter().enumerate() {
        assert_eq!(message.index(), position, "{message:?}");
    }
}

#[test]
fn pot_is_current() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n").join(format!("{}.pot", FreecamMessage::DOMAIN.name()));
    let stale = PotWriter::write::<FreecamMessage>(&path, true).unwrap_or_else(|error| panic!("{error}"));
    assert!(!stale, "{} is out of date: run `mise run i18n:extract`", path.display());
}
