use self::{control::Control, settings::Settings};
use crate::app::{
    computers::{SourceComputed, SourceKey},
    data::{save, Format},
    localize,
};
use egui::{RichText, Ui, Window};
use egui_phosphor::regular::{ARROWS_HORIZONTAL, FLOPPY_DISK, GEAR};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use table::TableView;
use tracing::error;

/// Source pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: DataFrame,
    pub(crate) target: DataFrame,
    pub(crate) control: Control,
}

impl Pane {
    pub(crate) const fn new(data_frame: DataFrame) -> Self {
        Self {
            source: data_frame,
            target: DataFrame::empty(),
            control: Control::new(),
        }
    }

    pub(super) fn header(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.visuals_mut().button_frame = false;
        ui.toggle_value(
            &mut self.control.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading());
        ui.separator();
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            if ui.button("Parquet").clicked() {
                if let Err(error) = save("df.parquet", Format::Parquet, self.target.clone()) {
                    error!(%error);
                }
            }
            if ui.button("BIN").clicked() {
                // println!("self.target: {}", self.target);
                // let lazy_frame = self.target.clone().lazy().select([
                //     col("Mode").struct_().field_by_names(["*"]),
                //     col("FA"),
                //     col("Time").struct_().field_by_name("Values").alias("Time"),
                // ]);
                if let Err(error) = save("df.bin", Format::Bin, self.target.clone()) {
                    error!(%error);
                }
            }
            if ui.button("RON").clicked() {
                if let Err(error) = save("df.ron", Format::Ron, self.target.clone()) {
                    error!(%error);
                }
            }
        });
    }

    pub(super) fn content(&mut self, ui: &mut Ui) {
        self.window(ui);
        self.target = ui.memory_mut(|memory| {
            memory.caches.cache::<SourceComputed>().get(SourceKey {
                data_frame: &self.source,
                settings: &self.control.settings,
            })
        });
        TableView::new(&self.target, &self.control.settings).ui(ui);
    }

    fn window(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Source settings"))
            .id(ui.next_auto_id())
            .open(&mut self.control.open)
            .show(ui.ctx(), |ui| {
                self.control.settings.ui(ui, &self.source);
            });
    }
}

pub(crate) mod settings;

mod control;
mod table;
