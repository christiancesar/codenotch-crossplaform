//! Every file Codenotch writes goes through here: one directory, atomic writes, and files that
//! cannot be read are moved aside instead of overwritten.

pub mod atomic;
pub mod paths;
pub mod versioned;
