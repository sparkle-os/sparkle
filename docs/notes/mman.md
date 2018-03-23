# kernelspace
## page frame allocator
* allocate spans of frames when we need to map data
  * don't necessarily need contiguous frames, except for DMA or hardware.
    perhaps have a way to request different 'layouts' when asking for n frames?

## kernel heap allocator
* note: is _not_ the userspace allocator! see [here](#splitting-kernel-and-userspace-alloc)

## memory initialization on boot
* recieve segment list from Multiboot info pointer given to us by GRUB
* call `memory::init(boot_info)`, returning a `MemoryController`
  * compute `(kernel_start, kernel_end)` and `(multiboot_start, multiboot_end)` addresses
  * initialize a `FrameAllocator`, passing the previous ranges in so it can avoid them,
    as well as other memory areas as indicated by the Multiboot header
  * enable the NXE bit (NO_EXECUTE pages), and the WRPROT bit (disable writes to non-WRITABLE pages)
  * remap the kernel
    * create a scratch page for a temporary page remapping
    * create a new P4 table
    * temporarily map the new table; identity map the kernel, VGA buffer, and multiboot info into it
    * switch to the new table
    * create a guard page in place of the old P4 table's page
  * create a heap, allocating frames and mapping them in
  * initialize the heap; switch global Rust allocator to the new heap
  * create an allocator for stacks (for creating, eg, ISR stacks), and store it in the `MemoryController`.
  * return the `MemoryController`.

# userspace
## splitting kernel and userspace alloc
* userspace has its own alloc server.
