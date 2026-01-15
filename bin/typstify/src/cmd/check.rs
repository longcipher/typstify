//! Check command - validate configuration and content

use std::{collections::HashMap, path::Path};

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

        // Check for multi-language content completeness
        if let Some(ref cfg) = config {
            println!("\nChecking multi-language content...");
            validate_language_content(content_dir, cfg, &mut result)?;
        }
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

/// Quick validation for build/watch commands.
///
/// Returns warnings for missing language translations (non-fatal).
/// Call this before starting build/watch.
pub fn quick_validate(config: &Config) -> Vec<String> {
    let mut warnings = Vec::new();
    let content_dir = Path::new("content");

    if !content_dir.exists() {
        return warnings;
    }

    let all_langs = config.all_languages();
    if all_langs.len() <= 1 {
        return warnings;
    }

    let default_lang = &config.site.default_language;

    // Collect all content files and group by canonical name
    let mut files_by_canonical: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(content_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if let Some((canonical, lang)) =
                    parse_content_file(&filename, default_lang, &all_langs)
                {
                    files_by_canonical.entry(canonical).or_default().push(lang);
                }
            }
        }
    }

    // Check for missing translations of index page
    let canonical = "index";
    if let Some(langs) = files_by_canonical.get(canonical) {
        for lang in &all_langs {
            let lang_str = (*lang).to_string();
            if !langs.contains(&lang_str) {
                let expected_file = if *lang == default_lang.as_str() {
                    "index.md".to_string()
                } else {
                    format!("index.{lang}.md")
                };
                warnings.push(format!(
                    "Missing content/{expected_file} - visiting /{lang}/  will show 404",
                ));
            }
        }
    }

    warnings
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

    // Check for conflicting language settings
    let all_langs = config.all_languages();
    if all_langs.len() > 1 {
        // Check if default language is in the languages map
        if !config.languages.contains_key(&config.site.default_language)
            && config.site.default_language != "en"
        {
            result.add_warning(format!(
                "Default language '{}' not explicitly configured in [languages] section",
                config.site.default_language
            ));
        }
    }

    println!("  ✓ Configuration values checked");
}

/// Validate multi-language content completeness.
///
/// Checks that important content files exist for all configured languages.
fn validate_language_content(
    content_dir: &Path,
    config: &Config,
    result: &mut ValidationResult,
) -> Result<()> {
    let all_langs = config.all_languages();

    // Only check if multiple languages are configured
    if all_langs.len() <= 1 {
        println!("  ✓ Single language configured, skipping multi-language checks");
        return Ok(());
    }

    let default_lang = &config.site.default_language;

    // Collect all content files and group by canonical name
    let mut files_by_canonical: HashMap<String, Vec<String>> = HashMap::new();

    for entry in walkdir::WalkDir::new(content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let relative = path
            .strip_prefix(content_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Parse the filename to extract language suffix
        // e.g., "index.md" (default), "index.zh.md" (zh), "posts/hello.md", "posts/hello.zh.md"
        if let Some((canonical, lang)) = parse_content_file(&relative, default_lang, &all_langs) {
            files_by_canonical.entry(canonical).or_default().push(lang);
        }
    }

    // Check for missing translations of important pages
    let mut missing_count = 0;
    let important_pages = ["index.md", "about.md"];

    for page in important_pages {
        let canonical = page.replace(".md", "");
        if let Some(langs) = files_by_canonical.get(&canonical) {
            for lang in &all_langs {
                let lang_str = (*lang).to_string();
                if !langs.contains(&lang_str) {
                    let expected_file = if *lang == default_lang.as_str() {
                        page.to_string()
                    } else {
                        page.replace(".md", &format!(".{lang}.md"))
                    };
                    result.add_warning(format!(
                        "Missing translation: content/{expected_file} (language: {lang})",
                    ));
                    missing_count += 1;
                }
            }
        }
    }

    // Report summary
    let total_pages = files_by_canonical.len();
    let mut fully_translated = 0;
    let mut partially_translated = 0;

    for langs in files_by_canonical.values() {
        if langs.len() == all_langs.len() {
            fully_translated += 1;
        } else if langs.len() > 1 {
            partially_translated += 1;
        }
    }

    if missing_count == 0 {
        println!(
            "  ✓ All important pages have translations ({} languages)",
            all_langs.len()
        );
    } else {
        println!("  ⚠ {missing_count} missing translation(s) for important pages");
    }

    println!(
        "  ℹ Content summary: {total_pages} pages, {fully_translated} fully translated, {partially_translated} partially translated"
    );

    Ok(())
}

/// Parse a content file path to extract canonical name and language.
///
/// Returns (canonical_name, language_code)
fn parse_content_file(
    path: &str,
    default_lang: &str,
    all_langs: &[&str],
) -> Option<(String, String)> {
    let ext = if path.ends_with(".md") {
        ".md"
    } else if path.ends_with(".typ") {
        ".typ"
    } else {
        return None;
    };

    let without_ext = path.strip_suffix(ext)?;

    // Check for language suffix like ".zh" or ".ja"
    for lang in all_langs {
        if *lang != default_lang {
            let suffix = format!(".{lang}");
            if without_ext.ends_with(&suffix) {
                let canonical = without_ext.strip_suffix(&suffix)?.to_string();
                return Some((canonical, (*lang).to_string()));
            }
        }
    }

    // No language suffix means default language
    Some((without_ext.to_string(), default_lang.to_string()))
}
