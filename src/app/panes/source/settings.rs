use crate::{
    app::{localize, text::Text, MAX_PRECISION},
    special::fa_column::{ColumnExt, FattyAcid},
};
use egui::{
    emath::{Float, OrderedFloat},
    ComboBox, Grid, Slider, Ui,
};
use egui_phosphor::regular::TRASH;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) interpolate: bool,
    // pub(crate) interpolation: Mode,
    // pub(crate) filter_onset_temperature: Option<i32>,
    // pub(crate) filter_temperature_step: Option<i32>,
    pub(crate) filter: Filter,
    pub(crate) sort: Sort,
    pub(crate) order: Order,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 1,
            truncate: false,

            interpolate: false,
            // interpolation: Mode::new(),
            // filter_onset_temperature: None,
            // filter_temperature_step: None,
            filter: Filter::new(),
            sort: Sort::Mode,
            order: Order::Descending,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("calculation").show(ui, |ui| {
            // Precision floats
            ui.label(localize!("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Sticky columns
            ui.label(localize!("sticky"));
            ui.add(Slider::new(&mut self.sticky, 0..=data_frame.width()));
            ui.end_row();

            // Truncate titles
            ui.label(localize!("truncate"));
            ui.checkbox(&mut self.truncate, "");
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Filter
            // Onset temperature filter
            ui.label("Onset").on_hover_text("Onset temperature filter");
            ComboBox::from_id_salt("FilterOnsetTemperature")
                .selected_text(format!("{:?}", self.filter.mode.onset_temperature))
                .show_ui(ui, |ui| {
                    let current_value = &mut self.filter.mode.onset_temperature;
                    let onset_temperature = data_frame["OnsetTemperature"]
                        .f64()
                        .unwrap()
                        .unique()
                        .unwrap();
                    for selected_value in &onset_temperature {
                        ui.selectable_value(
                            current_value,
                            selected_value,
                            AnyValue::from(selected_value).to_string(),
                        );
                    }
                    ui.selectable_value(current_value, None, "None");
                });
            ui.end_row();

            // Temperature step filter
            ui.label("Step").on_hover_text("Temperature step filter");
            ComboBox::from_id_salt("FilterTemperatureStep")
                .selected_text(format!("{:?}", self.filter.mode.temperature_step))
                .show_ui(ui, |ui| {
                    let current_value = &mut self.filter.mode.temperature_step;
                    let temperature_step = data_frame["TemperatureStep"]
                        .f64()
                        .unwrap()
                        .unique()
                        .unwrap();
                    for selected_value in &temperature_step {
                        ui.selectable_value(
                            current_value,
                            selected_value,
                            AnyValue::from(selected_value).to_string(),
                        );
                    }
                    ui.selectable_value(current_value, None, "None");
                });
            ui.end_row();

            // Fatty acids filter
            ui.label("Filter");
            // let text = AnyValue::List(Series::from_iter(
            //     self.filter
            //         .fatty_acids
            //         .iter()
            //         .map(|fatty_acid| fatty_acid.to_string()),
            // ))
            // .to_string();
            let text = self.filter.fatty_acids.len().to_string();
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("FilterFattyAcids")
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        let fatty_acids = data_frame["FA"]
                            .unique()
                            .unwrap()
                            .sort(Default::default())
                            .unwrap()
                            .fa();
                        for fatty_acid in fatty_acids.iter().unwrap() {
                            let contains = self.filter.fatty_acids.contains(&fatty_acid);
                            let mut selected = contains;
                            ui.toggle_value(&mut selected, fatty_acid.to_string());
                            if selected && !contains {
                                self.filter.fatty_acids.push(fatty_acid);
                            } else if !selected && contains {
                                self.filter.remove(&fatty_acid);
                            }
                        }
                    });
                if ui.button(TRASH).clicked() {
                    self.filter.fatty_acids = Vec::new();
                }
            });
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Sort
            ui.label("Sort");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(format!("{:?}", self.sort))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort, Sort::Mode, "Mode");
                    ui.selectable_value(&mut self.sort, Sort::Ecl, "ECL");
                    ui.selectable_value(&mut self.sort, Sort::Time, "Time");
                });
            ui.end_row();

            // Order
            ui.label("Order");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(self.order.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.order, Order::Ascending, Order::Ascending.text())
                        .on_hover_text(Order::Ascending.hover_text());
                    ui.selectable_value(
                        &mut self.order,
                        Order::Descending,
                        Order::Descending.text(),
                    )
                    .on_hover_text(Order::Descending.hover_text());
                })
                .response
                .on_hover_text(self.order.hover_text());
            ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Filter {
    pub(crate) mode: Mode,
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Filter {
    pub const fn new() -> Self {
        Self {
            mode: Mode::new(),
            fatty_acids: Vec::new(),
        }
    }
}

impl Filter {
    fn remove(&mut self, target: &FattyAcid) -> Option<FattyAcid> {
        let position = self
            .fatty_acids
            .iter()
            .position(|source| source == target)?;
        Some(self.fatty_acids.remove(position))
    }
}

/// Mode
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Mode {
    pub(crate) onset_temperature: Option<f64>,
    pub(crate) temperature_step: Option<f64>,
}

impl Mode {
    pub const fn new() -> Self {
        Self {
            onset_temperature: None,
            temperature_step: None,
        }
    }
}

impl Hash for Mode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.map(Float::ord).hash(state);
        self.temperature_step.map(Float::ord).hash(state);
    }
}

/// Sort
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Mode,
    Time,
    Ecl,
}

impl Text for Sort {
    fn text(&self) -> &'static str {
        match self {
            Self::Mode => "Mode",
            Self::Time => "Time",
            Self::Ecl => "ECL",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Mode => "Mode",
            Self::Time => "Retention time",
            Self::Ecl => "Equivalent carbon number",
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(in crate::app) enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub(in crate::app) fn text(self) -> &'static str {
        match self {
            Self::Ascending => "Ascending",
            Self::Descending => "Descending",
        }
    }

    pub(in crate::app) fn hover_text(self) -> &'static str {
        match self {
            Self::Ascending => "Dscending",
            Self::Descending => "Descending",
        }
    }
}
