// SPDX-License-Identifier: CC0-1.0

//! # Rust Bitcoin Internal
//!
//! This crate is only meant to be used internally by crates in the
//! [rust-bitcoin](https://github.com/rust-bitcoin) ecosystem.
//!

#![no_std]
// Experimental features we need.

// Coding conventions
#![warn(missing_docs)]
// Exclude clippy lints we don't think are valuable
#![allow(clippy::needless_question_mark)] // https://github.com/rust-bitcoin/rust-bitcoin/pull/2134

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// This crate is `#![no_std]`, so on edition 2018 Kani cannot find its runtime on its own.
// Import it explicitly so `cargo kani --only-codegen` can process the crate.
#[cfg(kani)]
extern crate kani;

pub mod error;
pub mod macros;
mod parse;
pub mod serde;
