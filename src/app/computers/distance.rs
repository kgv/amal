use crate::{
    app::panes::distance::settings::{Settings, Sort, SortBy},
    special::polars::{ExprExt as _, Mass as _, Rcooh},
};
use egui::{
    emath::Float,
    util::cache::{ComputerMut, FrameCache},
};
use polars::{
    lazy::dsl::{max_horizontal, min_horizontal},
    prelude::*,
};
use std::hash::{Hash, Hasher};

/// Distance computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Distance computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<DataFrame> {
        // Mode, FA, Time
        let mut lazy_frame = key.data_frame.clone().lazy().select([
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
        lazy_frame = lazy_frame.with_columns([
            col("Time")
                .struct_()
                .with_fields(vec![relative_time().over(["Mode"]).alias("Relative")])?,
            ecl().over(["Mode"]).alias("ECL"),
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
                let fa = col("FA")
                    .fa()
                    .c()
                    .eq(lit(fatty_acid.carbons))
                    .and(col("FA").fa().indices().eq(lit(indices)))
                    .and(col("FA").fa().bounds().eq(lit(bounds)));
                expr = expr.or(fa.clone());
            }
            lazy_frame = lazy_frame.filter(expr);
            // .with_columns([col("Time")
            //     .struct_()
            //     .field_by_name("Mean")
            //     // .filter(fa)
            //     .diff(1, NullBehavior::Ignore)
            //     .over(["FA"])
            //     .alias("TEMP")]);
            // // Sort
            // lazy_frame = lazy_frame.sort_by_exprs(
            //     [diff(left, right).over(["Mode"])],
            //     SortMultipleOptions::default().with_order_reversed(),
            // );
        }
        // Join
        lazy_frame = lazy_frame
            .clone()
            .select([
                as_struct(vec![col("FA"), col("Mode")]).alias("From"),
                col("Time")
                    .struct_()
                    .field_by_name("Mean")
                    .alias("FromTime"),
                col("ECL").alias("FromECL"),
            ])
            .with_row_index("LeftIndex", None)
            .join_builder()
            .with(
                lazy_frame
                    .select([
                        as_struct(vec![col("FA"), col("Mode")]).alias("To"),
                        col("Time").struct_().field_by_name("Mean").alias("ToTime"),
                        col("ECL").alias("ToECL"),
                    ])
                    .with_row_index("RightIndex", None),
            )
            .join_where(vec![
                // Same modes
                col("From")
                    .struct_()
                    .field_by_name("Mode")
                    .struct_()
                    .field_by_name("OnsetTemperature")
                    .eq(col("To")
                        .struct_()
                        .field_by_name("Mode")
                        .struct_()
                        .field_by_name("OnsetTemperature"))
                    .and(
                        col("From")
                            .struct_()
                            .field_by_name("Mode")
                            .struct_()
                            .field_by_name("TemperatureStep")
                            .eq(col("To")
                                .struct_()
                                .field_by_name("Mode")
                                .struct_()
                                .field_by_name("TemperatureStep")),
                    ),
                // Fatty asids not equals combination
                col("LeftIndex").lt(col("RightIndex")),
            ]);
        // Cache
        lazy_frame = lazy_frame.cache().select([
            col("From").struct_().field_by_name("Mode").alias("Mode"),
            col("From").struct_().field_by_name("FA").name().keep(),
            col("To").struct_().field_by_name("FA").name().keep(),
            (col("ToTime") - col("FromTime"))
                .over([col("From").struct_().field_by_name("Mode")])
                .alias("Time"),
            (col("ToECL") - col("FromECL"))
                .over([col("From").struct_().field_by_name("Mode")])
                .alias("ECL"),
        ]);
        // Sort
        let name = match key.settings.sort_by {
            SortBy::Ecl => "ECL",
            SortBy::Time => "Time",
        };
        lazy_frame = lazy_frame.sort_by_exprs(
            [col(name).abs().median().over([col("Mode")])],
            SortMultipleOptions::new().with_order_reversed(),
        );
        // Index
        lazy_frame = lazy_frame.with_row_index("Index", None);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        self.try_compute(key).unwrap()
    }
}

/// Distance key
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
        for value in self.data_frame["OnsetTemperature"].f64().unwrap() {
            value.map(|value| value.ord()).hash(state);
        }
        for value in self.data_frame["TemperatureStep"].f64().unwrap() {
            value.map(|value| value.ord()).hash(state);
        }
        // for value in self.data_frame["Time"].vec_hash() {
        //     value.hash(state);
        // }
        // for series in self.data_frame.iter() {
        //     for value in series.iter() {
        //         value.hash(state);
        //     }
        // }
        self.settings.hash(state);
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
