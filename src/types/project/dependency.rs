#[derive(Debug)]
pub enum ProjectDependency {
    /// Links to another target. If you are using project references you can specify a target
    /// within another project by using ProjectName/TargetName for the name
    Target(String),
    /// Links to a framework or XCFramework
    Framework(String),
    /// Helper for linking to a Carthage framework (not XCFramework)
    Carthage(String),
    /// Links to a dependency with the SDK. This can either be a relative path within the sdk root
    /// or a single filename that references a framework (.framework) or lib (.tbd)
    Sdk(String),
    /// Links to a Swift Package. The name must match the name of a package defined in the top
    /// level packages
    Package(String),
    /// Adds the pre-built bundle for the supplied name to the copy resources build phase. This is
    /// useful when a dependency exists on a static library target that has an associated bundle
    /// target, both existing in a separate project. Only usable in target types which can copy
    /// resources.
    Bundle(String),
    None,
}
