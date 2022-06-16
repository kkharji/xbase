use super::{Platform, ProjectDependency, ProjectTargetType};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tap::Pipe;

/// Represent XCode Target
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTarget {
    /// Target Type
    pub r#type: ProjectTargetType,

    /// Target Platform
    #[serde(deserialize_with = "value_or_vec", default)]
    pub platform: Vec<Platform>,

    /// The deployment target If this is not specified the value from the project set in
    /// project.options.deploymentTarget.platform will be used.
    #[serde(deserialize_with = "version_or_hashmap", default)]
    pub deployment_target: HashMap<Platform, String>,

    /// Source directories of the target
    // #[serde(deserialize_with = "value_or_vec", default)]
    // pub sources: Vec<String>,

    /// Config Files
    #[serde(default)]
    pub config_files: HashMap<String, String>,

    /// Target specific build settings.
    ///
    /// Default platform and product type settings will be applied
    /// first before any custom settings defined here
    #[serde(default)]
    pub settings: HashMap<String, Value>,

    /// Target Dependencies
    #[serde(deserialize_with = "de_project_dependency", default)]
    #[serde(serialize_with = "se_project_dependency")]
    pub dependencies: Vec<ProjectDependency>,
}

pub(crate) fn version_or_hashmap<'de, D>(
    deserializer: D,
) -> Result<HashMap<Platform, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum ValueOrMap {
        Key(Platform),
        Float(f64),
        Number(i32),
        Map(HashMap<Platform, String>),
    }

    match ValueOrMap::deserialize(deserializer) {
        Ok(ValueOrMap::Key(s)) => Ok(HashMap::from([(s, String::default())])),
        Ok(ValueOrMap::Float(s)) => Ok(HashMap::from([(Platform::default(), s.to_string())])),
        Ok(ValueOrMap::Number(s)) => Ok(HashMap::from([(Platform::default(), s.to_string())])),
        Ok(ValueOrMap::Map(v)) => Ok(v),
        _ => Ok(HashMap::default()),
    }
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

fn de_project_dependency<'de, D>(deserializer: D) -> Result<Vec<ProjectDependency>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use ProjectDependency::*;

    Vec::<HashMap<String, String>>::deserialize(deserializer)?
        .into_iter()
        .flat_map(|map| {
            map.into_iter()
                .map(|(k, v)| match k.as_str() {
                    "bundle" => Bundle(v),
                    "carthage" => Carthage(v),
                    "framework" => Framework(v),
                    "package" => Package(v),
                    "sdk" => Sdk(v),
                    "target" => Target(v),
                    _ => None,
                })
                .collect::<Vec<ProjectDependency>>()
        })
        .collect::<Vec<ProjectDependency>>()
        .pipe(Ok)
}

fn se_project_dependency<S>(value: &[ProjectDependency], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use ProjectDependency::*;

    let to_hashmap = |k: &str, v: &String| Some(HashMap::from([(k.to_string(), v.to_owned())]));
    value
        .iter()
        .map(|t| match t {
            Bundle(v) => to_hashmap("bundle", v),
            Carthage(v) => to_hashmap("carthage", v),
            Framework(v) => to_hashmap("framework", v),
            Package(v) => to_hashmap("package", v),
            Sdk(v) => to_hashmap("sdk", v),
            Target(v) => to_hashmap("target", v),
            None => Option::None,
        })
        .flatten()
        .pipe(|iter| serializer.collect_seq(iter))
}
