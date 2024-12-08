use super::Settings;
use egui::Ui;
use egui_ext::color;
use egui_plot::{Plot, Points};
use itertools::izip;
use polars::prelude::*;
use std::iter::zip;

/// Plot view
#[derive(Clone, Debug)]
pub(crate) struct PlotView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl<'a> PlotView<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> PolarsResult<Self> {
        Ok(Self {
            data_frame,
            settings,
        })
    }
}

impl PlotView<'_> {
    pub(super) fn ui(&mut self, ui: &mut Ui) {
        self.try_ui(ui).unwrap();
    }

    fn try_ui(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        // let mode = &self.data_frame["Mode"];
        let index = self.data_frame["Index"].u32()?;
        let time = self.data_frame["Time"].list()?;
        let ecl = self.data_frame["ECL"].list()?;
        // let time = time.f64()?;
        // let ecl = ecl.f64()?;
        let mut plot = Plot::new("plot")
            // .allow_drag(context.settings.visualization.drag)
            // .allow_scroll(context.settings.visualization.scroll)
            ;
        // if context.settings.visualization.legend {
        //     plot = plot.legend(Default::default());
        // }
        plot.show(ui, |ui| {
            for (index, time, ecl) in izip!(index, time, ecl) {
                if let Some((time, ecl)) = time.zip(ecl) {
                    let mut points = Vec::new();
                    for (time, ecl) in zip(time.f64().unwrap(), ecl.f64().unwrap()) {
                        if let Some((time, ecl)) = time.zip(ecl) {
                            points.push([time, ecl]);
                        }
                    }
                    ui.points(
                        Points::new(points)
                            .color(color(index.unwrap() as _))
                            .radius(3.0),
                    );
                }
            }
            // let mut offsets = HashMap::new();
            // for (key, values) in visualized {
            //     // Bars
            //     let mut offset = 0.0;
            //     let x = key.into_inner();
            //     for (name, value) in values {
            //         let mut y = value;
            //         if percent {
            //             y *= 100.0;
            //         }
            //         let bar = Bar::new(x, y).name(name).base_offset(offset);
            //         let chart = BarChart::new(vec![bar])
            //             .width(context.settings.visualization.width)
            //             .name(x)
            //             .color(color(x as _));
            //         ui.bar_chart(chart);
            //         offset += y;
            //     }
            //     // // Text
            //     // if context.settings.visualization.text.show
            //     //     && offset >= context.settings.visualization.text.min
            //     // {
            //     //     let y = offset;
            //     //     let text = Text::new(
            //     //         PlotPoint::new(x, y),
            //     //         RichText::new(format!("{y:.p$}"))
            //     //             .size(context.settings.visualization.text.size)
            //     //             .heading(),
            //     //     )
            //     //     .name(x)
            //     //     .color(color(x as _))
            //     //     .anchor(Align2::CENTER_BOTTOM);
            //     //     ui.text(text);
            //     // }
            // }
        });
        Ok(())
    }
}
