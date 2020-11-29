TARGET=debug

OVMF_BIN_DIR=third_party/ovmf_bins
OVMF_CODE_BIN=$(OVMF_BIN_DIR)/OVMF_CODE-pure-efi.fd
OVMF_VARS_BIN=$(OVMF_BIN_DIR)/OVMF_VARS-pure-efi.fd

EFI_EXE=target/x86_64-pc-windows-gnu/$(TARGET)/potato_loader.exe
UEFI_IMAGE=obj/uefi.img
PART_IMAGE=obj/part.img

build:
	cargo build

image: build 
	mkdir -p obj/

	dd if=/dev/zero of=$(UEFI_IMAGE) bs=512 count=93750

	parted $(UEFI_IMAGE) -s -a minimal mklabel gpt
	parted $(UEFI_IMAGE) -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $(UEFI_IMAGE) -s -a minimal toggle 1 boot

	dd if=/dev/zero of=$(PART_IMAGE) bs=512 count=91669
	mformat -i $(PART_IMAGE) -h 32 -t 32 -n 64 -c 1

	mmd -i $(PART_IMAGE) ::/EFI
	mmd -i $(PART_IMAGE) ::/EFI/boot

	mcopy -i $(PART_IMAGE) $(EFI_EXE) ::/EFI/boot/main.efi
	mcopy -i $(PART_IMAGE) startup.nsh ::
	mcopy -i $(PART_IMAGE) options.txt ::/EFI/boot/options.txt
	mcopy -i $(PART_IMAGE) test.bin ::/EFI/boot/test.bin

	dd if=$(PART_IMAGE) of=$(UEFI_IMAGE) bs=512 count=91669 seek=2048 conv=notrunc

run: 
	qemu-system-x86_64 -drive file=$(UEFI_IMAGE) -m 256M -cpu qemu64 -drive if=pflash,format=raw,unit=0,file="$(OVMF_CODE_BIN)",readonly=on -drive if=pflash,format=raw,unit=1,file="$(OVMF_VARS_BIN)" -net none

clean:
	rm -rf obj/ target/ 
