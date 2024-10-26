use std::error::Error;

use vergen::EmitBuilder;

use std::{
    io::{self, Write},
    process,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=migrations");

    EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .all_rustc()
        .emit()?;

    build_tailwind();

    Ok(())
}

fn build_tailwind() {
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=frontend/src/styles.css");

    match process::Command::new("sh")
        .arg("-c")
        .arg("pnpm run build:tailwind")
        .current_dir("frontend")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let _ = io::stdout().write_all(&output.stdout);
                let _ = io::stdout().write_all(&output.stderr);
                panic!("Tailwind error");
            }
        }
        Err(e) => panic!("Tailwind error: {:?}", e),
    };
}
