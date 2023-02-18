use lazy_static::lazy_static;

/// build_info returns the version information of the current build.
pub fn build_info() -> String {
    format!(
        "{} ({} {})",
        env!("VERGEN_BUILD_SEMVER"),
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_GIT_COMMIT_DATE")
    )
}

lazy_static! {
    pub static ref VERSION: String = build_info();
}

// get_version returns the version of the current build.
pub fn get_version() -> &'static str {
    &VERSION
}
