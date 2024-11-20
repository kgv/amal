// use crate::app::{text::Text, widgets::FloatValue, MARGIN};
use super::Settings;
use egui::{vec2, Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui, Vec2};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

const MARGIN: Vec2 = vec2(4.0, 0.0);

/// Composition table
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(in crate::app) struct TablePane {
    pub(in crate::app) data_frame: DataFrame,
    pub(in crate::app) settings: Settings,
}

impl TablePane {
    pub(in crate::app) fn new(data_frame: DataFrame, settings: Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub(super) fn ui(&mut self, ui: &mut Ui) {
        let id_salt = Id::new("CompositionTable");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as _;
        let num_columns = self.data_frame.width();
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(true);
                // Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky_columns)
            .headers([HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        match (row, column) {
            (0, 0) => {
                ui.heading("FA");
            }
            (0, 1) => {
                ui.heading("Temperature");
            }
            (0, 2) => {
                ui.heading("DeltaTemperature");
            }
            (0, 3) => {
                ui.heading("Time");
            }
            _ => unreachable!(),
        }
    }

    fn body_cell_content_ui(&mut self, ui: &mut Ui, row: usize, col: usize) {
        match (row, col) {
            (row, 0) => {
                let series = self.data_frame["FA"]
                    .struct_()
                    .unwrap()
                    .field_by_name("Label")
                    .unwrap();
                let values = series.str().unwrap();
                let value = values.get(row).unwrap();
                ui.label(value);
            }
            (row, column) => {
                let value = self.data_frame[column].get(row).unwrap();
                ui.label(value.to_string());
                // ui.add(Cell {
                //     row,
                //     column: &self.data_frame[column],
                //     percent: settings.percent,
                //     precision: settings.precision,
                // });
            }
        }
    }
}

impl TableDelegate for TablePane {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.group_index)
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 1 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr)
            });
    }
}
