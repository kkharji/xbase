#![allow(dead_code)]

use serde::de::{Deserialize, Deserializer, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;

/// Deserialize an optional type to default if none
pub fn value_or_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

/// Deserialize a vector possibly containing `null`.
pub fn from_nullable_vector<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Vec::<Option<T>>::deserialize(deserializer).map(|v| v.into_iter().flatten().collect())
}

/// Deserialize from a string of comma seperated items
fn from_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    Ok(s.split(", ")
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
        .collect())
}

/// Deserializes a sequence or null values as a vec.
///
pub fn deserialize_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // Visits a sequence or null type, returning either the sequence
    // or an empty vector.
    //
    struct VecVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for VecVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("sequence or unit or string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();

            while let Some(item) = seq.next_element()? {
                vec.push(item);
            }

            Ok(vec)
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_str(&v)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let value = T::deserialize(v.into_deserializer())?;

            Ok(vec![value])
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Default::default())
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(VecVisitor(PhantomData))
        }
    }

    deserializer.deserialize_option(VecVisitor(PhantomData))
}

pub(crate) fn value_or_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum ValueOrVec<T> {
        Value(T),
        Vec(Vec<T>),
        None,
    }

    match ValueOrVec::deserialize(deserializer) {
        Ok(ValueOrVec::Value(s)) => Ok(vec![s]),
        Ok(ValueOrVec::Vec(v)) => Ok(v),
        _ => Ok(vec![]),
    }
}

/// Deserializes a map or null values as a HashMap.
///
pub fn deserialize_hashmap<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de> + Default,
    K: Deserialize<'de> + Eq + std::hash::Hash,
{
    // Visits a map or null type, returning either the mapping
    // or an empty HashMap.
    //
    struct MapVisitor<K, V>(PhantomData<(K, V)>);

    impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
    where
        V: Deserialize<'de> + Default,
        K: Eq + std::hash::Hash + Deserialize<'de>,
    {
        type Value = HashMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map or unit")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut hashmap = HashMap::new();

            while let Some((key, value)) = map.next_entry()? {
                hashmap.insert(key, value);
            }

            Ok(hashmap)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Default::default())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_str(&v)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut hashmap = HashMap::new();
            let key = K::deserialize(v.into_deserializer())?;

            hashmap.insert(key, Default::default());

            Ok(hashmap)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(MapVisitor(PhantomData))
        }
    }

    deserializer.deserialize_option(MapVisitor(PhantomData))
}
