use std::time::Duration;

/// Where the time of the last [`super::UiRuntime::run`] went. All zero while profiling is off.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UiProfile {
    /// The egui pass: input, views, shape lists.
    pub run: Duration,
    pub tessellate: Duration,
    /// Copying meshes and texture changes into the output buffers.
    pub copy: Duration,
}
