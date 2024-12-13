use std::ops::Range;

use super::Settings;
use crate::{
    app::panes::widgets::float::FloatValue, special::columns::fatty_acids::ColumnExt as _,
};
use egui::{vec2, Frame, Grid, Id, Margin, TextStyle, TextWrapMode, Ui, Vec2};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use polars::prelude::*;

const RETENTION_TIME: usize = 3;
const EQUIVALENT: usize = 5;

const INDEX_RANGE: Range<usize> = 0..1;
const MODE_RANGE: Range<usize> = INDEX_RANGE.end..INDEX_RANGE.end + 1;
const FA_RANGE: Range<usize> = MODE_RANGE.end..MODE_RANGE.end + 1;
const TIME_RANGE: Range<usize> = FA_RANGE.end..FA_RANGE.end + 2;
const TEMPERATURE_RANGE: Range<usize> = TIME_RANGE.end..TIME_RANGE.end + 1;
const EQUIVALENT_RANGE: Range<usize> = TEMPERATURE_RANGE.end..TEMPERATURE_RANGE.end + 3;
const MASS_RANGE: Range<usize> = EQUIVALENT_RANGE.end..EQUIVALENT_RANGE.end + 1;
const META_RANGE: Range<usize> = MASS_RANGE.end..MASS_RANGE.end + 3;

const INDEX: usize = 0;
const MODE: usize = 1;
const FA: usize = 2;
const ABSOLUTE_TIME: usize = 3;
const RELATIVE_TIME: usize = 4;
const TEMPERATURE: usize = 5;
const ECL: usize = 6;
const FCL: usize = 7;
const ECN: usize = 8;
const MASS: usize = 9;
const DELTA: usize = 10;
const TANGENT: usize = 11;
const ANGLE: usize = 12;
const LEN: usize = 13;

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
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
            .headers([
                HeaderRow {
                    height,
                    groups: vec![
                        INDEX_RANGE,
                        MODE_RANGE,
                        FA_RANGE,
                        TIME_RANGE,
                        TEMPERATURE_RANGE,
                        EQUIVALENT_RANGE,
                        MASS_RANGE,
                        META_RANGE,
                    ],
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, 0) => {
                ui.heading("Index");
            }
            (0, 1) => {
                ui.heading("Mode");
            }
            (0, 2) => {
                ui.heading("Fatty acid");
            }
            (0, 3) => {
                ui.heading("Retention time");
            }
            (0, 4) => {
                ui.heading("Temperature");
            }
            (0, 5) => {
                ui.heading("Equivalent");
            }
            (0, 6) => {
                ui.heading("Mass");
            }
            (0, 7) => {
                ui.heading("Meta");
            }
            // Bottom
            (1, 0..3) => {}
            (1, 3) => {
                ui.heading("Absolute");
            }
            (1, 4) => {
                ui.heading("Relative");
            }
            (1, 5) => {}
            (1, ECL) => {
                ui.heading("ECL");
            }
            (1, FCL) => {
                ui.heading("FCL");
            }
            (1, ECN) => {
                ui.heading("ECN");
            }
            (1, MASS) => {}
            (1, DELTA) => {
                ui.heading("Delta");
            }
            (1, TANGENT) => {
                ui.heading("Tan");
            }
            (1, ANGLE) => {
                ui.heading("Angle");
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
            (row, FA) => {
                let fatty_acids = self.data_frame["FA"].fa();
                let fatty_acid = fatty_acids.get(row).unwrap();
                ui.label(fatty_acid.to_string())
                    .on_hover_text(fatty_acid.label());
            }
            (row, ABSOLUTE_TIME) => {
                let time = self.data_frame["Time"].struct_().unwrap();
                let absolute = time.field_by_name("Absolute").unwrap();
                let absolute = absolute.struct_().unwrap();
                let mean = absolute.field_by_name("Mean").unwrap();
                ui.add(
                    FloatValue::new(mean.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    let standard_deviation = absolute.field_by_name("StandardDeviation").unwrap();
                    ui.horizontal(|ui| {
                        ui.label(mean.str_value(row).unwrap());
                        ui.label("±");
                        ui.label(standard_deviation.str_value(row).unwrap());
                    });
                })
                .on_hover_ui(|ui| {
                    ui.heading("Repetitions");
                    let values = absolute
                        .field_by_name("Values")
                        .unwrap()
                        .list()
                        .unwrap()
                        .get_as_series(row)
                        .unwrap();
                    ui.horizontal(|ui| {
                        for value in values.iter() {
                            ui.label(value.to_string());
                        }
                    });
                });
            }
            (row, RELATIVE_TIME) => {
                let time = self.data_frame["Time"].struct_().unwrap();
                let relative = time.field_by_name("Relative").unwrap();
                ui.add(
                    FloatValue::new(relative.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, TEMPERATURE) => {
                let temperature = &self.data_frame["Temperature"];
                ui.add(
                    FloatValue::new(temperature.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, ECL) => {
                let equivalent = self.data_frame["Equivalent"].struct_().unwrap();
                let ecl = equivalent.field_by_name("ECL").unwrap();
                ui.add(
                    FloatValue::new(ecl.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, FCL) => {
                let equivalent = self.data_frame["Equivalent"].struct_().unwrap();
                let fcl = equivalent.field_by_name("FCL").unwrap();
                ui.add(
                    FloatValue::new(fcl.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, ECN) => {
                let equivalent = self.data_frame["Equivalent"].struct_().unwrap();
                let ecn = equivalent.field_by_name("ECN").unwrap();
                ui.label(ecn.str_value(row).unwrap());
            }
            (row, MASS) => {
                let mass = self.data_frame["Mass"].struct_().unwrap();
                let rcooch3 = mass.field_by_name("RCOOCH3").unwrap();
                ui.add(
                    FloatValue::new(rcooch3.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision)),
                )
                .on_hover_ui(|ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        ui.label("RCOO");
                        let rcoo = mass.field_by_name("RCOO").unwrap();
                        ui.label(rcoo.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("RCOOH");
                        let rcooh = mass.field_by_name("RCOOH").unwrap();
                        ui.label(rcooh.str_value(row).unwrap());
                        ui.end_row();

                        ui.label("RCOOCH3");
                        ui.label(rcooch3.str_value(row).unwrap());
                    });
                });
            }
            (row, DELTA) => {
                let equivalent = self.data_frame["Meta"].struct_().unwrap();
                let dx = equivalent.field_by_name("Delta").unwrap();
                ui.add(
                    FloatValue::new(dx.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, TANGENT) => {
                let equivalent = self.data_frame["Meta"].struct_().unwrap();
                let slope = equivalent.field_by_name("Tangent").unwrap();
                ui.add(
                    FloatValue::new(slope.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, ANGLE) => {
                let equivalent = self.data_frame["Meta"].struct_().unwrap();
                let angle = equivalent.field_by_name("Angle").unwrap();
                ui.add(
                    FloatValue::new(angle.f64().unwrap().get(row))
                        .precision(Some(self.settings.precision))
                        .hover(),
                );
            }
            (row, column) => {
                let value = self.data_frame[column - 1].get(row).unwrap().str_value();
                ui.label(value);
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
