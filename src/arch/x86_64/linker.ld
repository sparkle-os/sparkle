ENTRY(start)

SECTIONS {
	. = 1M; /* load the kernel at 1MiB above 0x0 */

	/* Merged section for both the multiboot header and .rodata,
	   so we don't waste a page on the multiboot header */
	.rodata : {
		/* ensure the multiboot header is at the start of the binary */
		KEEP(*(.multiboot_header))

		/* ensure small fragments in .rodata and .rel.ro are linked together */
		*(.rodata .rodata.*)

		. = ALIGN(4K);
	}

	.text : {
		/* Kernel code */
		*(.text .text.*) /* .text.* ensures that all small .text fragments are linked together */
		. = ALIGN(4K);
	}

	.data : {
		*(.data .data.*)
		. = ALIGN(4K);
	}

	.bss : {
		*(.bss .bss.*)
		. = ALIGN(4K);
	}

	.got : {
		*(.got)
		. = ALIGN(4K);
	}

	.got.plt : {
		*(.got.plt)
		. = ALIGN(4K);
	}

	.data.rel.ro : ALIGN(4K) {
		*(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
		. = ALIGN(4K);
	}

	.gcc_except_table : ALIGN(4K) {
		*(.gcc_except_table)
		. = ALIGN(4K);
	}
}
