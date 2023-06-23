//! While this is mostly a CLI binary around bustools, some functionality might be useful as a library,
//! in particular the [count], [count2] and [butterfly] modules.
//! 
#![deny(missing_docs)]
pub mod count;
pub mod count2;
pub mod butterfly;
pub mod inspect;
pub mod countmatrix;
pub mod sort;
pub mod correct;
pub mod busmerger;
