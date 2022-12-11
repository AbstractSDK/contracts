use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Copy,
)]
pub struct UniquePoolId(u64);

impl UniquePoolId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Into<u64> for UniquePoolId {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<u64> for UniquePoolId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

// impl Display for UniquePoolId {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }
//
// impl<'a> PrimaryKey<'a> for UniquePoolId {
//     type Prefix = ();
//     type SubPrefix = ();
//     type Suffix = Self;
//     type SuperSuffix = Self;
//
//     fn key(&self) -> Vec<cw_storage_plus::Key> {
//         vec![Key::Val64(self.to_cw_bytes())]
//     }
// }
//
// impl<'a> Prefixer<'a> for UniquePoolId {
//     fn prefix(&self) -> Vec<cw_storage_plus::Key> {
//         vec![cw_storage_plus::Key::Val64(self.to_cw_bytes())]
//     }
// }
//
// impl KeyDeserialize for UniquePoolId {
//     type Output = Self;
//     #[inline(always)]
//     fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
//         Ok(Self::from_cw_bytes(value.as_slice().try_into()
//             .map_err(|err: TryFromSliceError| StdError::generic_err(err.to_string()))?))
//     }
// }
//
// // macro_rules! cw_uint_keys {
// //     (for $($t:ty),+) => {
// //         $(impl IntKey for $t {
// //             type Buf = [u8; mem::size_of::<$t>()];
// //
// //             #[inline]
// //             fn to_cw_bytes(&self) -> Self::Buf {
// //                 self.to_be_bytes()
// //             }
// //
// //             #[inline]
// //             fn from_cw_bytes(bytes: Self::Buf) -> Self {
// //                 Self::from_be_bytes(bytes)
// //             }
// //         })*
// //     }
// // }
//
// impl IntKey for UniquePoolId {
//     type Buf = [u8; mem::size_of::<u64>()];
//
//     #[inline]
//     fn to_cw_bytes(&self) -> Self::Buf {
//         self.0.to_be_bytes()
//     }
//
//     #[inline]
//     fn from_cw_bytes(bytes: Self::Buf) -> Self {
//         Self(u64::from_be_bytes(bytes))
//     }
// }
//
// // macro_rules! cw_int_keys {
// //     (for $($t:ty, $ut:ty),+) => {
// //         $(impl IntKey for $t {
// //             type Buf = [u8; mem::size_of::<$t>()];
// //
// //             #[inline]
// //             fn to_cw_bytes(&self) -> Self::Buf {
// //                 (*self as $ut ^ <$t>::MIN as $ut).to_be_bytes()
// //             }
// //
// //             #[inline]
// //             fn from_cw_bytes(bytes: Self::Buf) -> Self {
// //                 (Self::from_be_bytes(bytes) as $ut ^ <$t>::MIN as $ut) as _
// //             }
// //         })*
// //     }
// // }
// //
// // impl IntKey for i64 {
// //     type Buf = [u8; mem::size_of::<i64>()];
// //
// //     #[inline]
// //     fn to_cw_bytes(&self) -> Self::Buf {
// //         (*self as u64 ^ i64::MIN as u64).to_be_bytes()
// //     }
// //
// //     #[inline]
// //     fn from_cw_bytes(bytes: Self::Buf) -> Self {
// //         (i64::from_be_bytes(bytes) as u64 ^ i64::MIN as u64) as _
// //     }
// // }
