ENTRY(start)

SECTIONS {
	. = 1M; /* load the kernel at 1MiB above 0x0 */

	.boot : {
		/* ensure the multiboot header is at the start of the binary */
		KEEP(*(.multiboot_header))
	}

	.text : {
		/* Kernel code */
		*(.text)
	}
}