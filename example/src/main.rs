use std::process::Command;
use std::path::Path;
use std::error::Error;

// A flag to compile the projects in release mode
const COMPILE_RELEASE_MODE: bool = false;
// The path to the bootloader the kernel should be packed with
const BOOTLOADER_PATH: &str = "../bootloader";
// The name of the bootloader exectuable
const BOOTLOADER_EXE_NAME: &str = "potato_loader.exe";

const KERNEL_EXE_NAME: &str = "example";

fn cargo_build<P: AsRef<Path>>(directory: P, target_dir: P)
    -> Result<(), Box<dyn Error>>
{
    let directory =
        directory.as_ref().to_str().unwrap_or("Failed to convert path");
    let target_dir =
        target_dir.as_ref().to_str().unwrap_or("Failed to convert path");

    // Issue the cargo build command for the kernel
    // TODO(patrik): Fix pls, ugly code
    let status =
        if COMPILE_RELEASE_MODE {
            Command::new("cargo")
                .current_dir(&directory)
                .args(&[
                    "build",
                    "--release",

                    // Set the target directory to the kernel build directory
                    "--target-dir",
                    target_dir
                ])
                .status()?
                .success()
        } else {
            Command::new("cargo")
                .current_dir(&directory)
                .args(&[
                    "build",

                    // Set the target directory to the kernel build directory
                    "--target-dir",
                    target_dir
                ])
                .status()?
                .success()
        };

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Failed to cargo build".into());
    }

    Ok(())
}

fn compile_kernel() -> Result<(), Box<dyn Error>> {
    // The path to the kernel
    let kernel_path =
        Path::new("kernel")
            .canonicalize()?;

    // The build directory the kernel should be built to
    let kernel_build_path =
        Path::new("build")
            .join("kernel")
            .canonicalize()?;

    // Debug print the kernel path and the kernel build path
    println!("Kernel Path: {:?}", kernel_path);
    println!("Kernel Build Path: {:?}", kernel_build_path);

    cargo_build(kernel_path, kernel_build_path)?;

    Ok(())
}

fn compile_bootloader() -> Result<(), Box<dyn Error>> {
    // Construct the bootloader path
    let bootloader_path =
        Path::new(BOOTLOADER_PATH)
            .canonicalize()?;

    // Construct the build target where the bootloader files
    // will be compiled to
    let bootloader_build_path =
        Path::new("build")
            .join("bootloader")
            .canonicalize()?;

    println!("Bootloader Path: {:?}", bootloader_path);
    println!("Bootloader Build Path: {:?}", bootloader_build_path);

    cargo_build(bootloader_path, bootloader_build_path)?;

    // All good, the bootloader should be successfully built
    Ok(())
}

fn dd_create<P: AsRef<Path>>(input: P, output: P,
                             block_size: u64, count: u64)
    -> Result<(), Box<dyn Error>>
{
    let input =
        input.as_ref().to_str().unwrap_or("Failed to convert path");
    let output =
        output.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("dd")
            .args(&[
                format!("if={}", input),
                format!("of={}", output),
                format!("bs={}", block_size),
                format!("count={}", count),
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn dd_merge<P: AsRef<Path>>(input: P, output: P,
                            block_size: u64, count: u64, start: u64)
    -> Result<(), Box<dyn Error>>
{
    let input =
        input.as_ref().to_str().unwrap_or("Failed to convert path");
    let output =
        output.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("dd")
            .args(&[
                format!("if={}", input),
                format!("of={}", output),
                format!("bs={}", block_size),
                format!("count={}", count),
                format!("seek={}", start),
                "conv=notrunc".to_string()
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn parted<P, I, S>(image_path: P, other_args: I)
    -> Result<(), Box<dyn Error>>
    where P: AsRef<Path>,
          I: IntoIterator<Item = S>,
          S: AsRef<std::ffi::OsStr>,
{
    let image_path =
        image_path.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("parted")
            .args(&[
                image_path,
                "-s",
                "-a",
                "minimal"
            ])
            .args(other_args)
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn mformat<P: AsRef<Path>>(image_path: P) -> Result<(), Box<dyn Error>> {
    let image_path =
        image_path.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("mformat")
            .args(&[
                "-i",
                image_path,

                "-h",
                "32",

                "-t",
                "32",

                "-n",
                "64",

                "-c",
                "1"
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn mmd<P: AsRef<Path>>(image_path: P, directory: &str)
    -> Result<(), Box<dyn Error>>
{
    let image_path =
        image_path.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("mmd")
            .args(&[
                "-i",
                image_path,

                format!("::{}", directory).as_str()
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn mcopy<P: AsRef<Path>>(image_path: P, source: P, destination: &str)
    -> Result<(), Box<dyn Error>>
{
    let image_path =
        image_path.as_ref().to_str().unwrap_or("Failed to convert path");

    let source =
        source.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("mcopy")
            .args(&[
                "-i",
                image_path,

                source,
                format!("::{}", destination).as_str()
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn objcopy_binary<P: AsRef<Path>>(input: P, output: P)
    -> Result<(), Box<dyn Error>>
{
    let input =
        input.as_ref().to_str().unwrap_or("Failed to convert path");
    let output =
        output.as_ref().to_str().unwrap_or("Failed to convert path");

    let status =
        Command::new("objcopy")
            .args(&[
                "-O",
                "binary",

                input,
                output
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn package() -> Result<(), Box<dyn Error>> {
    let uefi_image_count = 93750;
    let uefi_image = "build/uefi_image.img";
    dd_create("/dev/zero", uefi_image, 512, uefi_image_count)?;

    parted(uefi_image, &["mklabel", "gpt"])?;
    // 93716
    parted(uefi_image, &["mkpart", "EFI", "FAT16", "2048s",
                         format!("{}s", 93750 - 2048).as_str()])?;
    parted(uefi_image, &["toggle", "1", "boot"])?;

    let part_image_count = uefi_image_count - 2048; // 91669;
    let part_image = "build/part.img";
    dd_create("/dev/zero", part_image, 512, part_image_count)?;

    mformat(part_image)?;
    mmd(part_image, "/EFI")?;
    mmd(part_image, "/EFI/boot")?;

    let bootloader_exe_path =
        Path::new("build")
            .join("bootloader")
            .join("x86_64-pc-windows-gnu");

    let bootloader_exe_path = if COMPILE_RELEASE_MODE {
        bootloader_exe_path.join("release")
    } else {
        bootloader_exe_path.join("debug")
    };

    let bootloader_exe_path = bootloader_exe_path.join(BOOTLOADER_EXE_NAME);
    let bootloader_exe_path = bootloader_exe_path.canonicalize()?;

    println!("Bootloader Exe Path: {:?}", bootloader_exe_path);

    let kernel_path =
        Path::new("build")
            .join("kernel")
            .join("x86_64-kernel");

    let kernel_path = if COMPILE_RELEASE_MODE {
        kernel_path.join("release")
    } else {
        kernel_path.join("debug")
    };

    let kernel_path = kernel_path.join(KERNEL_EXE_NAME);
    let kernel_path = kernel_path.canonicalize()?;

    let kernel_binary_path =
        Path::new("build")
            .join("test.bin");

    println!("Kernel Path: {:?}", kernel_path);

    objcopy_binary(kernel_path, kernel_binary_path)
        .expect("Failed to objcopy kernel");

    mcopy(part_image, "startup.nsh", "/EFI/boot")?;
    mcopy(part_image, "options.txt", "/EFI/boot")?;
    mcopy(part_image, "build/test.bin", "/EFI/boot")?;
    mcopy(part_image, bootloader_exe_path.to_str().unwrap(),
          "/EFI/boot/main.efi")?;

    dd_merge(part_image, uefi_image, 512, part_image_count, 2048)?;
    Ok(())
}

fn run_qemu() -> Result<(), Box<dyn Error>> {
    let uefi_image = "build/uefi_image.img";
    let memory_size = "1G";

    let ovmf_code_bin = "third_party/ovmf_bins/OVMF_CODE-pure-efi.fd";
    let ovmf_vars_bin = "third_party/ovmf_bins/OVMF_VARS-pure-efi.fd";

    let status =
        Command::new("qemu-system-x86_64")
            .args(&[
                "-drive",
                format!("file={}", uefi_image).as_str(),

                "-m",
                memory_size,

                "-cpu",
                "qemu64",

                "-drive",
                format!("if=pflash,format=raw,unit=0,file={},readonly=on",
                        ovmf_code_bin).as_str(),

                "-drive",
                format!("if=pflash,format=raw,unit=1,file={}",
                        ovmf_vars_bin).as_str(),

                "-net",
                "none"
            ])
            .status()?
            .success();

    // Check the status for a success otherwise return a error
    if !status {
        return Err("Status is false".into());
    }

    Ok(())
}

fn main() {
    // Create the all the build directories
    std::fs::create_dir_all("build")
        .expect("Failed to create build directory");
    std::fs::create_dir_all("build/kernel")
        .expect("Failed to create bootloader build directory");
    std::fs::create_dir_all("build/bootloader")
        .expect("Failed to create kernel build directory");

    // TODO(patrik): Check if the tools are on the system

    // TODO(patrik): Compile the kernel
    println!("---------------------------------------------");
    compile_kernel().expect("Failed to compile the kernel");

    // TODO(patrik): Compile the bootloader
    println!("---------------------------------------------");
    compile_bootloader().expect("Failed to compile bootloader");

    // TODO(patrik): Package the kernel and bootloader
    println!("---------------------------------------------");
    package().expect("Failed to package");

    println!("---------------------------------------------");

    run_qemu().expect("Failed to run qemu");
}
