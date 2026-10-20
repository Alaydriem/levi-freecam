#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartRefusal {
    NotInWorld,
    MenuOpen,
    NotReady,
    ServerCameraActive,
}
