mod utils;

use std::{ env, io, fs };
use std::process::{ Command, Stdio };
use std::ops::Not;

use utils::*;

const TARGET: &str = "aarch64-unknown-none-softfloat";
const KERNEL_RELEASE: &str = "target/aarch64-unknown-none-softfloat/release/kernel";
const KERNEL_DEBUG: &str = "target/aarch64-unknown-none-softfloat/debug/kernel";
const KERNEL_ELF: &str = "kernel.elf";
const KERNEL_BIN: &str = "kernel.bin";
const LINKER_FILE: &str = "src/arch/link.ld";

type AnyErr = Box<dyn std::error::Error>;
type Result = std::result::Result<(), AnyErr>;

fn main() {
    let subcommand = env::args().nth(1);
    let args = env::args().skip_while(|arg| arg != "--").skip(1);
    let is_debug = env::args().find(|arg| arg == "--debug").is_some();
    let res = match subcommand.as_deref() {
        Some("build") => build(is_debug, args),
        Some("qemu")  => build(is_debug, args).and_then(|_| qemu()),
        Some("debug") => build(true, args).and_then(|_| qemu()),
        Some("gdb") => build(true, args).and_then(|_| qemu_gdb()),
        Some("clippy") => clippy(),

        _ => {
            eprintln!("usage: cargo xtask <task>");
            eprintln!("Tasks:");
            eprintln!("    build - build the OS");
            Ok(())
        }
    };

    if let Err(e) = res {
        eprintln!("{}", e);
    }
}

fn build(is_debug: bool, args: impl Iterator<Item = String>) -> Result {
    check_deps()?;

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("rustc")
       .args(&["--target", TARGET]);
    if !is_debug { cmd.arg("--release"); }
    cmd.arg("--")
       .args(&["-C", &format!("link-arg=-T{}", LINKER_FILE)])
       .args(&["-C", "target-cpu=cortex-a53"])
       .args(&["-C", "relocation-model=static"])
       .args(&["-D", "warnings"])
       .args(&["-D", "missing_docs"])
       .args(args);

    print_command(&cmd);

    if cmd
        .status()?
        .success()
        .not()
    {
        return Err("Build failed".into());
    }

    if is_debug {
        print_info(format!("Copy {KERNEL_DEBUG} -> {KERNEL_ELF}"));
        fs::copy(KERNEL_DEBUG, KERNEL_ELF)?;
    } else {
        print_info(format!("Copy {KERNEL_RELEASE} -> {KERNEL_ELF}"));
        fs::copy(KERNEL_RELEASE, KERNEL_ELF)?;
    }

    let mut cmd = Command::new("rust-objcopy");
    cmd.args(&["-O", "binary"]);
    if !is_debug { cmd.arg("--strip-all"); }
    cmd.arg(if is_debug { KERNEL_DEBUG } else { KERNEL_RELEASE })
       .arg(KERNEL_BIN);

    print_command(&cmd);

    if cmd
        .status()?
        .success()
        .not()
    {
        return Err("Objcopy failed".into());
    }

    Ok(())
}

fn qemu() -> Result {
    check_qemu()?;

    let mut qemu_cmd = qemu_cmd(KERNEL_ELF);
    print_command(&qemu_cmd);

    if qemu_cmd
        .status()?
        .success()
        .not()
    {
        return Err("Qemu failed".into());
    }

    Ok(())
}

fn qemu_gdb() -> Result {
    check_qemu()?;

    let mut qemu_cmd = qemu_cmd(KERNEL_ELF);
    qemu_cmd
        .arg("-S")
        .arg("-s");

    print_command(&qemu_cmd);
    print_info("Open gdb and connect to localhost:1234");

    if qemu_cmd
        .status()?
        .success()
        .not()
    {
        return Err("Qemu failed".into());
    }

    Ok(())
}

fn clippy() -> Result {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("clippy")
       .args(&["--target", TARGET]);

    print_command(&cmd);

    if cmd
        .status()?
        .success()
        .not()
    {
        return Err("Build failed".into());
    }

    Ok(())
}

fn qemu_cmd(fname: &str) -> Command {
    let mut qemu_cmd = Command::new("qemu-system-aarch64");
    qemu_cmd
        .args(&["-M", "raspi3b"])
        // .args(&["-d", "in_asm"])
        .args(&["-display", "none"])
        .args(&["-serial", "null"])
        .args(&["-serial", "stdio"])
        .args(&["-kernel", fname]);
    qemu_cmd
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
