use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let binding = PathBuf::from(&target_dir);
    let workspace_root = &binding.parent().unwrap().parent().unwrap();
    let elf_path = workspace_root
        .join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release/zkdb_merkle");
    // Set the ELF env variable.
    let elf_path_str = elf_path.to_str().unwrap();
    println!("cargo:rustc-env=ZKDB_ELF_PATH={}", elf_path_str);

    // Skip RISC-V compilation if running under clippy.
    if env::var("CLIPPY_ARGS").is_ok() {
        println!("cargo:warning=Skipping RISC-V compilation during clippy check");
        return;
    }

    // Run cargo prove build.
    let status = Command::new("cargo")
        .current_dir(workspace_root.join("crates/zkdb-merkle"))
        .args([
            "prove",
            "build",
            "--output-directory",
            &workspace_root
                .join("target/elf-compilation")
                .display()
                .to_string(),
            "--elf-name",
            "zkdb_merkle",
        ])
        .status()
        .expect("Failed to execute cargo prove build");

    if !status.success() {
        panic!("Failed to build zkdb_merkle with cargo prove build");
    }

    if !elf_path.exists() {
        panic!(
            "zkdb_merkle.elf not found at {:?} after cargo prove build",
            elf_path
        );
    }

    // Tell cargo to rerun this script if the ELF file changes
    println!("cargo:rerun-if-changed={}", elf_path.display());
}
