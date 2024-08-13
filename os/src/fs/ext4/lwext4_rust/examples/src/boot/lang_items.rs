use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> !{
	println!("{}", info);
	crate::boot::sbi::shutdown();
	unreachable!()
}

#[no_mangle]
pub extern "C" fn abort() -> !{
	panic!("abort!");
}

