#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GuiCommand {
    #[default]
    None,
    Hide,
    Show,
}
