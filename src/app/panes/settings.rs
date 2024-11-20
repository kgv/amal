use crate::app::MAX_PRECISION;
use egui::{emath::Float, ComboBox, DragValue, Slider, Ui, WidgetText};
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
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) sticky_columns: usize,
    pub(crate) resizable: bool,
    pub(crate) interpolation: Interpolation,
    pub(crate) filter_onset_temperature: Option<i32>,
    pub(crate) filter_temperature_step: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sticky_columns: 1,
            resizable: false,
            interpolation: Default::default(),
            filter_onset_temperature: None,
            filter_temperature_step: None,
        }
    }
}

/// Interpolation
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Interpolation {
    pub(crate) onset_temperature: f64,
    pub(crate) temperature_step: f64,
}

impl Hash for Interpolation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.onset_temperature.ord().hash(state);
        self.temperature_step.ord().hash(state);
    }
}

impl Settings {
    pub(crate) fn ui(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        ui.add(
            Slider::new(&mut self.sticky_columns, 0..=data_frame.width()).text("Sticky columns"),
        );
        ui.checkbox(&mut self.resizable, "Resizable");
        ui.separator();
        ui.label("Interpolation");
        let (min, max) = data_frame["OnsetTemperature"]
            .f64()
            .unwrap()
            .min_max()
            .unwrap();
        ui.add(Slider::new(
            &mut self.interpolation.onset_temperature,
            min..=max,
        ));
        let (min, max) = data_frame["TemperatureStep"]
            .f64()
            .unwrap()
            .min_max()
            .unwrap();
        ui.add(Slider::new(
            &mut self.interpolation.temperature_step,
            min..=max,
        ));
        ui.label("Filter");
        // ComboBox::from_id_salt("FilterOnsetTemperature")
        //     // .selected_text(self.sort.text())
        //     .show_ui(ui, |ui| {
        //         ui.selectable_value(&mut self.filter_onset_temperature, None, "None");
        //         let onset_temperature = data_frame["OnsetTemperature"]
        //             .unique()
        //             .unwrap()
        //             .sort(Default::default())
        //             .unwrap();
        //         let onset_temperature = onset_temperature.f64().unwrap();
        //         for selected_value in onset_temperature {
        //             ui.selectable_value(
        //                 &mut self.filter_onset_temperature,
        //                 selected_value,
        //                 format!("{selected_value:?}"),
        //             )
        //             .on_hover_text("Sort::RetentionTime.description()");
        //         }
        //     })
        //     .response
        //     .on_hover_text("self.sort.description()");
        // ComboBox::from_id_salt("FilterTemperatureStep")
        //     // .selected_text(self.sort.text())
        //     .show_ui(ui, |ui| {
        //         ui.selectable_value(&mut self.filter_temperature_step, None, "None");
        //         let temperature_step = data_frame["TemperatureStep"]
        //             .unique()
        //             .unwrap()
        //             .sort(Default::default())
        //             .unwrap();
        //         let temperature_step = temperature_step.f64().unwrap();
        //         for selected_value in temperature_step {
        //             ui.selectable_value(
        //                 &mut self.filter_temperature_step,
        //                 selected_value,
        //                 format!("{selected_value:?}"),
        //             )
        //             .on_hover_text("Sort::RetentionTime.description()");
        //         }
        //     })
        //     .response
        //     .on_hover_text("self.sort.description()");

        // ui.horizontal(|ui| {
        //     ui.label("Retention time");
        //     ComboBox::from_id_salt("retention_time_units")
        //         .selected_text(self.retention_time.units.singular())
        //         .show_ui(ui, |ui| {
        //             ui.selectable_value(
        //                 &mut self.retention_time.units,
        //                 TimeUnits::Millisecond,
        //                 TimeUnits::Millisecond.singular(),
        //             )
        //             .on_hover_text(TimeUnits::Millisecond.abbreviation());
        //             ui.selectable_value(
        //                 &mut self.retention_time.units,
        //                 TimeUnits::Second,
        //                 TimeUnits::Second.singular(),
        //             )
        //             .on_hover_text(TimeUnits::Second.abbreviation());
        //             ui.selectable_value(
        //                 &mut self.retention_time.units,
        //                 TimeUnits::Minute,
        //                 TimeUnits::Minute.singular(),
        //             )
        //             .on_hover_text(TimeUnits::Minute.abbreviation());
        //         })
        //         .response
        //         .on_hover_text(format!(
        //             "Units {}",
        //             self.retention_time.units.abbreviation(),
        //         ));
        //     ui.add(DragValue::new(&mut self.retention_time.precision).range(0..=MAX_PRECISION))
        //         .on_hover_text("Precision");
        // });
        // ui.horizontal(|ui| {
        //     ui.label("Mass to charge");
        //     ui.add(DragValue::new(&mut self.mass_to_charge.precision).range(0..=MAX_PRECISION))
        //         .on_hover_text("Precision");
        // });
        // ui.separator();
        // ui.horizontal(|ui| {
        //     ui.label("Explode");
        //     ui.checkbox(&mut self.explode, "")
        //         .on_hover_text("Explode lists");
        // });
        // ui.horizontal(|ui| {
        //     ui.label("Filter empty/null");
        //     ui.checkbox(&mut self.filter_null, "")
        //         .on_hover_text("Filter empty/null retention time");
        // });
        // ui.separator();
        // ui.horizontal(|ui| {
        //     ui.label("Sort");
        //     ComboBox::from_id_source("sort")
        //         .selected_text(self.sort.text())
        //         .show_ui(ui, |ui| {
        //             ui.selectable_value(
        //                 &mut self.sort,
        //                 Sort::RetentionTime,
        //                 Sort::RetentionTime.text(),
        //             )
        //             .on_hover_text(Sort::RetentionTime.description());
        //             ui.selectable_value(
        //                 &mut self.sort,
        //                 Sort::MassToCharge,
        //                 Sort::MassToCharge.text(),
        //             )
        //             .on_hover_text(Sort::MassToCharge.description());
        //         })
        //         .response
        //         .on_hover_text(self.sort.description());
        // });
        // ui.separator();
        // ui.horizontal(|ui| {
        //     ui.label("Normalize");
        //     ui.checkbox(&mut self.normalize, "")
        //         .on_hover_text("Normalize");
        // });
        // ui.separator();
        // ui.horizontal(|ui| {
        //     ui.label("Legend");
        //     ui.checkbox(&mut self.legend, "")
        //         .on_hover_text("Show plot legend");
        // });
        // // ui.horizontal(|ui| {
        // //     ui.selectable_value(&mut self.visible, Some(true), "◉👁");
        // //     ui.selectable_value(&mut self.visible, Some(false), "◎👁");
        // // });
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
