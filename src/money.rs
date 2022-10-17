use std::{
    fmt::Display,
    ops::{Add, Deref, Mul, Sub},
};

use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode, Encode, FromRow, Type};

#[derive(Debug, FromRow, Deserialize, Serialize, Clone, Copy)]
pub struct Money(i32);

impl<'r, DB: Database> Decode<'r, DB> for Money
where
    i32: Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = i32::decode(value)?;
        Ok(Money(value))
    }
}

impl<'r, DB: Database> Encode<'r, DB> for Money
where
    i32: Encode<'r, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }

    fn encode(
        self,
        buf: &mut <DB as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.0.encode(buf)
    }

    fn produces(&self) -> Option<DB::TypeInfo> {
        self.0.produces()
    }

    fn size_hint(&self) -> usize {
        self.0.size_hint()
    }
}

impl<DB: Database> Type<DB> for Money
where
    i32: Type<DB>,
{
    fn type_info() -> <DB as Database>::TypeInfo {
        <i32 as Type<DB>>::type_info()
    }

    fn compatible(ty: &<DB as Database>::TypeInfo) -> bool {
        <i32 as Type<DB>>::compatible(ty)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let euros = self.0 / 100;
        let cents = self.0 % 100;
        write!(f, "€ {}.{:02}", euros, cents)
    }
}

impl From<i32> for Money {
    fn from(value: i32) -> Self {
        Money(value)
    }
}

impl From<Money> for i32 {
    fn from(value: Money) -> Self {
        value.0
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Self) -> Self::Output {
        Money(self.0 + rhs.0)
    }
}

impl Mul<i32> for Money {
    type Output = Money;

    fn mul(self, rhs: i32) -> Self::Output {
        Money(self.0 * rhs)
    }
}

impl Mul<Money> for i32 {
    type Output = Money;

    fn mul(self, rhs: Money) -> Self::Output {
        Money(self * rhs.0)
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Self) -> Self::Output {
        Money(self.0 - rhs.0)
    }
}

impl Deref for Money {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<f64> for Money {
    fn from(value: f64) -> Self {
        Money((value * 100.0) as i32)
    }
}
