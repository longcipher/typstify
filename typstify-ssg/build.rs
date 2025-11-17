use std::{path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../style/input.css");
    println!("cargo:rerun-if-changed=../tailwind.config.js");
    println!("cargo:rerun-if-changed=../package.json");

    // Check if bun is available
    let has_bun = Command::new("bun").arg("--version").output().is_ok();

    if !has_bun {
        println!("cargo:warning=bun is not available, CSS will not be built automatically");
        return;
    }

    // Check if output.css exists, if not, build it
    let output_css = Path::new("../style/output.css");
    if !output_css.exists() {
        println!("cargo:warning=Building CSS with Tailwind...");

        let output = Command::new("bun")
            .arg("x")
            .arg("tailwindcss")
            .arg("--input")
            .arg("../style/input.css")
            .arg("--output")
            .arg("../style/output.css")
            .arg("--minify")
            .current_dir("..")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!("cargo:warning=CSS built successfully");
            }
            Ok(output) => {
                println!(
                    "cargo:warning=Failed to build CSS: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => {
                println!("cargo:warning=Failed to run tailwindcss command: {}", e);
            }
        }
    }
}
