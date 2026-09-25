// SPDX-License-Identifier: CC0-1.0

//! Test the API surface (not functionality) of `bitcoin-units`.
//!
//! See [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) and the [rust-bitcoin policies](../../docs/policy.md).

// intentionally put all features so that gates don't kill readability.
#![cfg(feature = "encoding")]
#![cfg(feature = "serde")]
#![cfg(feature = "arbitrary")]
#![allow(dead_code)]
#![allow(unused_imports)]

// These imports test "typical" usage by user code.
use bitcoin_units::locktime::{absolute, relative}; // Typical usage is `absolute::LockTime`.
use bitcoin_units::{
    amount, block, fee_rate, locktime, parse_int, pow, result, sequence, time, weight, Amount,
    BlockHeight, BlockHeightInterval, BlockMtp, BlockMtpInterval, BlockTime, FeeRate, NumOpResult,
    Sequence, SignedAmount, Weight,
};

include!("../../include/api_test_tooling.rs");

/// Groups of units public types for testing semantics, groups overlap.
macro_rules! units {
    (@apply $types:tt, assert_implements, $traits:tt, except $except:tt) => {
        units!(@except $types, true, $traits, $except)
    };
    (@apply $types:tt, assert_does_not_implement, $traits:tt, except $except:tt) => {
        units!(@except $types, false, $traits, $except)
    };
    (@apply [$($ty:ty),+ $(,)?], assert_implements, $traits:tt) => {
        $(assert_trait_impls!($ty, $traits, true);)+
    };
    (@apply [$($ty:ty),+ $(,)?], assert_does_not_implement, $traits:tt) => {
        $(assert_trait_impls!($ty, $traits, false);)+
    };
    (@except [$($ty:ty),+ $(,)?], $want:expr, $traits:tt, [$($ex:ty),+ $(,)?]) => {{
        // `stringify!` spacing differs between call sites, compare with whitespace removed.
        let same = |a: &str, b: &str| a.replace(' ', "") == b.replace(' ', "");
        let members = [$(stringify!($ty)),+];
        let exceptions = [$(stringify!($ex)),+];
        for ex in exceptions {
            assert!(members.iter().any(|m| same(m, ex)), "exception {} is not in the group", ex);
        }
        $(assert_trait_impls!(
            $ty,
            $traits,
            $want != exceptions.iter().any(|ex| same(ex, stringify!($ty)))
        );)+
    }};
    // An explicit list of types.
    ([$($ty:ty),+ $(,)?], $($rest:tt)*) => {
        units!(@apply [$($ty),+], $($rest)*)
    };
    // Every public type. Other groups are subsets of this one.
    (all, $($rest:tt)*) => {
        units!(@apply [
            amount::Amount,
            amount::AmountDecoder,
            amount::AmountDecoderError,
            amount::AmountEncoder<'static>,
            amount::BadPositionError,
            amount::Denomination,
            amount::Display,
            amount::InvalidCharacterError,
            amount::MissingDenominationError,
            amount::MissingDigitsError,
            amount::OutOfRangeError,
            amount::ParseAmountError,
            amount::ParseDenominationError,
            amount::ParseError,
            amount::PossiblyConfusingDenominationError,
            amount::SignedAmount,
            amount::TooPreciseError,
            amount::UnknownDenominationError,
            block::BlockHeight,
            block::BlockHeightDecoder,
            block::BlockHeightDecoderError,
            block::BlockHeightEncoder<'static>,
            block::BlockHeightInterval,
            block::BlockMtp,
            block::BlockMtpInterval,
            block::TooBigForRelativeHeightError,
            fee_rate::FeeRate,
            fee_rate::serde::OverflowError,
            locktime::absolute::ConversionError,
            locktime::absolute::Height,
            locktime::absolute::IncompatibleHeightError,
            locktime::absolute::IncompatibleTimeError,
            locktime::absolute::LockTime,
            locktime::absolute::LockTimeDecoder,
            locktime::absolute::LockTimeDecoderError,
            locktime::absolute::LockTimeEncoder<'static>,
            locktime::absolute::MedianTimePast,
            locktime::absolute::ParseHeightError,
            locktime::absolute::ParseTimeError,
            locktime::relative::DisabledLockTimeError,
            locktime::relative::IncompatibleHeightError,
            locktime::relative::IncompatibleTimeError,
            locktime::relative::InvalidHeightError,
            locktime::relative::InvalidTimeError,
            locktime::relative::IsSatisfiedByError,
            locktime::relative::IsSatisfiedByHeightError,
            locktime::relative::IsSatisfiedByTimeError,
            locktime::relative::LockTime,
            locktime::relative::NumberOf512Seconds,
            locktime::relative::NumberOfBlocks,
            locktime::relative::TimeOverflowError,
            parse_int::ParseIntError,
            parse_int::PrefixedHexError,
            parse_int::UnprefixedHexError,
            pow::CompactTarget,
            pow::CompactTargetDecoder,
            pow::CompactTargetDecoderError,
            pow::CompactTargetEncoder<'static>,
            pow::ParseTargetError,
            pow::ParseWorkError,
            pow::Target,
            pow::Work,
            result::MathOp,
            result::NumOpError,
            result::NumOpResult<Amount>,
            sequence::Sequence,
            sequence::SequenceDecoder,
            sequence::SequenceDecoderError,
            sequence::SequenceEncoder<'static>,
            time::BlockTime,
            time::BlockTimeDecoder,
            time::BlockTimeDecoderError,
            time::BlockTimeEncoder<'static>,
            weight::Weight,
        ], $($rest)*)
    };
    // Every public type that is not an error, encoder or decoder.
    (public_types, $($rest:tt)*) => {
        units!(@apply [
            amount::Amount,
            result::MathOp,
            block::BlockHeight,
            block::BlockHeightInterval,
            block::BlockMtp,
            block::BlockMtpInterval,
            time::BlockTime,
            pow::CompactTarget,
            amount::Denomination,
            amount::Display,
            fee_rate::FeeRate,
            locktime::absolute::Height,
            locktime::absolute::LockTime,
            locktime::relative::LockTime,
            locktime::absolute::MedianTimePast,
            locktime::relative::NumberOf512Seconds,
            locktime::relative::NumberOfBlocks,
            result::NumOpResult<Amount>,
            sequence::Sequence,
            amount::SignedAmount,
            pow::Target,
            weight::Weight,
            pow::Work,
        ], $($rest)*)
    };
    // Every error type, feature gated ones included.
    (errors, $($rest:tt)*) => {
        units!(@apply [
            fee_rate::serde::OverflowError,
            locktime::absolute::ConversionError,
            parse_int::ParseIntError,
            result::NumOpError,
            amount::MissingDenominationError,
            amount::ParseDenominationError,
            amount::PossiblyConfusingDenominationError,
            amount::UnknownDenominationError,
            amount::BadPositionError,
            amount::MissingDigitsError,
            locktime::absolute::LockTimeDecoderError,
            locktime::relative::TimeOverflowError,
            amount::OutOfRangeError,
            amount::ParseAmountError,
            amount::ParseError,
            locktime::absolute::ParseHeightError,
            pow::ParseTargetError,
            locktime::absolute::ParseTimeError,
            pow::ParseWorkError,
            parse_int::PrefixedHexError,
            sequence::SequenceDecoderError,
            amount::TooPreciseError,
            parse_int::UnprefixedHexError,
            amount::AmountDecoderError,
            block::TooBigForRelativeHeightError,
            block::BlockHeightDecoderError,
            time::BlockTimeDecoderError,
            pow::CompactTargetDecoderError,
            locktime::relative::DisabledLockTimeError,
            locktime::absolute::IncompatibleHeightError,
            locktime::relative::IncompatibleHeightError,
            locktime::absolute::IncompatibleTimeError,
            locktime::relative::IncompatibleTimeError,
            amount::InvalidCharacterError,
            locktime::relative::InvalidHeightError,
            locktime::relative::InvalidTimeError,
            locktime::relative::IsSatisfiedByError,
            locktime::relative::IsSatisfiedByHeightError,
            locktime::relative::IsSatisfiedByTimeError,
        ], $($rest)*)
    };
    // Every encoder and decoder.
    (codecs, $($rest:tt)*) => {
        units!(@apply [
            amount::AmountDecoder,
            amount::AmountEncoder<'static>,
            time::BlockTimeDecoder,
            time::BlockTimeEncoder<'static>,
            pow::CompactTargetDecoder,
            pow::CompactTargetEncoder<'static>,
            block::BlockHeightDecoder,
            block::BlockHeightEncoder<'static>,
            locktime::absolute::LockTimeDecoder,
            locktime::absolute::LockTimeEncoder<'static>,
            sequence::SequenceDecoder,
            sequence::SequenceEncoder<'static>,
        ], $($rest)*)
    };
    // All public non-error enums.
    (enums, $($rest:tt)*) => {
        units!(@apply [
            amount::Denomination,
            locktime::absolute::LockTime,
            locktime::relative::LockTime,
            result::MathOp,
            result::NumOpResult<Amount>,
        ], $($rest)*)
    };
    // The value types. `amount::Display`, encoders and decoders have groups of their own.
    (structs, $($rest:tt)*) => {
        units!(@apply [
            amount::Amount,
            amount::SignedAmount,
            block::BlockHeight,
            block::BlockHeightInterval,
            block::BlockMtp,
            block::BlockMtpInterval,
            fee_rate::FeeRate,
            locktime::absolute::Height,
            locktime::absolute::MedianTimePast,
            locktime::relative::NumberOf512Seconds,
            locktime::relative::NumberOfBlocks,
            pow::CompactTarget,
            pow::Target,
            pow::Work,
            sequence::Sequence,
            time::BlockTime,
            weight::Weight,
        ], $($rest)*)
    };
    // All public decoder types.
    (decoders, $($rest:tt)*) => {
        units!(@apply [
            amount::AmountDecoder,
            block::BlockHeightDecoder,
            locktime::absolute::LockTimeDecoder,
            pow::CompactTargetDecoder,
            sequence::SequenceDecoder,
            time::BlockTimeDecoder,
        ], $($rest)*)
    };
    // All public encoder types. The lifetime is a `PhantomData` marker, so `'static` probes work.
    (encoders, $($rest:tt)*) => {
        units!(@apply [
            amount::AmountEncoder<'static>,
            block::BlockHeightEncoder<'static>,
            locktime::absolute::LockTimeEncoder<'static>,
            pow::CompactTargetEncoder<'static>,
            sequence::SequenceEncoder<'static>,
            time::BlockTimeEncoder<'static>,
        ], $($rest)*)
    };
}

#[test]
fn clone_trait() {
    // C-COMMON-TRAITS: Every public type implements `Clone`.
    // POLICY: Errors derive `Clone`, value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: Copy
    // REQUIRED BY DEPENDENCY: `LockTime` derives `Clone` over its heights and times.
    // REQUIRED BY DEPENDENCY: Errors and `NumOpResult` derive `Clone` over inner errors.
    units!(all, assert_implements, [Clone]);
}

#[test]
fn copy_trait() {
    // C-COMMON-TRAITS: Every value type and enum is `Copy`.
    // POLICY: Value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: `LockTime` derives `Copy` over its heights and times.
    units!(structs, assert_implements, [Copy]);
    units!(enums, assert_implements, [Copy]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // A formatting adapter built by `Amount::display_in`, it is printed and dropped.
    units!([amount::Display], assert_does_not_implement, [Copy]);

    // POLICY: Errors do not derive `Copy` unless they have to.
    units!(
        errors,
        assert_does_not_implement,
        [Copy],
        except [
            // REQUIRED BY DEPENDENCY: `NumOpResult` is `Copy` and holds it.
            result::NumOpError,
        ]
    );

    // FORBIDDEN BY DEPENDENCY: consensus_encoding codecs derive only `Debug, Clone`.
    units!(codecs, assert_does_not_implement, [Copy]);
}

#[test]
fn debug_trait() {
    // C-DEBUG, C-GOOD-ERR: Every public type implements `Debug`.
    // POLICY: Errors derive `Debug`, value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: std::error::Error
    // REQUIRED BY DEPENDENCY: `relative::LockTime` derives `Debug` over its heights and times.
    // REQUIRED BY DEPENDENCY: Errors and `NumOpResult` derive `Debug` over inner errors.
    units!(all, assert_implements, [Debug]);
}

#[test]
fn default_trait() {
    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // A height, timestamp, target, sequence, weight or fee rate has no value that is
    // more default than another.
    units!(
        structs,
        assert_does_not_implement,
        [Default],
        except [
            // TODO: satsfy says IMPLEMENT (satisfied already)
            // Zero is an ok default for an amount.
            amount::Amount,
            amount::SignedAmount,
            // TODO: satsfy says IMPLEMENT (satisfied already)
            // Zero is an ok default for an interval.
            block::BlockHeightInterval,
            block::BlockMtpInterval,
            locktime::relative::NumberOf512Seconds,
            locktime::relative::NumberOfBlocks,
        ]
    );

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Formatting adapter has nothing to default to.
    units!([amount::Display], assert_does_not_implement, [Default]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // No enum has a variant that is more default than the others.
    units!(enums, assert_does_not_implement, [Default]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // There is no default failure.
    units!(errors, assert_does_not_implement, [Default]);

    // P-DECODERS: Decoders have a default constructor.
    units!(decoders, assert_implements, [Default]);
    // FORBIDDEN BY DEPENDENCY: consensus_encoding encoders have no `Default`.
    units!(encoders, assert_does_not_implement, [Default]);
}

#[test]
fn display_trait() {
    // TODO: satsfy says IMPLEMENT (satisfied already)
    // Every value type, enum and formatting adapter prints.
    units!(
        public_types,
        assert_implements,
        [Display],
        except [
            // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
            // There is no canonical unit. `to_sat_per_*` methods make it a caller's choice.
            fee_rate::FeeRate,
            // TODO: satsfy IS UNDECIDED
            result::NumOpResult<Amount>,
        ]
    );

    // C-GOOD-ERR: Every error type implements `Display`.
    // REQUIRED BY DEPENDENCY: std::error::Error
    units!(errors, assert_implements, [Display]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Encoders and decoders return bytes and are not printed.
    units!(codecs, assert_does_not_implement, [Display]);
}

#[test]
fn partial_eq_trait() {
    // C-COMMON-TRAITS: Every value type and enum implements `PartialEq`.
    // POLICY: Value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: Eq, PartialOrd
    // REQUIRED BY DEPENDENCY: `LockTime` derives `PartialEq` over its heights and times.
    units!(
        public_types,
        assert_implements,
        [PartialEq],
        except [
            // TODO: satsfy is UNDECIDED
            amount::Display,
        ]
    );

    // POLICY: Every error type derives `PartialEq`.
    // REQUIRED BY DEPENDENCY: Eq
    // REQUIRED BY DEPENDENCY: Errors and `NumOpResult` derive `PartialEq` over inner errors.
    units!(errors, assert_implements, [PartialEq]);

    // FORBIDDEN BY DEPENDENCY: consensus_encoding codecs derive only `Debug, Clone`.
    units!(codecs, assert_does_not_implement, [PartialEq]);
}

#[test]
fn eq_trait() {
    // C-COMMON-TRAITS: Every value type and enum implements `Eq`.
    // POLICY: Value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: Ord
    // REQUIRED BY DEPENDENCY: `LockTime` derives `Eq` over its heights and times.
    units!(
        public_types,
        assert_implements,
        [Eq],
        except [
            // FORBIDDEN BY DEPENDENCY: PartialEq
            amount::Display,
        ]
    );

    // POLICY: Every error type derives `Eq`.
    // REQUIRED BY DEPENDENCY: Errors and `NumOpResult` derive `Eq` over inner errors.
    units!(errors, assert_implements, [Eq]);

    // FORBIDDEN BY DEPENDENCY: PartialEq
    units!(codecs, assert_does_not_implement, [Eq]);
}

#[test]
fn partial_ord_trait() {
    // C-COMMON-TRAITS: Every value type implements `PartialOrd`.
    // POLICY: Value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: Ord
    units!(structs, assert_implements, [PartialOrd]);

    // FORBIDDEN BY DEPENDENCY: PartialEq
    units!([amount::Display], assert_does_not_implement, [PartialOrd]);

    // POLICY: Consider carefully before deriving, it bakes variant order into the public API.
    units!(enums, assert_does_not_implement, [PartialOrd]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Errors are not ordered.
    units!(errors, assert_does_not_implement, [PartialOrd]);

    // FORBIDDEN BY DEPENDENCY: PartialEq
    units!(codecs, assert_does_not_implement, [PartialOrd]);
}

#[test]
fn ord_trait() {
    // C-COMMON-TRAITS: Every value type implements `Ord`.
    // POLICY: Value types use the standard derive set.
    units!(structs, assert_implements, [Ord]);

    // FORBIDDEN BY DEPENDENCY: PartialOrd
    units!(enums, assert_does_not_implement, [Ord]);

    // FORBIDDEN BY DEPENDENCY: Eq, PartialOrd
    units!([amount::Display], assert_does_not_implement, [Ord]);

    // FORBIDDEN BY DEPENDENCY: PartialOrd
    units!(errors, assert_does_not_implement, [Ord]);

    // FORBIDDEN BY DEPENDENCY: Eq, PartialOrd
    units!(codecs, assert_does_not_implement, [Ord]);
}

#[test]
fn hash_trait() {
    // C-COMMON-TRAITS: Every public type SHOULD implement Hash
    // POLICY: Value types use the standard derive set.
    // REQUIRED BY DEPENDENCY: `LockTime` derives `Hash` over its heights and times.
    units!(
        public_types,
        assert_implements,
        [Hash],
        except [
            // FORBIDDEN BY DEPENDENCY: Eq
            amount::Display,
            // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
            // Only labels a failed operation inside an error, never a key.
            result::MathOp,
            // FORBIDDEN BY DEPENDENCY: `NumOpError` has no `Hash`.
            result::NumOpResult<Amount>,
        ]
    );

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Nobody uses an error as a key, and every field inside error must implement Hash too.
    units!(errors, assert_does_not_implement, [Hash]);

    // FORBIDDEN BY DEPENDENCY: Eq
    units!(codecs, assert_does_not_implement, [Hash]);
}

#[test]
fn send_trait() {
    // C-SEND-SYNC: Every public type implements `Send`.
    units!(all, assert_implements, [Send]);
}

#[test]
fn sync_trait() {
    // C-SEND-SYNC: Every public type implements `Sync`.
    units!(all, assert_implements, [Sync]);
}

#[test]
fn serialize_trait() {
    // C-SERDE: Data structures implement `Serialize`.
    units!(
        structs,
        assert_implements,
        [Serialize],
        except [
            // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
            // No canonical unit, sats or BTC? Pick via `serde(with = "amount::serde::as_sat")`.
            amount::Amount,
            amount::SignedAmount,
            fee_rate::FeeRate,
        ]
    );

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Denominations are a display concern and the arithmetic result types are transient.
    units!(
        enums,
        assert_does_not_implement,
        [Serialize],
        except [
            // TODO: satsfy IMPLEMENT (satisfied already)
            // The locktime is stored as a `u32` that can be serialized.
            locktime::absolute::LockTime,
            locktime::relative::LockTime,
        ]
    );

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // A formatting adapter is just for printing.
    units!([amount::Display], assert_does_not_implement, [Serialize]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Errors are just for reporting.
    units!(errors, assert_does_not_implement, [Serialize]);

    // TODO: satsfy says DO NOT IMPLEMENT (satisfied already)
    // Encoders and decoders only hold bytes mid-conversion, so they don't need serialization.
    units!(codecs, assert_does_not_implement, [Serialize]);
}

#[test]
fn deserialize_trait() {
    // C-SERDE: Data structures implement `Deserialize`.
    // REQUIRED BY DEPENDENCY: Serialize
    units!(
        public_types,
        assert_implements,
        [Deserialize],
        except [
            // FORBIDDEN BY DEPENDENCY: Serialize
            amount::Amount,
            amount::SignedAmount,
            fee_rate::FeeRate,
            amount::Denomination,
            result::MathOp,
            result::NumOpResult<Amount>,
            amount::Display,
        ]
    );

    // FORBIDDEN BY DEPENDENCY: Serialize
    units!(errors, assert_does_not_implement, [Deserialize]);

    // FORBIDDEN BY DEPENDENCY: Serialize
    units!(codecs, assert_does_not_implement, [Deserialize]);
}

#[test]
fn arbitrary_trait() {
    // P-ARBITRARY: Public types implement `Arbitrary`.
    units!(
        public_types,
        assert_implements,
        [Arbitrary],
        except [
            // TODO: satsfy says IMPLEMENT
            amount::Display,
        ]
    );

    // TODO: satsfy says IMPLEMENT
    units!(errors, assert_does_not_implement, [Arbitrary]);

    // FORBIDDEN BY DEPENDENCY: consensus_encoding codecs have no `Arbitrary`.
    units!(codecs, assert_does_not_implement, [Arbitrary]);
}

/// C-DEBUG-NONEMPTY: Tests that all public non-error types have non-empty Debug.
#[test]
fn c_debug_nonempty() {
    // TODO: Test error types.
    let debug = [
        format!("{:?}", amount::Denomination::Bitcoin),
        format!("{:?}", absolute::LockTime::Blocks(absolute::Height::MAX)),
        format!("{:?}", relative::LockTime::Blocks(relative::NumberOfBlocks::MAX)),
        format!("{:?}", result::MathOp::Add),
        format!("{:?}", result::NumOpResult::Valid(Amount::MAX)),
        format!("{:?}", Amount::MAX),
        format!("{:?}", Amount::MAX.display_in(amount::Denomination::Bitcoin)),
        format!("{:?}", SignedAmount::MAX),
        format!("{:?}", BlockHeight::MAX),
        format!("{:?}", BlockHeightInterval::MAX),
        format!("{:?}", BlockMtp::MAX),
        format!("{:?}", BlockMtpInterval::MAX),
        format!("{:?}", FeeRate::MAX),
        format!("{:?}", absolute::Height::MAX),
        format!("{:?}", absolute::MedianTimePast::MAX),
        format!("{:?}", relative::NumberOf512Seconds::MAX),
        format!("{:?}", relative::NumberOfBlocks::MAX),
        format!("{:?}", pow::CompactTarget::from_consensus(u32::MAX)),
        format!("{:?}", pow::Target::MAX),
        format!("{:?}", pow::Target::MAX.to_work()),
        format!("{:?}", Sequence::MAX),
        format!("{:?}", BlockTime::from_u32(u32::MAX)),
        format!("{:?}", Weight::MAX),
    ];
    for s in debug {
        assert!(!s.is_empty());
    }
}

/// C-OBJECT: Tests that traits are object-safe where appropriate.
#[test]
fn c_object() {
    // If this builds then traits are dyn compatible.
    struct Traits {
        // These traits are explicitly not dyn compatible.
        // b: Box<dyn amount::serde::SerdeAmount>,
        // c: Box<dyn amount::serde::SerdeAmountForOpt>,
        // d: Box<dyn parse::Integer>, // Because of core::num::ParseIntError
    }
}

macro_rules! assert_format_matches {
    ($type:expr, $num:expr) => {
        let got = format!("{:o}", $type);
        let want = format!("{:o}", $num);
        assert_eq!(got, want);

        let got = format!("{:b}", $type);
        let want = format!("{:b}", $num);
        assert_eq!(got, want);

        let got = format!("{:x}", $type);
        let want = format!("{:x}", $num);
        assert_eq!(got, want);

        let got = format!("{:X}", $type);
        let want = format!("{:X}", $num);
        assert_eq!(got, want);
    };
}

/// C-NEWTYPE: Newtype wrappers format identically to their inner types, maintaining transparency.
#[test]
fn c_newtype_transparent_format() {
    // Confirm that for a set of pseudo-random numbers, formatting is equivalent to the inner value
    let mut rand_num = 10;
    for _ in 0..50 {
        assert_format_matches!(Amount::from_sat_u32(rand_num), rand_num);
        assert_format_matches!(BlockHeight::from(rand_num), rand_num);
        assert_format_matches!(BlockHeightInterval::from(rand_num), rand_num);
        assert_format_matches!(BlockMtp::from(rand_num), rand_num);
        assert_format_matches!(BlockMtpInterval::from(rand_num), rand_num);
        assert_format_matches!(BlockTime::from(rand_num), rand_num);
        assert_format_matches!(
            relative::NumberOfBlocks::from_count(rand_num as u16),
            rand_num as u16
        );
        assert_format_matches!(
            relative::NumberOf512Seconds::from_512_second_intervals(rand_num as u16),
            rand_num as u16
        );
        assert_format_matches!(Sequence::from_consensus(rand_num), rand_num);
        assert_format_matches!(Weight::from_wu(rand_num.into()), u64::from(rand_num));

        if let Ok(height) = absolute::Height::from_u32(rand_num) {
            assert_format_matches!(height, rand_num);
        }
        if let Ok(mtp) = absolute::MedianTimePast::from_u32(rand_num) {
            assert_format_matches!(mtp, rand_num);
        }
        if let Ok(ssat) = SignedAmount::from_sat(i64::from(rand_num)) {
            assert_format_matches!(ssat, rand_num);
            assert_format_matches!(-ssat, -i64::from(rand_num));
        }

        rand_num = rand_num.wrapping_mul(1039).wrapping_add(677);
    }
}

/// P-CONSISTENT-EXPORTS: Tests that modules are exported from the crate root.
#[test]
fn p_consistent_exports_crate_modules() {
    use bitcoin_units::{
        amount, block, fee_rate, locktime, parse_int, pow, result, sequence, time, weight,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that type aliases are exported from the crate root.
#[test]
fn p_consistent_exports_crate_types() {
    use bitcoin_units::{
        Amount, BlockHeight, BlockHeightInterval, BlockMtp, BlockMtpInterval, BlockTime,
        CompactTarget, FeeRate, NumOpResult, Sequence, SignedAmount, Weight,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `amount` module.
#[test]
fn p_consistent_exports_amount() {
    use bitcoin_units::amount::{
        Amount, AmountDecoder, AmountDecoderError, AmountEncoder, Denomination, Display,
        OutOfRangeError, ParseAmountError, ParseDenominationError, ParseError, SignedAmount,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `amount::error` module.
#[test]
fn p_consistent_exports_amount_error() {
    use bitcoin_units::amount::error::{
        MissingDigitsError, OutOfRangeError, ParseAmountError, ParseDenominationError, ParseError,
        PossiblyConfusingDenominationError, TooPreciseError, UnknownDenominationError,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `block` module.
#[test]
fn p_consistent_exports_block() {
    use bitcoin_units::block::{
        BlockHeight, BlockHeightDecoder, BlockHeightDecoderError, BlockHeightEncoder,
        BlockHeightInterval, BlockMtp, BlockMtpInterval, TooBigForRelativeHeightError,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `sequence` module.
#[test]
fn p_consistent_exports_sequence() {
    use bitcoin_units::sequence::{
        Sequence, SequenceDecoder, SequenceDecoderError, SequenceEncoder,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `fee_rate` module.
#[test]
fn p_consistent_exports_fee_rate() {
    use bitcoin_units::fee_rate::serde::OverflowError;
    use bitcoin_units::fee_rate::FeeRate;
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `locktime::absolute` module.
#[test]
fn p_consistent_exports_locktime_absolute() {
    use bitcoin_units::locktime::absolute::error::{
        ConversionError as _, IncompatibleHeightError as _, IncompatibleTimeError as _,
        LockTimeDecoderError as _, ParseHeightError as _, ParseTimeError as _,
    };
    use bitcoin_units::locktime::absolute::{
        ConversionError, IncompatibleHeightError, IncompatibleTimeError, LockTimeDecoder,
        LockTimeDecoderError, LockTimeEncoder, ParseHeightError, ParseTimeError,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `locktime::relative` module.
#[test]
fn p_consistent_exports_locktime_relative() {
    use bitcoin_units::locktime::relative::error::{
        DisabledLockTimeError as _, InvalidHeightError as _, InvalidTimeError as _,
        IsSatisfiedByError as _, IsSatisfiedByHeightError as _, IsSatisfiedByTimeError as _,
        TimeOverflowError as _,
    };
    use bitcoin_units::locktime::relative::{
        DisabledLockTimeError, InvalidHeightError, InvalidTimeError, IsSatisfiedByError,
        IsSatisfiedByHeightError, IsSatisfiedByTimeError, NumberOf512Seconds, NumberOfBlocks,
        TimeOverflowError,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `parse_int` module.
#[test]
fn p_consistent_exports_parse() {
    use bitcoin_units::parse_int::{ParseIntError, PrefixedHexError, UnprefixedHexError};
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `result` module.
#[test]
fn p_consistent_exports_result() {
    use bitcoin_units::result::{MathOp, NumOpError, NumOpResult};
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `pow` module.
#[test]
fn p_consistent_exports_pow() {
    use bitcoin_units::pow::{
        CompactTarget, CompactTargetDecoder, CompactTargetDecoderError, CompactTargetEncoder,
        ParseTargetError, ParseWorkError, Target, Work,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `time` module.
#[test]
fn p_consistent_exports_time() {
    use bitcoin_units::time::{
        BlockTime, BlockTimeDecoder, BlockTimeDecoderError, BlockTimeEncoder,
    };
}

/// P-CONSISTENT-EXPORTS: Tests that all types can be imported from the `weight` module.
#[test]
fn p_consistent_exports_weight() {
    use bitcoin_units::weight::Weight;
}

/// P-DEFAULT-CHANGE: Tests regression for Default implementation values.
#[test]
fn p_default_change() {
    assert_eq!(Amount::default(), Amount::ZERO);
    assert_eq!(SignedAmount::default(), SignedAmount::ZERO);
    assert_eq!(BlockHeightInterval::default(), BlockHeightInterval::ZERO);
    assert_eq!(BlockMtpInterval::default(), BlockMtpInterval::ZERO);
    assert_eq!(relative::NumberOf512Seconds::default(), relative::NumberOf512Seconds::ZERO);
    assert_eq!(relative::NumberOfBlocks::default(), relative::NumberOfBlocks::ZERO);
}

/// P-DECODERS: Tests that decoders implement a constructor method.
#[test]
fn p_decoders_implement_new() {
    let _ = amount::AmountDecoder::new();
    let _ = block::BlockHeightDecoder::new();
    let _ = locktime::absolute::LockTimeDecoder::new();
    let _ = pow::CompactTargetDecoder::new();
    let _ = sequence::SequenceDecoder::new();
    let _ = time::BlockTimeDecoder::new();
}
