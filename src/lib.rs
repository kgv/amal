#![feature(decl_macro)]
#![feature(try_trait_v2)]

pub use app::App;

mod app;
mod r#const;
mod special;
mod utils;

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use polars::prelude::*;
    use ron::{extensions::Extensions, ser::PrettyConfig};
    use special::expressions::fatty_acid::{ExprExt, FattyAcid as _};
    use std::{fs::write, iter::empty, path::Path};
    use walkdir::WalkDir;

    fn format(path: &Path) -> Result<()> {
        let source = std::fs::read_to_string(path)?;
        let data_frame: DataFrame = ron::de::from_str(&source)?;
        let formated = ron::ser::to_string_pretty(
            &data_frame,
            PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
        )?;
        if source != formated {
            // std::fs::copy(path, format!("{path}.bk"))?;
            std::fs::write(path, formated)?;
        }
        Ok(())
    }

    #[test]
    fn test_format() -> Result<()> {
        for entry in WalkDir::new("input/data/") {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                println!("{}", entry.path().display());
                format(entry.path())?;
            }
        }
        Ok(())
    }

    // Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])
    #[test]
    fn test() -> Result<()> {
        let data_frame = df! {
            "FA" => df! {
                "Carbons" => &[
                    8u8,
                    10,
                    11,
                    12,
                    13,
                    14,
                    14,
                    15,
                    15,
                    16,
                    16,
                    17,
                    17,
                    18,
                    18,
                    18,
                    18,
                    19,
                    18,
                    18,
                    20,
                    18,
                    20,
                    21,
                    20,
                    20,
                    22,
                    20,
                    20,
                    22,
                    23,
                    22,
                    20,
                    24,
                    24,
                    22,
                ],
                "Indices" => &[
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([9u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([10u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([9u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([10u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([9u8]),
                    Series::from_iter([9u8]),
                    Series::from_iter([9, 12u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([9, 12u8]),
                    Series::from_iter([6, 9, 12u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([9, 12, 15u8]),
                    Series::from_iter([11u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([11, 14u8]),
                    Series::from_iter([8, 11, 14u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([11, 14, 17u8]),
                    Series::from_iter([5, 8, 11, 14u8]),
                    Series::from_iter([13u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([13, 16u8]),
                    Series::from_iter([5, 8, 11, 14, 17u8]),
                    Series::from_iter(empty::<u8>()),
                    Series::from_iter([15u8]),
                    Series::from_iter([4, 7, 10, 13, 16, 19u8]),
                ],
                "Bounds" => &[
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([-2i8]),
                    Series::from_iter([2i8]),
                    Series::from_iter([-2, -2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2, 2i8]),
                    Series::from_iter([2, 2, 2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2, 2, 2i8]),
                    Series::from_iter([2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2, 2i8]),
                    Series::from_iter([2, 2, 2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2, 2, 2i8]),
                    Series::from_iter([2, 2, 2, 2i8]),
                    Series::from_iter([2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2, 2i8]),
                    Series::from_iter([2, 2, 2, 2, 2i8]),
                    Series::from_iter(empty::<i8>()),
                    Series::from_iter([2i8]),
                    Series::from_iter([2, 2, 2, 2, 2, 2i8]),
                ],
                "Label" => &[
                    "Methyl octanoate",
                    "Methyl decanoate",
                    "Methyl undecanoate",
                    "Methyl dodecanoate",
                    "Methyl tridecanoate",
                    "Methyl myristate [Methyl tetradecanoate]",
                    "Methyl myristoleate [Methyl cis-9-tetradecenoate]",
                    "Methyl pentadecanoate",
                    "Methyl cis-10 pentadecenoate",
                    "Methyl palmitate",
                    "Methyl palmitoleate-Z9",
                    "Methyl heptadecanoate",
                    "Methyl cis-10 heptadecenoate",
                    "Methyl stearate",
                    "Methyl trans-9 eladiate [Methyl trans-9-octadecenoate]",
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
            "OnsetTemperature" => [
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
                70f64,
            ],
            "TemperatureStep" => [
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
                1f64,
            ],
            "Time" => [
                Series::from_iter([Some(17.914), Some(17.840), Some(17.821)]),
                Series::from_iter([Some(31.002), Some(30.910), Some(30.888)]),
                Series::from_iter([Some(38.693), Some(38.594), Some(38.567)]),
                Series::from_iter([Some(46.620), Some(46.529), Some(46.497)]),
                Series::from_iter([Some(54.538), Some(54.439), Some(54.408)]),
                Series::from_iter([Some(62.300), Some(62.180), Some(62.144)]),
                Series::from_iter([Some(66.760), Some(66.648), Some(66.621)]),
                Series::from_iter([Some(69.798), Some(69.669), Some(69.651)]),
                Series::from_iter([Some(74.146), Some(74.043), Some(74.016)]),
                Series::from_iter([Some(77.065), Some(76.912), Some(76.885)]),
                Series::from_iter([Some(80.313), Some(80.185), Some(80.158)]),
                Series::from_iter([Some(83.957), Some(83.816), Some(83.793)]),
                Series::from_iter([Some(87.118), Some(87.015), Some(86.988)]),
                Series::from_iter([Some(90.684), Some(90.514), Some(90.479)]),
                Series::from_iter([Some(92.386), Some(92.241), Some(92.214)]),
                Series::from_iter([Some(93.165), Some(93.024), Some(92.993)]),
                Series::from_iter([Some(95.993), Some(95.860), Some(95.833)]),
                Series::from_iter([None, Some(96.895), Some(96.864)]),
                Series::from_iter([Some(97.782), Some(97.645), Some(97.622)]),
                Series::from_iter([Some(100.919), Some(100.803), Some(100.780)]),
                Series::from_iter([Some(103.243), Some(103.061), Some(103.030)]),
                Series::from_iter([Some(103.252), Some(103.099), Some(103.059)]),
                Series::from_iter([Some(105.515), Some(105.366), Some(105.339)]),
                Series::from_iter([Some(109.109), Some(108.948), Some(108.912)]),
                Series::from_iter([Some(109.896), Some(109.768), Some(109.737)]),
                Series::from_iter([Some(112.765), Some(112.637), Some(112.618)]),
                Series::from_iter([Some(114.900), Some(114.706), Some(114.675)]),
                Series::from_iter([Some(114.979), Some(114.854), Some(114.827)]),
                Series::from_iter([Some(114.814), Some(114.702), Some(114.654)]),
                Series::from_iter([Some(117.044), Some(116.887), Some(116.843)]),
                Series::from_iter([Some(120.329), Some(120.151), Some(120.112)]),
                Series::from_iter([Some(121.153), Some(121.009), Some(120.977)]),
                Series::from_iter([Some(120.036), Some(119.929), Some(119.906)]),
                Series::from_iter([Some(125.749), Some(125.539), Some(125.507)]),
                Series::from_iter([Some(127.790), Some(127.616), Some(127.581)]),
                Series::from_iter([Some(133.062), Some(132.954), Some(132.935)]),
            ],
        }?;
        println!("data_frame: {data_frame}");
        println!(
            "data_frame: {}",
            data_frame
                .clone()
                .lazy()
                .with_columns([
                    col("FA").fa().ecn().alias("ECN"),
                    col("FA").fa().mass().alias("Mass"),
                    col("FA").fa().rcooch3().mass().alias("MethylEsterMass"),
                    col("FA").fa().rcoo().mass().alias("RCOOMass"),
                    // col("Time").list().mean().alias("Mean"),
                    // col("Time").list().std(0).alias("StandardDeviation"),
                    // col("FA").fa().unsaturation().alias("Unsaturation"),
                ])
                .collect()?
        );
        // println!("data_frame: {}", data_frame.unnest(["FA"]).unwrap());
        let contents = ron::ser::to_string_pretty(
            &data_frame,
            PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
        )?;
        write("df.amal.ron", contents)?;
        Ok(())
    }
}
