//! Check command - validate configuration and content

use std::path::Path;

use color_eyre::eyre::{Result, bail};
use typstify_core::Config;
use typstify_parser::ParserRegistry;

/// Validation result.
#[derive(Debug, Default)]
struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationResult {
    fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Run the check command.
///
/// Validates configuration and all content files.
pub fn run(config_path: &Path, strict: bool) -> Result<()> {
    tracing::info!(?config_path, strict, "Checking configuration and content");

    let mut result = ValidationResult::default();

    // Validate configuration
    println!("Checking configuration...");
    let config = match Config::load(config_path) {
        Ok(c) => {
            println!("  ✓ Configuration valid");
            Some(c)
        }
        Err(e) => {
            result.add_error(format!("Configuration error: {e}"));
            println!("  ✗ Configuration invalid: {e}");
            None
        }
    };

    // Validate content files
    let content_dir = Path::new("content");
    if content_dir.exists() {
        println!("\nChecking content files...");
        validate_content_files(content_dir, &mut result)?;
    } else {
        result.add_warning("Content directory does not exist");
    }

    // Check required directories
    println!("\nChecking directories...");
    check_directories(&mut result);

    // Check for common issues
    if let Some(ref cfg) = config {
        println!("\nChecking configuration values...");
        check_config_values(cfg, &mut result);
    }

    // Print summary
    println!();
    println!("Summary:");
    println!("  Errors:   {}", result.errors.len());
    println!("  Warnings: {}", result.warnings.len());

    if result.has_errors() {
        println!();
        println!("Errors:");
        for err in &result.errors {
            println!("  ✗ {err}");
        }
    }

    if result.has_warnings() {
        println!();
        println!("Warnings:");
        for warn in &result.warnings {
            println!("  ⚠ {warn}");
        }
    }

    // Determine exit status
    if result.has_errors() {
        bail!("Validation failed with {} error(s)", result.errors.len());
    }

    if strict && result.has_warnings() {
        bail!(
            "Validation failed with {} warning(s) (strict mode)",
            result.warnings.len()
        );
    }

    println!();
    println!("✓ All checks passed");

    Ok(())
}

/// Validate all content files in the given directory.
fn validate_content_files(dir: &Path, result: &mut ValidationResult) -> Result<()> {
    let registry = ParserRegistry::new();
    let mut checked = 0;
    let mut failed = 0;

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Skip non-content files
        if !matches!(ext, "md" | "typ") {
            continue;
        }

        checked += 1;

        // Try to parse the file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                result.add_error(format!("{}: Failed to read file: {e}", path.display()));
                failed += 1;
                continue;
            }
        };

        if let Err(e) = registry.parse(&content, path) {
            result.add_error(format!("{}: Parse error: {e}", path.display()));
            failed += 1;
        }
    }

    if failed == 0 {
        println!("  ✓ All {checked} content files valid");
    } else {
        println!("  ✗ {failed}/{checked} content files have errors");
    }

    Ok(())
}

/// Check that required directories exist.
fn check_directories(result: &mut ValidationResult) {
    let dirs = [
        ("content", true),
        ("templates", false),
        ("style", false),
        ("assets", false),
    ];

    for (dir, required) in dirs {
        let path = Path::new(dir);
        if path.exists() {
            println!("  ✓ {dir}/ exists");
        } else if required {
            result.add_error(format!("Required directory missing: {dir}/"));
            println!("  ✗ {dir}/ missing (required)");
        } else {
            result.add_warning(format!("Optional directory missing: {dir}/"));
            println!("  ⚠ {dir}/ missing (optional)");
        }
    }
}

/// Check configuration values for common issues.
fn check_config_values(config: &Config, result: &mut ValidationResult) {
    // Check base_url
    if config.site.base_url.is_empty() {
        result.add_warning("site.base_url is empty");
    } else if !config.site.base_url.starts_with("http") {
        result.add_warning("site.base_url should start with http:// or https://");
    }

    // Check title
    if config.site.title.is_empty() {
        result.add_warning("site.title is empty");
    }

    // Check output directory
    let output = Path::new(&config.build.output_dir);
    if output.exists() && !output.is_dir() {
        result.add_error(format!(
            "Output path exists but is not a directory: {}",
            config.build.output_dir
        ));
    }

    println!("  ✓ Configuration values checked");
}
