//! Build script for Ferrite
//!
//! This script handles platform-specific build tasks:
//! - All platforms: bakes an optional build label into the binary
//! - Windows: Embeds the application icon into the executable

fn main() {
    emit_build_label();

    // Windows: Embed icon into the executable
    #[cfg(target_os = "windows")]
    {
        let rc_path = std::path::Path::new("assets/icons/windows/app.rc");
        let ico_path = std::path::Path::new("assets/icons/windows/app.ico");

        // Only embed if both the .rc and .ico files exist
        if rc_path.exists() && ico_path.exists() {
            embed_resource::compile("assets/icons/windows/app.rc", embed_resource::NONE);
            println!("cargo:rerun-if-changed=assets/icons/windows/app.rc");
            println!("cargo:rerun-if-changed=assets/icons/windows/app.ico");
        } else {
            println!(
                "cargo:warning=Windows icon files not found at {:?} and {:?}. \
                 The executable will not have an embedded icon.",
                rc_path, ico_path
            );
        }
    }

    // Rerun build script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

/// Bake `FERRITE_BUILD_LABEL` into the binary so the About box can name the
/// branch and commit a build came from.
///
/// The label is informational only: an unset variable compiles to an empty
/// string and About simply shows nothing extra. It is sanitised to a single
/// line first, because `cargo:rustc-env` output is line-oriented and an
/// embedded newline would be parsed as another build-script directive.
fn emit_build_label() {
    let label = std::env::var("FERRITE_BUILD_LABEL").unwrap_or_default();
    let sanitized: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();

    println!("cargo:rustc-env=FERRITE_BUILD_LABEL={}", sanitized.trim());
    println!("cargo:rerun-if-env-changed=FERRITE_BUILD_LABEL");
}
