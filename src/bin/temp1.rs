use anyhow::Result;
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use std::{fs::write, iter::empty, path::Path};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let content = include_str!("../../df.ron");
    let data_frame: DataFrame = ron::de::from_str(content).unwrap();
    println!("data_frame: {data_frame}");
    // println!(
    //     "data_frame: {}",
    //     data_frame
    //         .clone()
    //         .lazy()
    //         .with_columns([
    //             col("FA").fa().ecn().alias("ECN"),
    //             col("FA").fa().mass().alias("Mass"),
    //             col("FA").fa().rcooch3().mass().alias("MethylEsterMass"),
    //             col("FA").fa().rcoo().mass().alias("RCOOMass"),
    //             // col("Time").list().mean().alias("Mean"),
    //             // col("Time").list().std(0).alias("StandardDeviation"),
    //             // col("FA").fa().unsaturation().alias("Unsaturation"),
    //         ])
    //         .collect()?
    // );
    // println!("data_frame: {}", data_frame.unnest(["FA"]).unwrap());
    let contents = ron::ser::to_string_pretty(
        &data_frame,
        PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
    )?;
    write("df.out.ron", contents)?;
    Ok(())
}
