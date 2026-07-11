use std::env;
use std::fs;
use std::path::Path;

fn main()
{
    let manifest_directory = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let mut violations = Vec::new();
    check_directory(Path::new(&manifest_directory), &mut violations);
    if !violations.is_empty()
    {
        panic!(
            "Rust source layout check failed; keep at most one empty line between items:\n{}",
            violations.join("\n")
        );
    }
}

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

fn check_file(path: &Path, violations: &mut Vec<String>)
{
    let content = fs::read_to_string(path).expect("read Rust source");
    let mut consecutive_empty_lines = 0;
    for (index, line) in content.lines().enumerate()
    {
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
    }
}
