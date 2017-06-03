#![feature(lang_items)]
#![no_std]

extern crate rlibc;

#[no_mangle]
pub extern fn kernel_main() {
	//////////// !!! WARNING !!! ////////////
	// WE HAVE AN EXTREMELY SMALL STACK    //
	// AND NO GUARD PAGE                   //
	/////////////////////////////////////////

	loop {}	
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! {loop{}}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
	// we should hlt here
	loop {}
}