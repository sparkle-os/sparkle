ENTRY(start)

SECTIONS {
	. = 1M; /* load the kernel at 1MiB above 0x0 */

	.boot : {
		/* ensure the multiboot header is at the start of the binary */
		KEEP(*(.multiboot_header))
	}

	.text : {
		/* Kernel code */
		*(.text .text.*) /* .text.* ensures that all small .text fragments are linked together */
	}

	/* ensure small fragments in .rodata and .rel.ro are linked together */
	.rodata : {
		*(.rodata .rodata.*)
	}
	.data.rel.ro : {
		*(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
	}
}
