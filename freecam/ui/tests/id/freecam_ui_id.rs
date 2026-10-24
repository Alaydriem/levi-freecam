use egui::Id;
use freecam_ui::id::FreecamUiId;
use levi_ui::id::LeviUiId;

/// An id enum from another crate, with a derived `Hash`.
#[derive(Debug, Clone, Copy, Hash)]
enum OtherId {
    First,
    Second,
}

#[test]
fn a_freecam_ui_id_never_equals_the_same_numbered_variant_of_another_enum() {
    assert_ne!(Id::unique(FreecamUiId::PlaybackBar), Id::unique(OtherId::First));
    assert_ne!(Id::unique(FreecamUiId::Rewind), Id::unique(OtherId::Second));
    assert_ne!(Id::unique(FreecamUiId::PlaybackBar), Id::unique(LeviUiId::Locale));
    assert_ne!(Id::unique(FreecamUiId::Rewind), Id::unique(LeviUiId::SegmentSlide));
}

#[test]
fn freecam_ui_id_variants_keep_their_own_ids() {
    assert_ne!(Id::unique(FreecamUiId::PlaybackBar), Id::unique(FreecamUiId::Rewind));
}
