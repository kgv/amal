use crate::{
    app::{
        panes::source::settings::{Order, Settings, Sort},
        MAX_TEMPERATURE,
    },
    special::expressions::fatty_acid::{ExprExt as _, FattyAcid as _},
};
use egui::util::cache::{ComputerMut, FrameCache};
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
        // Time
        lazy_frame = lazy_frame.with_column(
            as_struct(vec![
                col("Time").list().mean().alias("Mean"),
                col("Time")
                    .list()
                    .std(key.settings.ddof)
                    .alias("StandardDeviation"),
                col("Time").alias("Values"),
            ])
            .alias("Time"),
        );
        // Relative time, Temperature, ECL, ECN, Mass
        lazy_frame = lazy_frame
            .with_columns([
                col("Time")
                    .struct_()
                    .with_fields(vec![relative_time(key.settings)
                        .over(["Mode"])
                        .alias("Relative")])?,
                ecl().over(["Mode"]).alias("ECL"),
                (col("Mode").struct_().field_by_name("OnsetTemperature")
                    + col("Time").struct_().field_by_name("Mean")
                        * col("Mode").struct_().field_by_name("TemperatureStep"))
                .clip_max(lit(MAX_TEMPERATURE))
                .alias("Temperature"),
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
        // // Meta
        // lazy_frame = concat_lf_horizontal(
        //     [
        //         lazy_frame.clone(),
        //         lazy_frame
        //             .select([
        //                 saturated(col("Temperature"))
        //                     .backward_fill(None)
        //                     .over(["Mode"])
        //                     .alias("BackwardTemperature"),
        //                 saturated(col("Temperature"))
        //                     .forward_fill(None)
        //                     .over(["Mode"])
        //                     .alias("ForwardTemperature"),
        //                 saturated(col("Time").struct_().field_by_name("Mean"))
        //                     .backward_fill(None)
        //                     .over(["Mode"])
        //                     .alias("BackwardTime"),
        //                 saturated(col("Time").struct_().field_by_name("Mean"))
        //                     .forward_fill(None)
        //                     .over(["Mode"])
        //                     .alias("ForwardTime"),
        //             ])
        //             .with_columns([
        //                 (col("BackwardTemperature") - col("ForwardTemperature"))
        //                     .alias("TemperatureDistance"),
        //                 (col("BackwardTime") - col("ForwardTime")).alias("TimeDistance"),
        //             ])
        //             .select([as_struct(vec![
        //                 col("BackwardTemperature"),
        //                 col("ForwardTemperature"),
        //                 col("TemperatureDistance"),
        //                 col("BackwardTime"),
        //                 col("ForwardTime"),
        //                 col("TimeDistance"),
        //                 (col("TimeDistance") / col("TemperatureDistance")).alias("Slope"),
        //             ])
        //             .alias("Meta")]),
        //     ],
        //     UnionArgs::default(),
        // )?;
        // Filter
        if let Some(onset_temperature) = key.settings.filter.mode.onset_temperature {
            lazy_frame = lazy_frame.filter(
                col("Mode")
                    .struct_()
                    .field_by_name("OnsetTemperature")
                    .eq(lit(onset_temperature)),
            );
        }
        if let Some(temperature_step) = key.settings.filter.mode.temperature_step {
            lazy_frame = lazy_frame.filter(
                col("Mode")
                    .struct_()
                    .field_by_name("TemperatureStep")
                    .eq(lit(temperature_step)),
            );
        }
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
            Sort::Mode => lazy_frame.sort_by_fatty_acids(sort_options),
            Sort::Ecl => lazy_frame.sort_by_ecl(sort_options),
            Sort::Time => lazy_frame.sort_by_time(sort_options),
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

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.settings.ddof.hash(state);
        self.settings.relative.hash(state);
        self.settings.filter.hash(state);
        self.settings.sort.hash(state);
        self.settings.order.hash(state);
    }
}

/// Extension methods for [`LazyFrame`]
trait LazyFrameExt {
    fn sort_by_ecl(self, sort_options: SortMultipleOptions) -> LazyFrame;

    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame;

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame;
}

impl LazyFrameExt for LazyFrame {
    fn sort_by_ecl(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort(["Mode"], sort_options.clone()).select([all()
            .sort_by(&[col("ECL")], sort_options)
            .over([col("Mode")])])
    }

    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort_by_exprs(
            [
                col("Mode"),
                col("FA").fa().c(),
                col("FA").fa().unsaturation(),
                col("FA").fa().indices(),
                col("FA").fa().bounds(),
                // col("FA").fa().bounds().list().eval(-col(""), true),
            ],
            sort_options,
        )
    }

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort(["Mode"], sort_options.clone()).select([all()
            .sort_by(&[col("Time")], sort_options)
            .over([col("Mode")])])
    }
}

// col("FA")
//                 .fa()
//                 .c()
//                 .eq(lit(18))
//                 .and(col("FA").fa().saturated()),
fn distance(left: Expr, right: Expr) -> Expr {
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

fn relative_time(settings: &Settings) -> Expr {
    let mut expr = col("Time").struct_().field_by_name("Mean");
    if let Some(relative) = &settings.relative {
        expr = expr
            / col("Time")
                .struct_()
                .field_by_name("Mean")
                .filter(
                    col("FA")
                        .fa()
                        .c()
                        .eq(lit(relative.carbons))
                        .and(col("FA").fa().indices().eq(lit(Scalar::new(
                            DataType::List(Box::new(DataType::UInt8)),
                            AnyValue::List(Series::from_iter(relative.indices.iter())),
                        ))))
                        .and(col("FA").fa().bounds().eq(lit(Scalar::new(
                            DataType::List(Box::new(DataType::Int8)),
                            AnyValue::List(Series::from_iter(relative.bounds.iter())),
                        )))),
                )
                .first()
    }
    expr
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

fn slope() -> Expr {
    ternary_expr(
        col("FA").fa().saturated(),
        lit(0),
        ternary_expr(col("FA").fa().saturated(), col("FA").fa().c(), lit(NULL)).forward_fill(None)
            + (col("Time").struct_().field_by_name("Mean") - time().forward_fill(None))
                / (time().backward_fill(None) - time().forward_fill(None)),
    )
}

fn saturated(expr: Expr) -> Expr {
    ternary_expr(col("FA").fa().saturated(), expr, lit(NULL))
}

fn time() -> Expr {
    ternary_expr(
        col("FA").fa().saturated(),
        col("Time").struct_().field_by_name("Mean"),
        lit(NULL),
    )
}
