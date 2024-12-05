use crate::{
    app::localize,
    special::fa_column::{ColumnExt, FattyAcid},
};
use egui::{emath::Float, ComboBox, Grid, Slider, Ui, WidgetText};
use egui_phosphor::regular::TRASH;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};
use uom::si::{
    f32::Time,
    time::{millisecond, minute, second, Units},
};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) sticky_columns: usize,
    pub(crate) resizable: bool,
    pub(crate) filter: Filter,
    pub(crate) interpolation: Interpolation,
    pub(crate) filter_onset_temperature: Option<i32>,
    pub(crate) filter_temperature_step: Option<i32>,
    pub(crate) sort_by: SortBy,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            sticky_columns: 1,
            resizable: false,
            filter: Filter::new(),
            interpolation: Interpolation::new(),
            filter_onset_temperature: None,
            filter_temperature_step: None,
            sort_by: SortBy::Time,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("calculation").show(ui, |ui| {
            // Sticky
            ui.label(localize!("sticky"));
            ui.add(Slider::new(
                &mut self.sticky_columns,
                0..=data_frame.width(),
            ));
            ui.end_row();

            ui.separator();
            ui.separator();
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

            ui.label("Sort");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, SortBy::Ecl, "ECL");
                    ui.selectable_value(&mut self.sort_by, SortBy::Time, "Time");
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    #[default]
    RetentionTime,
    MassToCharge,
}

impl Sort {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::RetentionTime => "Retention time",
            Self::MassToCharge => "Mass to charge",
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::RetentionTime => "Sort by retention time column",
            Self::MassToCharge => "Sort by mass to charge column",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum SortBy {
    Time,
    Ecl,
}

/// Mass to charge settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct MassToCharge {
    pub(crate) precision: usize,
}

impl Default for MassToCharge {
    fn default() -> Self {
        Self { precision: 1 }
    }
}

impl MassToCharge {
    pub(crate) fn format(self, value: f32) -> MassToChargeFormat {
        MassToChargeFormat {
            value,
            precision: Some(self.precision),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MassToChargeFormat {
    value: f32,
    precision: Option<usize>,
}

impl MassToChargeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for MassToChargeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let value = self.value;
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<MassToChargeFormat> for WidgetText {
    fn from(value: MassToChargeFormat) -> Self {
        value.to_string().into()
    }
}

/// Retention time settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct RetentionTime {
    pub(crate) precision: usize,
    pub(crate) units: TimeUnits,
}

impl RetentionTime {
    pub(crate) fn format(self, value: f32) -> RetentionTimeFormat {
        RetentionTimeFormat {
            value,
            precision: Some(self.precision),
            units: self.units,
        }
    }
}

impl Default for RetentionTime {
    fn default() -> Self {
        Self {
            precision: 2,
            units: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RetentionTimeFormat {
    value: f32,
    precision: Option<usize>,
    units: TimeUnits,
}

impl RetentionTimeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for RetentionTimeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let time = Time::new::<millisecond>(self.value as _);
        let value = match self.units {
            TimeUnits::Millisecond => time.get::<millisecond>(),
            TimeUnits::Second => time.get::<second>(),
            TimeUnits::Minute => time.get::<minute>(),
        };
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<RetentionTimeFormat> for WidgetText {
    fn from(value: RetentionTimeFormat) -> Self {
        value.to_string().into()
    }
}

/// Time units
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TimeUnits {
    Millisecond,
    #[default]
    Second,
    Minute,
}

impl TimeUnits {
    pub fn abbreviation(&self) -> &'static str {
        Units::from(*self).abbreviation()
    }

    pub fn singular(&self) -> &'static str {
        Units::from(*self).singular()
    }

    pub fn plural(&self) -> &'static str {
        Units::from(*self).plural()
    }
}

impl From<TimeUnits> for Units {
    fn from(value: TimeUnits) -> Self {
        match value {
            TimeUnits::Millisecond => Units::millisecond(millisecond),
            TimeUnits::Second => Units::second(second),
            TimeUnits::Minute => Units::minute(minute),
        }
    }
}