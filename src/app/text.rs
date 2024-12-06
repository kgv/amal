use crate::app::panes::source;

/// Text
pub trait Text {
    fn text(&self) -> &'static str;

    fn description(&self) -> &'static str;
}
