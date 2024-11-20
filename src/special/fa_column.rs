use crate::r#const::relative_atomic_mass::{C, H, O};
use polars::prelude::*;
use std::fmt::{self, Display, Formatter};

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
    labels: Series,
}

impl FattyAcids {
    pub fn new(column: &Column) -> PolarsResult<Self> {
        let carbons = column.struct_()?.field_by_name("Carbons")?;
        let indices = column.struct_()?.field_by_name("Indices")?;
        let labels = column.struct_()?.field_by_name("Label")?;
        Ok(Self {
            carbons,
            indices,
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
        let label = self.labels.str()?.get(index).unwrap().to_owned();
        Ok(FattyAcid {
            carbons,
            indices,
            label,
        })
    }
}

/// Fatty acid
#[derive(Clone)]
pub struct FattyAcid {
    carbons: u8,
    indices: Vec<u8>,
    label: String,
}

impl FattyAcid {
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Display for FattyAcid {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.carbons, self.indices.len())?;
        let mut indices = self.indices.iter();
        if let Some(index) = indices.next() {
            write!(f, "-{index}")?;
        }
        for index in indices {
            write!(f, ",{index}")?;
        }
        Ok(())
    }
}
