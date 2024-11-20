use crate::{
    app::panes::settings::{Settings, Sort},
    special::polars::{ExprExt as _, Mass as _, Rcooh},
};
use egui::{
    emath::Float,
    util::cache::{ComputerMut, FrameCache},
};
use polars::{frame::DataFrame, prelude::*};
use std::hash::{Hash, Hasher};
use tracing::error;

/// Filter computed
pub(crate) type Computed = FrameCache<DataFrame, Computer>;

/// Filter computer
#[derive(Default)]
pub(crate) struct Computer;

impl ComputerMut<Key<'_>, DataFrame> for Computer {
    fn compute(&mut self, key: Key<'_>) -> DataFrame {
        let mut data_frame = key.data_frame.clone();
        error!(?data_frame);
        let mut lazy_frame = data_frame.clone().lazy().select([
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
        println!("1: {}", lazy_frame.clone().collect().unwrap());
        // lazy_frame = concat(
        //     [
        //         lazy_frame,
        //         df! {
        //             "Mode" => df! {
        //                 "OnsetTemperature" => [key.settings.interpolation.onset_temperature],
        //                 "TemperatureStep" => [key.settings.interpolation.temperature_step],
        //             }.unwrap().into_struct(PlSmallStr::EMPTY),
        //         }
        //         .unwrap()
        //         .lazy(),
        //     ],
        //     UnionArgs::default(),
        // )
        // .unwrap();
        let other = df! {
            "Mode" => df! {
                "OnsetTemperature" => [key.settings.interpolation.onset_temperature],
                "TemperatureStep" => [key.settings.interpolation.temperature_step],
            }.unwrap().into_struct(PlSmallStr::EMPTY),
            "FA" => df! {
                "Carbons" => &[
                    18,
                ],
                "Bounds" => &[
                    df! { "Index" => [9i8], "Multiplicity" => [2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [-9, -12i8], "Multiplicity" => [2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8), "Multiplicity" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8) }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [9, 12i8], "Multiplicity" => [2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [6, 9, 12i8], "Multiplicity" => [2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8), "Multiplicity" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8) }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [9, 12, 15i8], "Multiplicity" => [2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [11i8], "Multiplicity" => [2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8), "Multiplicity" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8) }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [11, 14i8], "Multiplicity" => [2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [8, 11, 14i8], "Multiplicity" => [2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8), "Multiplicity" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8) }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [11, 14, 17i8], "Multiplicity" => [2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [5, 8, 11, 14i8], "Multiplicity" => [2, 2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [13i8], "Multiplicity" => [2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8), "Multiplicity" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8) }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [13, 16i8], "Multiplicity" => [2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [5, 8, 11, 14, 17i8], "Multiplicity" => [2, 2, 2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8), "Multiplicity" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8) }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [15i8], "Multiplicity" => [2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                    df! { "Index" => [4, 7, 10, 13, 16, 19i8], "Multiplicity" => [2, 2, 2, 2, 2, 2u8] }?.into_struct(PlSmallStr::EMPTY).into_series(),
                ],
                "Label" => &[
                    "Methyl cis-9 oleate [Methyl cis-9-octadecenoate]",
                    "Methyl linolelaidate [Methyl trans,trans-9,12-octadecadienoate]",
                    "Nonadecanoic acid methyl ester",
                    "Methyl linoleate [Methyl 9-cis,12-cis-octadecadienoate]",
                    "Methyl-gamma-linolenate, (6Z,9Z,12Z-octadecatrienoate)",
                    "Methyl arachidate",
                    "Methyl alfa linolenate, Methyl (9Z,12Z,15Z)-octadeca-9,12,15-trienoate",
                    "Methyl cis-11 eicosenoate",
                    "Methyl heneicosanoate",
                    "Methyl cis-11,14 eicosadienoate",
                    "Methyl cis-8,11,14 eicosatrienoate",
                    "Methyl behenate [Methyl docosanoate]",
                    "Methyl cis-11,14,17 eicosatrienoate",
                    "Methyl arachidonate",
                    "Methyl erucate [Methyl cis-13-docosenoate]",
                    "Methyl tricosanoate",
                    "Methyl cis-13,16 docosadienoate",
                    "Methyl cis-5,8,11,14,17 eicosapentaenoate",
                    "Methyl lignocerate [Methyl tetracosanoate]",
                    "Methyl nervonate [Methyl cis-15-tetracosenoate]",
                    "Methyl cis-4,7,10,13,16,19 docosahexaenoate",
                ],
            }?.into_struct(PlSmallStr::EMPTY),
        }
        .unwrap()
        .lazy();
        println!("2: {}", lazy_frame.clone().collect().unwrap());
        // lazy_frame = lazy_frame.join(
        //     df! {
        //         "Mode" => df! {
        //             "OnsetTemperature" => [key.settings.interpolation.onset_temperature],
        //             "TemperatureStep" => [key.settings.interpolation.temperature_step],
        //         }.unwrap().into_struct(PlSmallStr::EMPTY),
        //     }
        //     .unwrap()
        //     .lazy(),
        //     [
        //         col("Mode").struct_().field_by_name("OnsetTemperature"),
        //         col("Mode").struct_().field_by_name("TemperatureStep"),
        //     ],
        //     [
        //         col("Mode").struct_().field_by_name("OnsetTemperature"),
        //         col("Mode").struct_().field_by_name("TemperatureStep"),
        //     ],
        //     JoinArgs::new(JoinType::Left),
        // );
        // lazy_frame.with_collapse_joins(toggle)
        error!(?data_frame);
        lazy_frame = lazy_frame.with_columns([
            col("Time")
                .struct_()
                .with_fields(vec![relative_time().over(["Mode"]).alias("Relative")])
                .unwrap(),
            ecl().over(["Mode"]).alias("ECL"),
            col("FA").fa().ecn().alias("ECN"),
            as_struct(vec![
                col("FA").fa().mass().alias("RCOOH"),
                col("FA").fa().rcoo().mass().alias("RCOO"),
                col("FA").fa().rcooch3().mass().alias("RCOOCH3"),
            ])
            .alias("Mass"),
        ]);
        // if let Some(temperature_step) = key.settings.filter_temperature_step {
        //     lazy_frame = lazy_frame.filter(
        //         col("Mode")
        //             .struct_()
        //             .field_by_name("TemperatureStep")
        //             .eq(lit(temperature_step)),
        //     );
        // }
        data_frame = lazy_frame.with_row_index("Index", None).collect().unwrap();
        error!(?data_frame);
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
        // for value in self.data_frame["FA"].phys_iter() {
        //     value.hash(state);
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
