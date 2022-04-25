use lazy_static::lazy_static;
use regex::Regex;

#[cfg(feature = "compilation")]
lazy_static! {
    static ref COMPILE_SWIFT_SOURCES: Regex = Regex::new(r"^CompileSwiftSources\s*").unwrap();
    static ref COMPILE_SWIFT: Regex = Regex::new(r"^CompileSwift\s+ \w+\s+ \w+\s+ (.+)$").unwrap();
}

#[cfg(feature = "compilation")]
pub fn matches_compile_swift_sources(text: &str) -> bool {
    COMPILE_SWIFT_SOURCES.is_match(text)
}

#[cfg(feature = "compilation")]
pub fn matches_compile_swift(text: &str) -> bool {
    COMPILE_SWIFT.is_match(text)
}
