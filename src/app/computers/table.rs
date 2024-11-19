use crate::app::panes::settings::{Settings, Sort};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::{frame::DataFrame, prelude::*};
use std::hash::{Hash, Hasher};
use tracing::{error, trace, warn};

/// Filter computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Filter computer
#[derive(Default)]
pub(crate) struct Computer;

// fn signal() -> Expr {
//     col("").struct_().field_by_name("Signal")
// }

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        let mut data_frame = key.data_frame.clone();
        error!(?data_frame);
        // {
        //     let data_frame = data_frame
        //         .clone()
        //         .lazy()
        //         .select([
        //             col("RetentionTime"),
        //             col("Masspectrum").alias("MassSpectrum"),
        //         ])
        //         // .explode(["Masspectrum"])
        //         // .unnest(["Masspectrum"])
        //         //     .sort(["MassToCharge"], Default::default())
        //         //     .group_by([col("RetentionTime")])
        //         //     .agg([as_struct(vec![
        //         //         col("MassToCharge").drop_nulls(),
        //         //         col("Signal").drop_nulls(),
        //         //     ])
        //         //     .alias("MassSpectrum")])
        //         .collect()
        //         .unwrap();
        //     let contents = bincode::serialize(&data_frame).unwrap();
        //     std::fs::write("df.msv.bin", &contents).unwrap();
        //     // // let contents = ron::ser::to_string_pretty(&data_frame, Default::default()).unwrap();
        //     // // std::fs::write("df.msv.ron", &contents).unwrap();
        //     error!(?data_frame);
        // }
        let mut lazy_frame = data_frame.lazy();
        if key.settings.filter_null {
            lazy_frame = lazy_frame.filter(col("MassSpectrum").list().len().neq(lit(0)));
        }
        if key.settings.normalize {
            lazy_frame = lazy_frame
                .explode(["MassSpectrum"])
                .unnest(["MassSpectrum"])
                .with_column(col("Signal").cast(DataType::Float32) / max("Signal"))
                .group_by([col("RetentionTime")])
                .agg([as_struct(vec![col("MassToCharge"), col("Signal")]).alias("MassSpectrum")]);
        }
        match key.settings.sort {
            Sort::RetentionTime if key.settings.explode => {
                lazy_frame = lazy_frame
                    .explode(["MassSpectrum"])
                    .unnest(["MassSpectrum"])
                    .sort_by_exprs([col("RetentionTime")], Default::default());
            }
            Sort::RetentionTime => {
                lazy_frame = lazy_frame
                    .with_columns([
                        col("MassSpectrum").list().len().name().suffix(".Count"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("MassToCharge"), true)
                            .list()
                            .min()
                            .alias("MassToCharge.Min"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("MassToCharge"), true)
                            .list()
                            .max()
                            .alias("MassToCharge.Max"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .min()
                            .alias("Signal.Min"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .max()
                            .alias("Signal.Max"),
                        col("MassSpectrum")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .sum()
                            .alias("Signal.Sum"),
                    ])
                    .sort_by_exprs([col("RetentionTime")], Default::default());
            }
            Sort::MassToCharge if key.settings.explode => {
                lazy_frame = lazy_frame
                    .explode(["MassSpectrum"])
                    .unnest(["MassSpectrum"])
                    .sort_by_exprs([col("MassToCharge")], Default::default());
            }
            Sort::MassToCharge => {
                trace!(lazy_data_frame =? lazy_frame.clone().collect());
                lazy_frame = lazy_frame
                    .explode(["MassSpectrum"])
                    .unnest(["MassSpectrum"])
                    .sort_by_exprs([col("RetentionTime")], Default::default())
                    .group_by([col("MassToCharge").round(0)])
                    .agg([as_struct(vec![
                        col("RetentionTime").drop_nulls(),
                        col("Signal").drop_nulls(),
                    ])
                    .alias("ExtractedIonChromatogram")])
                    .sort_by_exprs([col("MassToCharge")], Default::default())
                    .with_columns([
                        col("ExtractedIonChromatogram")
                            .list()
                            .len()
                            .name()
                            .suffix(".Count"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("RetentionTime"), true)
                            .list()
                            .min()
                            .alias("RetentionTime.Min"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("RetentionTime"), true)
                            .list()
                            .max()
                            .alias("RetentionTime.Max"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .min()
                            .alias("Signal.Min"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .max()
                            .alias("Signal.Max"),
                        col("ExtractedIonChromatogram")
                            .list()
                            .eval(col("").struct_().field_by_name("Signal"), true)
                            .list()
                            .sum()
                            .alias("Signal.Sum"),
                    ]);
            }
        };
        data_frame = lazy_frame.collect().unwrap();
        trace!(?data_frame);
        data_frame
    }
}

/// Filter key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // self.context.state.index.hash(state);
        self.settings.hash(state);
    }
}
