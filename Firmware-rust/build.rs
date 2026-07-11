//! Cargo build hook that rejects unreadable layout in maintained Rust sources.
//!
//! Keeping the layout check in every build prevents mechanically generated or
//! refactored firmware files from regressing into the hard-to-read spacing that
//! motivated the repository's spacing and multi-line-function rules.

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
            "Rust source layout check failed; keep at most one empty line, separate a completed item from the next doc comment, put function bodies on their own lines, keep test bodies in dedicated *_tests.rs files, and aggregate test observations with TestFailures:\n{}",
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

/// Records repeated empty lines, attached documentation, and one-line function bodies.
///
/// Reporting one location per run keeps the compiler message actionable while
/// still allowing a single build to reveal problems across multiple files. A
/// completed item followed immediately by `///` requires one empty line so
/// separate declarations, fields, and variants remain scannable. Opening braces
/// and attributes stay attached to the documentation for their first item.
/// Function bodies use separate lines even when empty so implementations remain
/// visually consistent and have room for future behavior or rationale.
fn check_file(path: &Path, violations: &mut Vec<String>)
{
    let content = fs::read_to_string(path).expect("read Rust source");
    let is_test_file = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("_tests.rs"));
    let mut consecutive_empty_lines = 0;
    let mut previous_previous_line: Option<&str> = None;
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
        if is_one_line_function(line)
        {
            violations.push(format!(
                "{}:{} (put the function body on lines between the opening and closing braces)",
                path.display(),
                index + 1
            ));
        }
        if is_inline_test_module(line)
        {
            violations.push(format!(
                "{}:{} (move the inline test module body to a sibling *_tests.rs file)",
                path.display(),
                index + 1
            ));
        }
        if is_test_file && is_fail_fast_test_assertion(line)
        {
            violations.push(format!(
                "{}:{} (record the observation with TestFailures instead of a fail-fast assertion macro)",
                path.display(),
                index + 1
            ));
        }
        if line.trim() == "}"
            && previous_line.is_some_and(|previous| previous.trim().is_empty())
            && previous_previous_line
                .is_some_and(|previous| previous.trim() == "assert.finish();")
        {
            violations.push(format!(
                "{}:{} (remove the empty line between assert.finish(); and the test's closing brace)",
                path.display(),
                index
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
        previous_previous_line = previous_line;
        previous_line = Some(line);
    }
}

/// Returns whether a test line uses a standard assertion macro that stops at its first failure.
fn is_fail_fast_test_assertion(line: &str) -> bool
{
    let code = line.trim_start();
    !code.starts_with("//")
        && (code.contains("assert!(")
            || code.contains("assert_eq!(")
            || code.contains("assert_ne!("))
}

/// Detects both Rust's same-line and Allman opening-brace forms for an inline
/// `tests` module while allowing the required external `mod tests;` declaration.
fn is_inline_test_module(line: &str) -> bool
{
    let trimmed = line.trim();
    trimmed == "mod tests" || trimmed.starts_with("mod tests {")
}

/// Returns whether `line` contains a complete function definition and body.
///
/// Function-pointer types and macros are excluded because their braces do not
/// delimit a Rust function body. Comments are ignored so documentation examples
/// can show compact syntax without affecting the maintained source layout.
fn is_one_line_function(line: &str) -> bool
{
    let trimmed = line.trim();
    if trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("macro_rules!")
    {
        return false;
    }

    let Some(function_position) = trimmed.find("fn ") else
    {
        return false;
    };
    let prefix = &trimmed[..function_position];
    if prefix.contains('=') || prefix.contains(':') || prefix.contains('!')
    {
        return false;
    }

    let function = &trimmed[function_position..];
    let Some(opening_brace) = function.find('{') else
    {
        return false;
    };
    let Some(closing_brace) = function.rfind('}') else
    {
        return false;
    };

    opening_brace < closing_brace
}
