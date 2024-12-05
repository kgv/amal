pub(crate) use self::{difference::Pane as DifferencePane, source::Pane as SourcePane};

use egui::Ui;
use egui_phosphor::regular::{CHART_BAR, TABLE};
use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};

/// Pane
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum Pane {
    Source(SourcePane),
    Difference(DifferencePane),
}

impl Pane {
    pub(crate) fn source(data_frame: DataFrame) -> Self {
        Self::Source(SourcePane::new(data_frame))
    }

    pub(crate) fn difference(data_frame: DataFrame) -> Self {
        Self::Difference(DifferencePane::new(data_frame))
    }

    pub(crate) const fn icon(&self) -> &str {
        match self {
            Self::Source(_) => CHART_BAR,
            Self::Difference(_) => TABLE,
        }
    }

    pub(crate) const fn title(&self) -> &'static str {
        match self {
            Self::Source(_) => "Source",
            Self::Difference(_) => "Difference",
        }
    }

    pub(crate) const fn data_frame(&self) -> &DataFrame {
        match self {
            Self::Source(pane) => &pane.target,
            Self::Difference(pane) => &pane.target,
        }
    }
}

impl Pane {
    fn header(&mut self, ui: &mut Ui) {
        match self {
            Self::Source(pane) => pane.header(ui),
            Self::Difference(pane) => pane.header(ui),
        }
    }

    fn content(&mut self, ui: &mut Ui) {
        match self {
            Self::Source(pane) => pane.content(ui),
            Self::Difference(pane) => pane.content(ui),
        }
    }
}

pub(crate) mod behavior;
pub(crate) mod difference;
pub(crate) mod source;
pub(crate) mod widgets;
