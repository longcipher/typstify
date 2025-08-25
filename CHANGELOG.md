# Changelog

## [Unreleased] - 2025-08-25

### Added
- **Embedded CSS Support**: CSS files are now embedded directly into the `typstify-ssg` binary
  - Users can now distribute a single standalone binary without external CSS dependencies
  - Added `just build-standalone` command to create distribution-ready binaries
  - Build script automatically generates CSS during compilation if needed

### Changed
- Modified `copy_styles()` method to write embedded CSS content instead of copying external files
- Updated build process to include CSS generation in the binary build pipeline
- `just build` command no longer requires manual CSS copying step

### Technical Details
- Added `build.rs` script that automatically runs Tailwind CSS build during compilation
- Used Rust's `include_str!` macro to embed `style/output.css` into the binary
- Updated `lib.rs` to write embedded CSS content to output directories
- Standalone binaries are now self-contained with all required stylesheets

### Usage
```bash
# Build standalone binary for distribution
just build-standalone

# The resulting binary in dist/typstify-ssg is completely self-contained
./dist/typstify-ssg build -o my-site
```

This change makes it much easier to distribute `typstify-ssg` to users who don't need to install Node.js or manage CSS build processes.
