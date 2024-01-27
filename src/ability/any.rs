// use crate::prove::Prove;
use std::convert::Infallible;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DelegateAny;

// impl Prove<DelegateAny> for DelegateAny {
//     type Proven = Self;
//
//     fn prove(self, _proof: Self) -> Self::Proven {
//         self
//     }
// }
