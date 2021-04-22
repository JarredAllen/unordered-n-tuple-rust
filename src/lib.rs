#![no_std]

//! This crate provides a `UnorderedNTuple`, which is a struct that represents unordered tuples of
//! n homogenous elements.
//!
//! ## Crate Features
//! - `std`: Enables dependence on `std` to allow for more features
//! - `serde`: Enables serializing/deserializing the `UnorderedNTuple` struct in serde
//!
//! By default, both features are enabled.

macro_rules! if_feature {
    ($s:literal, $($i:item)*) => ($(
        #[cfg(feature = $s)] $i
    )*)
}

#[rustfmt::skip]
if_feature!("std", extern crate std; use std::hash::{Hash, Hasher};);

#[rustfmt::skip]
if_feature!(
    "serde",
    use std::{convert::TryInto, marker::PhantomData, fmt, vec::Vec};
    use serde::{
        de::{Deserialize, Deserializer, Error, SeqAccess, Visitor},
        ser::{Serialize, Serializer, SerializeSeq},
    };
);

/// An `UnorderedPair` is a special subtype of `UnorderedNTuple` for only 2 elements. This has been
/// given its own type for ease of use.
///
/// It can also be converted to or from a tuple (similar impls for larger types will come once
/// generics become stronger).
pub type UnorderedPair<T> = UnorderedNTuple<T, 2>;

impl<T> From<(T, T)> for UnorderedPair<T> {
    fn from(tuple: (T, T)) -> Self {
        Self([tuple.0, tuple.1])
    }
}
impl<T> From<UnorderedPair<T>> for (T, T) {
    fn from(pair: UnorderedPair<T>) -> (T, T) {
        let [first, second] = pair.0;
        (first, second)
    }
}

/// A type which represents an unordered tuple of N elements (i.e. an unordered pair if N == 2, and
/// unordered triplet if N == 3, and so on).
///
/// Two `UnorderedNTuple`s are equivalent if their elements are equivalent in any order, for
/// example:
/// ```
/// # use unordered_n_tuple::UnorderedNTuple;
/// assert_eq!(UnorderedNTuple([0, 3, 5]), UnorderedNTuple([5, 0, 3]));
/// ```
#[derive(Copy, Clone, Debug, Eq)]
pub struct UnorderedNTuple<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> From<[T; N]> for UnorderedNTuple<T, N> {
    fn from(arg: [T; N]) -> Self {
        Self(arg)
    }
}

impl<T, const N: usize> From<UnorderedNTuple<T, N>> for [T; N] {
    fn from(arg: UnorderedNTuple<T, N>) -> Self {
        arg.0
    }
}

impl<T, const N: usize> PartialEq for UnorderedNTuple<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &UnorderedNTuple<T, N>) -> bool {
        let mut used_indices = [false; N];
        for element in self.0.iter() {
            let mut found = false;
            for (index, other_element) in other.0.iter().enumerate() {
                if used_indices[index] {
                    continue;
                }
                if element == other_element {
                    used_indices[index] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }
}

#[rustfmt::skip]
if_feature!(
    "std",
    impl<T, const N: usize> Hash for UnorderedNTuple<T, N>
    where
        T: Hash + Ord + Clone,
    {
        fn hash<H: Hasher>(&self, state: &mut H) {
            let mut sorted = self.0.clone();
            sorted.sort();
            Hash::hash_slice(&sorted, state);
        }
    }
);

#[rustfmt::skip]
if_feature!(
    "serde",
    impl<T: Serialize, const N: usize> Serialize for UnorderedNTuple<T, N> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(N))?;
            for item in self.0.iter() {
                seq.serialize_element(item)?;
            }
            seq.end()
        }
    }
    struct UnorderedNTupleVisitor<T, const N: usize> {
        _phantom: PhantomData<fn() -> [T; N]>,
    }
    impl<T, const N: usize> UnorderedNTupleVisitor<T, N> {
        fn new() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }
    impl<'de, T, const N: usize> Visitor<'de> for UnorderedNTupleVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = UnorderedNTuple<T, N>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("Expecting a sequence with N homogenous elements of type T")
        }

        fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            if access.size_hint() != Some(N) {
                return Err(S::Error::custom("Wrong number of elements"));
            }
            let mut data: Vec<T> = Vec::new();
            for _ in 0..N {
                data.push(access.next_element()?.unwrap())
            }
            Ok(UnorderedNTuple(
                data.try_into().unwrap_or_else(|_| unreachable!()),
            ))
        }
    }
    impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for UnorderedNTuple<T, N> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(UnorderedNTupleVisitor::<T, N>::new())
        }
    }
);

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    /// Check that two pairs are equal, regardless of element order
    #[quickcheck]
    fn check_pair_equality(a: usize, b: usize) -> bool {
        UnorderedNTuple([a, b]) == UnorderedNTuple([b, a])
            && UnorderedNTuple([a, b]) == UnorderedNTuple([a, b])
    }

    /// Check that two singleton sets compare the same as their members
    #[quickcheck]
    fn check_singleton_equality(a: usize, b: usize) -> bool {
        (a == b) == (UnorderedNTuple([a]) == UnorderedNTuple([b]))
    }

    /// Check that pairs with non-equal elements actually compare non-equal
    #[quickcheck]
    fn check_pair_inequality(a: usize, b: usize, c: usize) -> TestResult {
        if b == c {
            TestResult::discard()
        } else {
            TestResult::from_bool(UnorderedNTuple([a, b]) != UnorderedNTuple([a, c]))
        }
    }

    /// Check that triples with equal elements compare equal, regardless of order
    #[quickcheck]
    fn check_triple_equality(a: usize, b: usize, c: usize) -> bool {
        let triples = [
            UnorderedNTuple([a, b, c]),
            UnorderedNTuple([b, a, c]),
            UnorderedNTuple([b, c, a]),
            UnorderedNTuple([a, c, b]),
            UnorderedNTuple([c, a, b]),
            UnorderedNTuple([c, b, a]),
        ];
        for triple_a in triples.iter() {
            for triple_b in triples.iter() {
                if triple_a != triple_b {
                    return false;
                }
            }
        }
        true
    }
}
