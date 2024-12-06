use super::Settings;
use crate::special::fa_column::ColumnExt as _;
use egui::{vec2, Frame, Id, Margin, TextStyle, TextWrapMode, Ui, Vec2};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use polars::prelude::*;

const INDEX: usize = 0;
const MODE: usize = 1;
const FROM: usize = 2;
const TO: usize = 3;
const TIME: usize = 4;
const ECN: usize = 5;
const MASS: usize = 6;

const MARGIN: Vec2 = vec2(4.0, 0.0);

/// Table view
#[derive(Clone, Debug)]
pub(crate) struct TableView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl<'a> TableView<'a> {
    pub(crate) const fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }
}

impl TableView<'_> {
    pub(super) fn ui(&mut self, ui: &mut Ui) {
        ui.visuals_mut().collapsing_header_frame = true;
        let id_salt = Id::new("Table");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as _;
        let num_columns = self.data_frame.width();
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
            .headers([HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        match (row, column) {
            (0, INDEX) => {
                ui.heading("Index");
            }
            (0, MODE) => {
                ui.heading("Mode");
            }
            (0, FROM) => {
                ui.heading("From");
            }
            (0, TO) => {
                ui.heading("To");
            }
            (0, TIME) => {
                ui.heading("Time");
            }
            (0, ECN) => {
                ui.heading("ECL");
            }
            (0, MASS) => {
                ui.heading("Mass");
            }
            _ => {} // _ => unreachable!(),
        }
    }

    fn body_cell_content_ui(&mut self, ui: &mut Ui, row: usize, col: usize) {
        match (row, col) {
            (row, INDEX) => {
                let indices = self.data_frame["Index"].u32().unwrap();
                let value = indices.get(row).unwrap();
                ui.label(value.to_string());
            }
            (row, MODE) => {
                let mode = self.data_frame["Mode"].struct_().unwrap();
                let onset_temperature = mode.field_by_name("OnsetTemperature").unwrap();
                let temperature_step = mode.field_by_name("TemperatureStep").unwrap();
                ui.label(format!(
                    "{}/{}",
                    onset_temperature.str_value(row).unwrap(),
                    temperature_step.str_value(row).unwrap()
                ));
            }
            // (row, FA) => {
            //     let fatty_acids = self.target["FA"].fa();
            //     let fatty_acid = fatty_acids.get(row).unwrap();
            //     ui.label(fatty_acid.to_string())
            //         .on_hover_text(fatty_acid.label());
            // }
            (row, FROM) => {
                let fatty_acids = self.data_frame["From"].fa();
                let fatty_acid = fatty_acids.get(row).unwrap();
                ui.label(fatty_acid.to_string())
                    .on_hover_text(fatty_acid.label());
            }
            (row, TO) => {
                let fatty_acids = self.data_frame["To"].fa();
                let fatty_acid = fatty_acids.get(row).unwrap();
                ui.label(fatty_acid.to_string())
                    .on_hover_text(fatty_acid.label());
            }
            // (row, MASS) => {
            //     let mass = self.target["Mass"].struct_().unwrap();
            //     let rcooch3 = mass.field_by_name("RCOOCH3").unwrap();
            //     ui.label(rcooch3.str_value(row).unwrap()).on_hover_ui(|ui| {
            //         Grid::new(ui.next_auto_id()).show(ui, |ui| {
            //             ui.label("RCOOCH3");
            //             ui.label(rcooch3.str_value(row).unwrap());
            //             ui.end_row();
            //             {
            //                 ui.label("RCOOH");
            //                 let rcooh = mass.field_by_name("RCOOH").unwrap();
            //                 ui.label(rcooh.str_value(row).unwrap());
            //             }
            //             ui.end_row();
            //             {
            //                 ui.label("RCOO");
            //                 let rcoo = mass.field_by_name("RCOO").unwrap();
            //                 ui.label(rcoo.str_value(row).unwrap());
            //             }
            //         });
            //     });
            // }
            // (row, TIME) => {
            //     let time = self.target["Time"].struct_().unwrap();
            //     let means = time.field_by_name("Mean").unwrap();
            //     ui.label(means.str_value(row).unwrap()).on_hover_ui(|ui| {
            //         Grid::new(ui.next_auto_id()).show(ui, |ui| {
            //             // Absolute
            //             ui.label("Absolute");
            //             ui.horizontal(|ui| {
            //                 ui.label(means.str_value(row).unwrap());
            //                 ui.label("±");
            //                 let standard_deviations =
            //                     time.field_by_name("StandardDeviation").unwrap();
            //                 ui.label(standard_deviations.str_value(row).unwrap());
            //             });
            //             ui.end_row();
            //             // Relative
            //             let relatives = time.field_by_name("Relative").unwrap();
            //             ui.label("Relative");
            //             ui.label(relatives.str_value(row).unwrap());
            //         });
            //     });
            // }
            (row, column) => {
                let value = self.data_frame[column].get(row).unwrap();
                ui.label(value.to_string());
            }
        }
    }
}

impl TableDelegate for TableView<'_> {
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
