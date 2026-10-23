/// The egui ids of the freecam views, used with `Id::unique` or `Id::with`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FreecamUiId {
    /// The playback bar's own area.
    PlaybackBar,
    /// The bar's buttons.
    Rewind,
    Play,
    Screenshot,
    ClearPoints,
    AddPoint,
    Replace,
    Delete,
    /// The scrub line, its point marks and the playhead's grow while scrubbing.
    ScrubTrack,
    Mark,
    MarkHover,
    MarkLit,
    ScrubActive,
    /// The play button's hover and press transitions.
    PlayHover,
    PlayPress,
}
