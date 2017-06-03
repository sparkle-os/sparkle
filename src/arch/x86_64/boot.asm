global start
extern long_start

section .text
bits 32
start:
	; point the esp register to the top of our stack
	; (the stack grows downwards)
	mov esp, stack_top

	call check_multiboot
	call check_cpuid
	call check_long_mode

	call setup_ptables
	call enable_paging

	; load the 64-bit gdt
	lgdt [gdt64.pointer]
	jmp gdt64.code:long_start

	; print OK
	; mov dword [0xb8000], 0x2f4b2f4f

	hlt ; halt

; Checks that we were actually loaded by a Multiboot-compatible system
check_multiboot:
	cmp eax, 0x36d76289
	jne .no_multiboot
	ret
.no_multiboot:
	mov al, "m"
	jmp error

; Checks that we have a CPUID-enabled processor
check_cpuid:
	; Check if CPUID is supported by attempting to flip the ID bit (bit 21) in
	; the FLAGS register. If we can flip it, CPUID is available.

	; Copy FLAGS in to EAX via stack
	pushfd
	pop eax

	; Copy to ECX as well for comparing later on
	mov ecx, eax

	; Flip the ID bit
	xor eax, 1 << 21

	; Copy EAX to FLAGS via the stack
	push eax
	popfd

	; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
	pushfd
	pop eax

	; Restore FLAGS from the old version stored in ECX (i.e. flipping the ID bit
	; back if it was ever flipped).
	push ecx
	popfd

	; Compare EAX and ECX. If they are equal then that means the bit wasn't
	; flipped, and CPUID isn't supported.
	xor eax, ecx
	jz .no_cpuid
	ret
.no_cpuid:
	mov al, "c"
	jmp error

; Checks if long mode is supported
check_long_mode:
	; test if extended processor info in available
	mov eax, 0x80000000    ; implicit argument for cpuid
	cpuid                  ; get highest supported argument
	cmp eax, 0x80000001    ; it needs to be at least 0x80000001
	jb .no_long_mode       ; if it's less, the CPU is too old for long mode

	; use extended info to test if long mode is available
	mov eax, 0x80000001    ; argument for extended processor info
	cpuid                  ; returns various feature bits in ecx and edx
	test edx, 1 << 29      ; test if the LM-bit is set in the D-register
	jz .no_long_mode       ; If it's not set, there is no long mode
	ret
.no_long_mode:
	mov al, "2"
	jmp error

setup_ptables:
	; p4[0] -> p3
	mov eax, p3_table
	or eax, 0b11 ; present + writable
	mov [p4_table], eax

	; p3[0] -> p2
	mov eax, p2_table
	or eax, 0b1 ; present + writable
	mov [p3_table], eax

	; map each p2 entry to a 2mib hugepage 
	mov ecx, 0
.map_p2:
	; p2[ecx] -> huge_page{@2MiB*ecx}
	mov eax, 0x200000  ; 2MiB
	mul ecx			   ; start address
	or eax, 0b10000011 ; present + writable + huge
	mov [p2_table + ecx*8], eax ; map ecx-th entry

	inc ecx ; increase counter
	cmp ecx, 512 ; whole table is mapped if ecx == 512
	jne .map_p2 ; else map the next entry

	ret

enable_paging:
	; load P4 to cr3 register (cpu uses this to access the P4 table)
	mov eax, p4_table
	mov cr3, eax

	; enable PAE-flag in cr4 (Physical Address Extension)
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; set the long mode bit in the EFER MSR (model specific register)
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; enable paging in the cr0 register
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	ret

; Print `ERR: ` + error code to screen and then HLTs
; lovingly ripped off from Phil Oppermann's os.phil-opp.com
; parameter: error code (ascii) in al
error:
	mov dword [0xb8000], 0x4f524f45
	mov dword [0xb8004], 0x4f3a4f52
	mov dword [0xb8008], 0x4f204f20
	mov byte  [0xb800a], al
	hlt

;;; Smol stack (64 bytes) just to make stuff work atm
section .bss
align 4096
p4_table:
	resb 4096
p3_table:
	resb 4096
p2_table:
	resb 4096
p1_table:
	resb 4096
stack_bottom:
	resb 64
stack_top:

section .rodata
gdt64:
    dq 0 ; zero entry
.code: equ $ - gdt64 ; new
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64