use crate::{
    app::{localize, MAX_PRECISION},
    special::fa_column::{ColumnExt, FattyAcid},
};
use egui::{emath::Float, ComboBox, Grid, Slider, Ui};
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

    pub(crate) filter: Filter,
    pub(crate) interpolate: bool,
    pub(crate) interpolation: Interpolation,
    pub(crate) filter_onset_temperature: Option<i32>,
    pub(crate) filter_temperature_step: Option<i32>,
    pub(crate) sort: Sort,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            precision: 2,
            resizable: false,
            sticky: 1,
            truncate: false,

            filter: Filter::new(),
            interpolate: false,
            interpolation: Interpolation::new(),
            filter_onset_temperature: None,
            filter_temperature_step: None,
            sort: Sort::Time,
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

            // Interpolate ECL
            ui.label(localize!("interpolate"));
            ui.checkbox(&mut self.interpolate, "");
            ui.end_row();

            // ui.label("Interpolation");
            ui.label(localize!("onset-temperature"));
            let (min, max) = data_frame["OnsetTemperature"]
                .f64()
                .unwrap()
                .min_max()
                .unwrap();
            ui.add(Slider::new(
                &mut self.interpolation.onset_temperature,
                min..=max,
            ));
            ui.end_row();

            ui.label(localize!("temperature-step"));
            let (min, max) = data_frame["TemperatureStep"]
                .f64()
                .unwrap()
                .min_max()
                .unwrap();
            ui.add(Slider::new(
                &mut self.interpolation.temperature_step,
                min..=max,
            ));
            ui.end_row();

            // Filter
            ui.label("Filter");
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("FilterFattyAcids")
                    // .selected_text(self.sort.text())
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
                    ui.selectable_value(&mut self.sort, Sort::Ecl, "ECL");
                    ui.selectable_value(&mut self.sort, Sort::Time, "Time");
                });
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
    pub(crate) fatty_acids: Vec<FattyAcid>,
}

impl Filter {
    pub const fn new() -> Self {
        Self {
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

/// Interpolation
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Interpolation {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Interpolation {
    pub const fn new() -> Self {
        Self {
            onset_temperature: 0.0,
            temperature_step: 0.0,
        }
    }
}

impl Hash for Interpolation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

/// Sort
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    // RetentionTime
    Time,
    Ecl,
}
