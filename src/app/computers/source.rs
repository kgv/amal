use crate::{
    app::panes::source::settings::{Order, Settings, Sort},
    special::polars::{ExprExt as _, Mass as _},
};
use egui::{
    emath::Float,
    util::cache::{ComputerMut, FrameCache},
};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Source computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Source computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // Mode, FA, Time
        lazy_frame = lazy_frame.select([
            as_struct(vec![
                col("OnsetTemperature").alias("OnsetTemperature"),
                col("TemperatureStep").alias("TemperatureStep"),
            ])
            .alias("Mode"),
            col("FA"),
            as_struct(vec![
                col("Time").list().mean().alias("Mean"),
                col("Time").list().std(0).alias("StandardDeviation"),
                col("Time").alias("Values"),
            ])
            .alias("Time"),
        ]);
        // Relative time, ECL, ECN, Mass
        lazy_frame = lazy_frame
            .with_columns([
                col("Time")
                    .struct_()
                    .with_fields(vec![relative_time().over(["Mode"]).alias("Relative")])?,
                ecl().over(["Mode"]).alias("ECL"),
            ])
            .with_columns([
                (col("ECL") - col("FA").fa().c()).alias("FCL"),
                col("FA").fa().ecn().alias("ECN"),
                as_struct(vec![
                    col("FA").fa().mass().alias("RCOOH"),
                    col("FA").fa().rcoo().mass().alias("RCOO"),
                    col("FA").fa().rcooch3().mass().alias("RCOOCH3"),
                ])
                .alias("Mass"),
            ]);
        // Filter
        if !key.settings.filter.fatty_acids.is_empty() {
            let mut expr = lit(false);
            for fatty_acid in &key.settings.filter.fatty_acids {
                // expr = expr.and(col("FA").fa().c().eq(lit(fatty_acid.carbons)).not());
                let indices = Scalar::new(
                    DataType::List(Box::new(DataType::UInt8)),
                    AnyValue::List(Series::from_iter(&fatty_acid.indices)),
                );
                let bounds = Scalar::new(
                    DataType::List(Box::new(DataType::Int8)),
                    AnyValue::List(Series::from_iter(&fatty_acid.bounds)),
                );
                expr = expr.or(col("FA")
                    .fa()
                    .c()
                    .eq(lit(fatty_acid.carbons))
                    .and(col("FA").fa().indices().eq(lit(indices)))
                    .and(col("FA").fa().bounds().eq(lit(bounds))));
            }
            lazy_frame = lazy_frame.filter(expr);
        }
        // Interpolate
        // if key.settings.interpolate {
        //     // lazy_frame = lazy_frame.with_columns([
        //     //     // col("Time").over(["Mode"]),
        //     //     col("Time")
        //     //         .struct_()
        //     //         .field_by_name("Mean")
        //     //         .fill_null(lit(0.0))
        //     //         .over(["Mode"])
        //     //         .alias("INTERPOLATE_TIME"),
        //     //     col("ECL")
        //     //         // .interpolate_by(col("ECN"))
        //     //         .interpolate(
        //     //             col("Time")
        //     //                 .struct_()
        //     //                 .field_by_name("Mean")
        //     //                 .fill_null(lit(0.0)),
        //     //         )
        //     //         // .interpolate(InterpolationMethod::Linear)
        //     //         .over(["Mode"])
        //     //         .alias("INTERPOLATE_ECL"),
        //     // ]);
        // }

        // Sort
        let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
        if key.settings.order == Order::Descending {
            sort_options = sort_options.with_order_descending(true);
        };
        lazy_frame = match key.settings.sort {
            Sort::Mode => lazy_frame.sort_by_exprs(&[col("Mode"), col("FA")], sort_options),
            Sort::Ecl => lazy_frame.sort_by_exprs(&[col("ECL").over([col("Mode")])], sort_options),
            Sort::Time => {
                lazy_frame.sort_by_exprs(&[col("Time").over([col("Mode")])], sort_options)
            }
        };
        // Index
        lazy_frame = lazy_frame.cache().with_row_index("Index", None);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).expect("compute source")
    }
}

/// Source key
#[derive(Clone, Copy, Debug)]
pub struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

// impl Hash for Key<'_> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         for column in self.data_frame.get_columns() {
//             for value in column.phys_iter() {
//                 value.hash(state);
//             }
//         }
//         self.settings.hash(state);
//     }
// }
impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // for column in self.data_frame.get_columns() {
        //     for value in column.phys_iter() {
        //         value.hash(state);
        //     }
        // }
        // for value in self.data_frame["OnsetTemperature"].f64().unwrap() {
        //     value.map(|value| value.ord()).hash(state);
        // }
        // for value in self.data_frame["TemperatureStep"].f64().unwrap() {
        //     value.map(|value| value.ord()).hash(state);
        // }
        // for value in self.data_frame["Time"].vec_hash() {
        //     value.hash(state);
        // }
        // for series in self.data_frame.iter() {
        //     for value in series.iter() {
        //         value.hash(state);
        //     }
        // }

        self.settings.filter.hash(state);
        self.settings.filter_onset_temperature.hash(state);
        self.settings.filter_temperature_step.hash(state);
        self.settings.interpolate.hash(state);
        self.settings.interpolation.hash(state);
        self.settings.sort.hash(state);
        self.settings.order.hash(state);
    }
}

// col("FA")
//                 .fa()
//                 .c()
//                 .eq(lit(18))
//                 .and(col("FA").fa().saturated()),
fn diff(left: Expr, right: Expr) -> Expr {
    (col("Time")
        .struct_()
        .field_by_name("Mean")
        .filter(left)
        .first()
        - col("Time")
            .struct_()
            .field_by_name("Mean")
            .filter(right)
            .first())
    .abs()
}

fn relative_time() -> Expr {
    col("Time").struct_().field_by_name("Mean")
        / col("Time")
            .struct_()
            .field_by_name("Mean")
            .filter(
                col("FA")
                    .fa()
                    .saturated()
                    .and(col("FA").fa().c().eq(lit(18))),
            )
            .first()
}

// fn ecl(fa: Expr, time: Expr) -> Expr {
//     as_struct(vec![fa, time]).apply(
//         |column| {
//             let chunked_array = column.struct_()?.field_by_name("FA")?;
//             let times = column.struct_()?.field_by_name("Time")?;
//             let chunked_array = column.f64()?;
//             let time = times.f64()?;
//             time.shift(1);
//             Ok(Some(Column::Series(
//                 chunked_array
//                     .into_iter()
//                     .map(|option| Some(option.unwrap_or_default() / chunked_array.sum()?))
//                     .collect(),
//             )))
//         },
//         GetOutput::same_type(),
//     )
// }
fn ecl() -> Expr {
    ternary_expr(
        col("FA").fa().saturated(),
        col("FA").fa().c(),
        ternary_expr(col("FA").fa().saturated(), col("FA").fa().c(), lit(NULL)).forward_fill(None)
            + (col("Time").struct_().field_by_name("Mean") - time().forward_fill(None))
                / (time().backward_fill(None) - time().forward_fill(None)),
    )
}

fn time() -> Expr {
    ternary_expr(
        col("FA").fa().saturated(),
        col("Time").struct_().field_by_name("Mean"),
        lit(NULL),
    )
}
