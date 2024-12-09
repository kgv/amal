use crate::{
    app::{
        panes::source::settings::{Group, Kind, Order, Settings, Sort},
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
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<LazyFrame> {
        let mut lazy_frame = key.data_frame.clone().lazy();
        // Meta
        lazy_frame = lazy_frame.with_columns([
            col("Time").list().mean().alias("TimeMean"),
            col("Time")
                .list()
                .std(key.settings.ddof)
                .alias("TimeStandardDeviation"),
        ]);
        lazy_frame = lazy_frame
            .with_columns([
                // Relative time
                relative_time(key.settings)
                    .over(["Mode"])
                    .alias("RelativeTime"),
                // ECL
                ecl().over(["Mode"]).alias("ECL"),
            ])
            .select([
                col("Mode"),
                col("FA"),
                // Time
                as_struct(vec![
                    as_struct(vec![
                        col("TimeMean").alias("Mean"),
                        col("TimeStandardDeviation").alias("StandardDeviation"),
                        col("Time").alias("Values"),
                    ])
                    .alias("Absolute"),
                    col("RelativeTime").alias("Relative"),
                ])
                .alias("Time"),
                // Temperature
                (col("Mode").struct_().field_by_name("OnsetTemperature")
                    + col("TimeMean") * col("Mode").struct_().field_by_name("TemperatureStep"))
                .clip_max(lit(MAX_TEMPERATURE))
                .alias("Temperature"),
                // Equivalent
                as_struct(vec![
                    col("ECL"),
                    (col("ECL") - col("FA").fa().c()).alias("FCL"),
                    col("FA").fa().ecn().alias("ECN"),
                ])
                .alias("Equivalent"),
                // Mass
                as_struct(vec![
                    col("FA").fa().mass().alias("RCOOH"),
                    col("FA").fa().rcoo().mass().alias("RCOO"),
                    col("FA").fa().rcooch3().mass().alias("RCOOCH3"),
                ])
                .alias("Mass"),
            ]);
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
        // Sort
        let mut sort_options = SortMultipleOptions::new().with_nulls_last(true);
        if key.settings.order == Order::Descending {
            sort_options = sort_options.with_order_descending(true);
        };
        lazy_frame = match key.settings.sort {
            Sort::FattyAcid => lazy_frame.sort_by_fatty_acids(sort_options),
            Sort::Time => lazy_frame.sort_by_time(sort_options),
        };
        Ok(lazy_frame)
    }
}

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        let mut lazy_frame = self.try_compute(key).expect("compute source");
        lazy_frame = match key.settings.kind {
            Kind::Plot => {
                lazy_frame = lazy_frame.select([
                    col("Mode"),
                    col("FA"),
                    col("Time")
                        .struct_()
                        .field_by_name("Absolute")
                        .struct_()
                        .field_by_name("Mean")
                        .alias("Time"),
                    col("Equivalent")
                        .struct_()
                        .field_by_name("ECL")
                        .alias("ECL"),
                ]);
                lazy_frame
                    .group_by([match key.settings.group {
                        Group::FattyAcid => col("FA"),
                        Group::OnsetTemperature => {
                            col("Mode").struct_().field_by_name("OnsetTemperature")
                        }
                        Group::TemperatureStep => {
                            col("Mode").struct_().field_by_name("TemperatureStep")
                        }
                    }])
                    .agg([col("Time"), col("ECL")])
            }
            Kind::Table => lazy_frame,
        };
        // Index
        lazy_frame = lazy_frame.cache().with_row_index("Index", None);
        lazy_frame.collect().expect("collect source")
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
        self.settings.kind.hash(state);
        self.settings.ddof.hash(state);
        self.settings.relative.hash(state);
        self.settings.filter.hash(state);
        self.settings.sort.hash(state);
        self.settings.order.hash(state);
        self.settings.group.hash(state);
    }
}

/// Extension methods for [`LazyFrame`]
trait LazyFrameExt {
    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame;

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame;
}

impl LazyFrameExt for LazyFrame {
    fn sort_by_fatty_acids(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort_by_exprs(
            [
                col("Mode"),
                col("FA").fa().c(),
                col("FA").fa().unsaturation(),
                col("FA").fa().indices(),
                col("FA").fa().bounds(),
            ],
            sort_options,
        )
    }

    fn sort_by_time(self, sort_options: SortMultipleOptions) -> LazyFrame {
        self.sort(["Mode"], sort_options.clone()).select([all()
            .sort_by(
                &[
                    col("Equivalent").struct_().field_by_name("ECL"),
                    col("Time")
                        .struct_()
                        .field_by_name("Absolute")
                        .struct_()
                        .field_by_name("Mean"),
                ],
                sort_options,
            )
            .over([col("Mode")])])
    }
}

fn relative_time(settings: &Settings) -> Expr {
    let mut expr = col("TimeMean");
    if let Some(relative) = &settings.relative {
        expr = expr
            / col("TimeMean")
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

fn ecl() -> Expr {
    ternary_expr(
        col("FA").fa().saturated(),
        col("FA").fa().c(),
        ternary_expr(col("FA").fa().saturated(), col("FA").fa().c(), lit(NULL)).forward_fill(None)
            + (col("TimeMean") - saturated(col("TimeMean")).forward_fill(None))
                / (saturated(col("TimeMean")).backward_fill(None)
                    - saturated(col("TimeMean")).forward_fill(None)),
    )
}

fn saturated(expr: Expr) -> Expr {
    ternary_expr(col("FA").fa().saturated(), expr, lit(NULL))
}

// fn slope() -> Expr {
//     ternary_expr(
//         col("FA").fa().saturated(),
//         lit(0),
//         ternary_expr(col("FA").fa().saturated(), col("FA").fa().c(), lit(NULL)).forward_fill(None)
//             + (col("Time").struct_().field_by_name("Mean") - time().forward_fill(None))
//                 / (time().backward_fill(None) - time().forward_fill(None)),
//     )
// }
