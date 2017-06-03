global long_start

section .text
bits 64
long_start:
	; load 0 into all data segment registers
	mov ax, 0
	mov ss, ax
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax

	extern kernel_main
	call kernel_main

	; print `EXIT` to screen
	mov rax, 0x2f542f492f582f45
	mov qword [0xb8000], rax

.halted
	hlt
	jmp .halted ; in case some interrupt kicks us out of hlt, jump back and hlt again