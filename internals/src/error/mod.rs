// SPDX-License-Identifier: CC0-1.0

//! # Error
//!
//! Error handling macros and helpers.

#[cfg(not(creusot))]
pub mod input_string;
mod parse_error;

#[cfg(not(creusot))]
pub use input_string::InputString;

#[doc(hidden)]
/// Helper trait for checking whether something implements [`std::error::Error`].
///
/// We use helper trait instead of function to make the compiler handle auto dereferencing.
#[cfg(feature = "std")]
pub trait ErrorExt: std::error::Error + 'static {
    /// Does nothing, just call it
    #[inline(always)]
    fn check_is_actually_error_source(&self) {}
}

#[doc(hidden)]
#[cfg(feature = "std")]
impl<T: std::error::Error + 'static> ErrorExt for T {}

/// Implements the `Error` trait for an error type.
///
/// On Rust 1.81 and later this implements `core::error::Error`, with or without the `std` feature.
/// On older compilers it implements `std::error::Error`, and only if the `std` feature of the
/// calling crate is enabled.
///
/// Inside the macro `Error` names the trait being implemented. Any item can go in, so a helper
/// that returns `dyn Error` is gated the same way as the impl that calls it.
///
/// # Examples
///
/// ```
/// # use core::fmt;
/// #[derive(Debug)]
/// pub struct InnerError;
/// # impl fmt::Display for InnerError {
/// #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("inner") }
/// # }
///
/// #[derive(Debug)]
/// pub struct OuterError(InnerError);
/// # impl fmt::Display for OuterError {
/// #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("outer") }
/// # }
///
/// bitcoin_internals::impl_std_error! {
///     impl Error for InnerError {}
/// }
///
/// bitcoin_internals::impl_std_error! {
///     impl Error for OuterError {
///         fn source(&self) -> Option<&(dyn Error + 'static)> { Some(&self.0) }
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_std_error {
    ($($item:item)*) => {
        $crate::rust_version::rust_version! {
            if >= 1.81 {
                const _: () = {
                    use core::error::Error;

                    $($item)*
                };
            } else {
                #[cfg(feature = "std")]
                const _: () = {
                    use std::error::Error;

                    $($item)*
                };
            }
        }
    }
}

/// Formats error.
///
/// If `std` feature is OFF appends error source (delimited by `: `). We do this because
/// `e.source()` is only available in std builds, without this macro the error source is lost for
/// no-std builds.
///
/// Note: will not compile on std if $source doesn't implement [`std::error::Error`]. This is
/// intentional because if it doesn't the macro MUST NOT be used. However this check is very rarely
/// imprecise so if you know what you're doing you can use the unchecked version.
#[macro_export]
macro_rules! write_err {
    ($writer:expr, $string:literal $(, $args:expr)*; $source:expr) => {
        {
            #[cfg(feature = "std")]
            {
                use $crate::error::ErrorExt;

                // Enforces that $source is actually an error.
                //
                // Occasionally people accidentally use write_err! with non-error and it's quite
                // hard to detect otherwise so this gives them compilation error.
                // Also avoids lints saying $source is unused.
                ($source).check_is_actually_error_source();

                write!($writer, $string $(, $args)*)
            }
            #[cfg(not(feature = "std"))]
            {
                write!($writer, concat!($string, ": {}") $(, $args)*, $source)
            }
        }
    }
}

/// Formats error.
///
/// Unlike [`write_err`] doesn't statically check that `$source` implements `std::error::Error`.
/// This should be used only in rare generic situations. Specifically, in case of a generic
/// `MyError<Error>`, this macro should be used when formatting an error message for the generic
/// type `Error`.
///
/// If `std` feature is OFF appends error source (delimited by `: `). We do this because
/// `e.source()` is only available in std builds, without this macro the error source is lost for
/// no-std builds.
#[macro_export]
macro_rules! write_err_unchecked_source {
    ($writer:expr, $string:literal $(, $args:expr)*; $source:expr) => {
        {
            #[cfg(feature = "std")]
            {
                let _ = &$source;   // Prevents clippy warnings.

                write!($writer, $string $(, $args)*)
            }
            #[cfg(not(feature = "std"))]
            {
                write!($writer, concat!($string, ": {}") $(, $args)*, $source)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    #[derive(Debug)]
    struct Leaf;

    impl fmt::Display for Leaf {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("leaf") }
    }

    impl_std_error! {
        impl Error for Leaf {}
    }

    #[derive(Debug)]
    struct Wrapper(Leaf);

    impl fmt::Display for Wrapper {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("wrapper") }
    }

    impl_std_error! {
        impl Error for Wrapper {
            fn source(&self) -> Option<&(dyn Error + 'static)> { Some(&self.0) }
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn impl_std_error_with_std() {
        use std::error::Error as _;

        assert!(Leaf.source().is_none());
        assert!(Wrapper(Leaf).source().unwrap().is::<Leaf>());
    }

    crate::rust_version::rust_version! {
        if >= 1.81 {
            #[test]
            fn impl_std_error_without_std() {
                use core::error::Error as _;

                assert!(Leaf.source().is_none());
                assert!(Wrapper(Leaf).source().unwrap().is::<Leaf>());
            }
        }
    }
}
