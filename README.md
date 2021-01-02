<h1 align="center">
  <br>
  Potato Bootloader
  <br>
</h1>

UEFI Bootloader written in rust for custom 64-bit x86-64 kernels compiled in to ELF64

## Key Features

## Build from source 

<b>For now we only support building the project on `Unix` Systems</b>

1. Dependencies
  * `git`
  * `Rust` Tested with version `rustc 1.51.0-nightly`
  * `lld-link` The LLVM Linker
  * `dd`
  * `parted`
  * `mtools`
  * `qemu` with the x86-64 emulation

2. Clone the [Source](https://github.com/Nanoteck137/potato) with `git`
```bash
git clone https://github.com/Nanoteck137/potato
cd potato
```

3. Change the Rust channel to `nightly`
```bash
rustup default nightly
```

4. Add the `rust-src` component
```bash
rustup component add rust-src
```

5. Install the `x86_64-pc-windows-gnu` target 
```bash
rustup target add x86_64-pc-windows-gnu
```

### Build only the Bootloader
```bash
cd bootloader
cargo build
```

### Build and run the Example kernel (builds the bootloader)
```bash
cd example
cargo run
```