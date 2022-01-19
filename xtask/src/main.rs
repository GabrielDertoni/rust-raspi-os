use std::{ env, io };
use std::process::{ Command, Stdio };
use std::ops::Not;

const ARCH: &str = "aarch64-unknown-none-softfloat";
const KERNEL_OUT: &str = "target/aarch64-unknown-none-softfloat/release/kernel";
const KERNEL_BIN: &str = "kernel.bin";
const LINKER_FILE: &str = "src/arch/link.ld";

type AnyErr = Box<dyn std::error::Error>;
type Result = std::result::Result<(), AnyErr>;

fn main() {
    let subcommand = std::env::args().nth(1);
    let res = match subcommand.as_deref() {
        Some("build") => build(),
        Some("qemu")  => build().and_then(|_| qemu()),

        _ => {
            eprintln!("Tasks:");
            eprintln!("  build - build the OS");
            Ok(())
        }
    };

    if let Err(e) = res {
        eprintln!("{}", e);
    }
}

fn build() -> Result {
    check_deps()?;

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    if Command::new(cargo)
        .arg("rustc")
        .args(&["--target", ARCH])
        .arg("--release")
        .arg("--")
        .args(&["-C", &format!("link-arg=-T{}", LINKER_FILE)])
        .args(&["-C", "target-cpu=cortex-a53"])
        .args(&["-C", "relocation-model=static"])
        .args(&["-D", "warnings"])
        // .args(&["-D", "missing_docs"])
        .status()?
        .success()
        .not()
    {
        return Err("Build failed".into());
    }

    if Command::new("rust-objcopy")
        .arg("--strip-all")
        .args(&["-O", "binary"])
        .arg(KERNEL_OUT)
        .arg(KERNEL_BIN)
        .status()?
        .success()
        .not()
    {
        return Err("Objcopy failed".into());
    }

    Ok(())
}

fn check_deps() -> Result {
    if Command::new("rust-objcopy")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success()
        .not()
    {
        eprintln!("Could not find rust-objcopy which is required for the build, would you like to install it? [y/n]");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if let "y" | "yes" = input.to_lowercase().as_ref() {
            if Command::new("cargo")
                .args(&["install", "cargo-binutils"])
                .status()?
                .success()
                .not()
            {
                return Err("Failed to install 'cargo-binutils'".into());
            }

            if Command::new("rustup")
                .args(&["component", "add", "llvm-tools-preview"])
                .status()?
                .success()
                .not()
            {
                return Err("Failed to add component 'llvm-tools-preview'".into());
            }
        }
    }

    Ok(())
}

fn qemu() -> Result {
    check_qemu()?;

    if Command::new("qemu-system-aarch64")
        .args(&["-M", "raspi3b"])
        // .args(&["-d", "in_asm"])
        .args(&["-display", "none"])
        .args(&["-serial", "null"])
        .args(&["-serial", "stdio"])
        .args(&["-kernel", KERNEL_BIN])
        .status()?
        .success()
        .not()
    {
        return Err("Qemu failed".into());
    }

    Ok(())
}

fn check_qemu() -> Result {
    if Command::new("qemu-system-aarch64")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success()
        .not()
    {
        return Err("Could not find required binary 'qemu-system-aarch64'".into());
    }

    Ok(())
}
