use anyhow::Result;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::write,
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) data_frame: DataFrame,
}

impl Data {
    pub(crate) fn save(&self, path: impl AsRef<Path>, format: Format) -> Result<()> {
        let data_frame = self.data_frame.select(["RetentionTime", "Masspectrum"])?;
        match format {
            Format::Bin => {
                let contents = bincode::serialize(&data_frame)?;
                write(path, contents)?;
            }
            Format::Ron => {
                let contents = ron::ser::to_string_pretty(&data_frame, Default::default())?;
                write(path, contents)?;
            }
        }
        Ok(())
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.data_frame, f)
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            data_frame: DataFrame::empty_with_schema(&Schema::from_iter([
                Field::new("RetentionTime".into(), DataType::String),
                Field::new(
                    "Masspectrum".into(),
                    DataType::List(Box::new(DataType::Struct(vec![
                        Field::new("MassToCharge".into(), DataType::String),
                        Field::new("Signal".into(), DataType::String),
                    ]))),
                ),
            ])),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Format {
    #[default]
    Bin,
    Ron,
}
