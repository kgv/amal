use crate::r#const::relative_atomic_mass::{C, H, O};
use itertools::izip;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter, Write},
    iter::zip,
};

/// Extension methods for [`Column`]
pub trait ColumnExt {
    fn fa(&self) -> FattyAcids;
}

impl ColumnExt for Column {
    fn fa(&self) -> FattyAcids {
        FattyAcids::new(self).expect(r#"Expected "FattyAcids" column"#)
    }
}

/// Fatty acids
#[derive(Clone)]
pub struct FattyAcids {
    carbons: Series,
    indices: Series,
    bounds: Series,
    labels: Series,
}

impl FattyAcids {
    pub fn new(column: &Column) -> PolarsResult<Self> {
        let carbons = column.struct_()?.field_by_name("Carbons")?;
        let indices = column.struct_()?.field_by_name("Indices")?;
        let bounds = column.struct_()?.field_by_name("Bounds")?;
        let labels = column.struct_()?.field_by_name("Label")?;
        Ok(Self {
            carbons,
            indices,
            bounds,
            labels,
        })
    }

    pub fn get(&self, index: usize) -> PolarsResult<FattyAcid> {
        let carbons = self.carbons.u8()?.get(index).unwrap();
        let indices = self
            .indices
            .list()?
            .get_as_series(index)
            .unwrap()
            .u8()?
            .to_vec_null_aware()
            .left()
            .unwrap();
        let bounds = self
            .bounds
            .list()?
            .get_as_series(index)
            .unwrap()
            .i8()?
            .to_vec_null_aware()
            .left()
            .unwrap();
        let label = self.labels.str()?.get(index).unwrap().to_owned();
        Ok(FattyAcid {
            carbons,
            indices,
            bounds,
            label,
        })
    }

    pub fn iter(&self) -> PolarsResult<impl Iterator<Item = FattyAcid> + '_> {
        Ok(izip!(
            self.carbons.u8()?,
            self.indices.list()?,
            self.bounds.list()?,
            self.labels.str()?
        )
        .filter_map(|(carbons, indices, bounds, label)| {
            Some(FattyAcid {
                carbons: carbons?,
                indices: indices?.u8().unwrap().to_vec_null_aware().left()?,
                bounds: bounds?.i8().unwrap().to_vec_null_aware().left()?,
                label: label?.to_owned(),
            })
        }))
    }
}

/// Fatty acid
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct FattyAcid {
    pub carbons: u8,
    pub indices: Vec<u8>,
    pub bounds: Vec<i8>,
    pub label: String,
}

impl FattyAcid {
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Display for FattyAcid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.carbons, self.indices.len())?;
        let mut indices = zip(&self.indices, &self.bounds);
        if let Some((index, &bound)) = indices.next() {
            write!(f, "-{index}")?;
            if bound < 0 {
                f.write_char('t')?;
            } else {
                f.write_char('c')?;
            }
        }
        for (index, &bound) in indices {
            write!(f, ",{index}")?;
            if bound < 0 {
                f.write_char('t')?;
            } else {
                f.write_char('c')?;
            }
        }
        Ok(())
    }
}
