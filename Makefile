arch := x86_64
kernel := build/$(arch)/kernel.bin
iso := build/$(arch)/os.iso
rs_target := $(arch)-sparkle
rs_kernel := target/$(rs_target)/debug/libsparkle_os.a

asm_src := $(wildcard src/arch/$(arch)/bload/*.asm)
asm_obj := $(patsubst src/arch/$(arch)/bload/%.asm, build/$(arch)/bload/%.o, $(asm_src))
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
qemu_flags := -serial mon:stdio -monitor vc

.PHONY: all clean run iso doc

all: $(kernel)

clean:
	rm -r build/ target/

check:
	RUST_TARGET_PATH=$(shell pwd) xargo check --target $(rs_target)

run: $(iso)
	qemu-system-x86_64 $(qemu_flags) -cdrom $(iso) -s

run-trif: $(iso)
	qemu-system-x86_64 $(qemu_flags) -cdrom $(iso) -no-reboot -d int -s

debug: $(iso)
	qemu-system-x86_64 $(qemu_flags) -cdrom $(iso) -s -S

iso: $(iso)

$(iso): $(kernel)
	mkdir -p build/isofiles/boot/grub
	cp $(kernel) build/isofiles/boot/kernel.bin
	cp $(grub_cfg) build/isofiles/boot/grub
	grub-mkrescue -o $(iso) build/isofiles
	rm -r build/isofiles

$(kernel): $(asm_obj) $(rs_kernel)
	ld -n --gc-sections -T $(linker_script) -o $(kernel) $^

.PHONY: $(rs_kernel) # always run xargo
$(rs_kernel):
	RUST_TARGET_PATH=$(shell pwd) xargo build --target $(rs_target)

build/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	nasm -felf64 $< -o $@

doc:
	cargo rustdoc --lib -- --document-private-items
