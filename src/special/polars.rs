use std::sync::LazyLock;

use crate::r#const::relative_atomic_mass::{C, H, O};
use polars::prelude::*;

// pub static FATTY_ACIDS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
//     Schema::from_iter([
//         Field::new("Carbons".into(), DataType::UInt8),
//         Field::new(
//             "Bounds".into(),
//             DataType::List(Box::new(DataType::Struct(vec![
//                 Field::new("Index".into(), DataType::Int8),
//                 Field::new("Multiplicity".into(), DataType::UInt8),
//             ]))),
//         ),
//         Field::new("Label".into(), DataType::String),
//     ])
// });

/// Extension methods for [`Expr`]
pub trait ExprExt {
    fn fa(self) -> Rcooh;
}

impl ExprExt for Expr {
    fn fa(self) -> Rcooh {
        Rcooh(self)
    }
}

/// Mass
pub trait Mass {
    /// Carbons count
    fn c(&self) -> Expr;

    /// Hydrogens count
    fn h(&self) -> Expr;

    fn mass(&self) -> Expr {
        self.c() * lit(C) + self.h() * lit(H) + lit(2) * lit(O)
    }
}

/// Fatty acids [`Expr`]
#[derive(Clone)]
pub struct Rcooh(Expr);

impl Rcooh {
    pub fn rcoo(self) -> Rcoo {
        Rcoo(self)
    }

    /// Methyl ester
    pub fn rcooch3(self) -> Rcooch3 {
        Rcooch3(self)
    }
}

impl Rcooh {
    /// Bounds count
    pub fn b(&self) -> Expr {
        self.0
            .clone()
            .struct_()
            .field_by_name("Bounds")
            .list()
            .len()
    }

    /// Fatty acid ECN (Equivalent carbon number)
    ///
    /// `ECN = CN - 2DB`
    pub fn ecn(self) -> Expr {
        self.c() - lit(2) * self.unsaturation()
    }

    pub fn indices(&self) -> Expr {
        self.0.clone().struct_().field_by_name("Indices")
    }

    pub fn saturated(&self) -> Expr {
        self.unsaturation().eq(lit(0))
    }

    pub fn unsaturated(&self) -> Expr {
        self.saturated().not()
    }

    pub fn unsaturation(&self) -> Expr {
        self.0
            .clone()
            .struct_()
            .field_by_name("Bounds")
            .list()
            .eval(col("") - lit(1), true)
            .list()
            .sum()
    }
}

impl Mass for Rcooh {
    fn c(&self) -> Expr {
        self.0.clone().struct_().field_by_name("Carbons")
    }

    fn h(&self) -> Expr {
        lit(2) * self.c() - lit(2) * self.unsaturation()
    }
}

impl From<Rcooh> for Expr {
    fn from(value: Rcooh) -> Self {
        value.0
    }
}

/// Fatty acid methyl ester [`Expr`]
#[derive(Clone)]
pub struct Rcooch3(Rcooh);

impl Mass for Rcooch3 {
    fn c(&self) -> Expr {
        self.0.c() + lit(1)
    }

    fn h(&self) -> Expr {
        self.0.h() + lit(2)
    }
}

/// Fatty acid RCOO- [`Expr`]
#[derive(Clone)]
pub struct Rcoo(Rcooh);

impl Mass for Rcoo {
    fn c(&self) -> Expr {
        self.0.c()
    }

    fn h(&self) -> Expr {
        self.0.h() - lit(1)
    }
}
