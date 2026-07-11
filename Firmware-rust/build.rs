//! Cargo build hook that rejects repeated empty separator lines in Rust sources.
//!
//! Keeping the layout check in every build prevents mechanically generated or
//! refactored firmware files from regressing into the hard-to-read spacing that
//! motivated the repository's one-empty-line rule.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::env;
use std::fs;
use std::path::Path;

/// Scans the package and stops compilation when a source violates the layout rule.
///
/// Cargo owns this entry point, so failure is reported as a build error where it
/// cannot be overlooked during an ordinary debug or release build.
fn main()
{
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let mut violations = Vec::new();
    check_directory(Path::new(&manifest_directory), &mut violations);
    if !violations.is_empty()
    {
        panic!(
            "Rust source layout check failed; keep at most one empty line and separate a completed item from the next doc comment:\n{}",
            violations.join("\n")
        );
    }
}

/// Recursively checks Rust files below `directory` and records all violations.
///
/// Generated, tracker, VCS, IDE, and temporary directories are skipped because
/// they are not maintained source. Rerun directives keep Cargo's cache correct
/// when either a checked file or the directory membership changes.
fn check_directory(directory: &Path, violations: &mut Vec<String>)
{
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory).expect("read source directory")
    {
        let path = entry.expect("read source entry").path();
        if path.is_dir()
        {
            let name = path.file_name().and_then(|name| name.to_str());
            if !matches!(name, Some("target" | ".beads" | ".git" | ".idea" | ".temp"))
            {
                check_directory(&path, violations);
            }
        }
        else if path.extension().and_then(|extension| extension.to_str()) == Some("rs")
        {
            println!("cargo:rerun-if-changed={}", path.display());
            check_file(&path, violations);
        }
    }
}

/// Records repeated empty lines and documentation attached without visual separation.
///
/// Reporting one location per run keeps the compiler message actionable while
/// still allowing a single build to reveal problems across multiple files. A
/// completed item followed immediately by `///` requires one empty line so
/// separate declarations, fields, and variants remain scannable. Opening braces
/// and attributes stay attached to the documentation for their first item.
fn check_file(path: &Path, violations: &mut Vec<String>)
{
    let content = fs::read_to_string(path).expect("read Rust source");
    let mut consecutive_empty_lines = 0;
    let mut previous_line: Option<&str> = None;
    for (index, line) in content.lines().enumerate()
    {
        if line.trim_start().starts_with("///")
            && previous_line.is_some_and(|previous| {
                let trimmed = previous.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with("///")
                    && !trimmed.starts_with('#')
                    && !trimmed.ends_with('{')
            })
        {
            violations.push(format!(
                "{}:{} (add one empty line between the previous item and this documentation)",
                path.display(),
                index + 1
            ));
        }
        if line.trim().is_empty()
        {
            consecutive_empty_lines += 1;
            if consecutive_empty_lines == 2
            {
                violations.push(format!("{}:{}", path.display(), index + 1));
            }
        }
        else
        {
            consecutive_empty_lines = 0;
        }
        previous_line = Some(line);
    }
}
