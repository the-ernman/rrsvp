#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Booting,
    Paused,
    Playing,
    Menu,
    CompanionSync,
    UsbTransfer,
    Standby,
    Sleeping,
}
