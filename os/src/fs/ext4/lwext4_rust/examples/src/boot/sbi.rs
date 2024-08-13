use core::arch::asm;

pub fn console_putchar(ch: usize){
	sbi_call(SBI_CONSOLE_PUTCHAR, ch, 0, 0);
}

pub fn console_getchar() -> isize {
	return sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0);
}

pub fn console_putchar_u8(ch: u8){
	let ret: isize;
	//let arg0: char = ch as char;
	let arg0: u8 = ch;
	let arg1: usize = 0;
	let arg2: usize = 0;
	let which: usize = 1; //SBI_ECALL_CONSOLE_PUTCHAR
	unsafe{
		asm!("ecall",
		     lateout("x10") ret,
		     in("x10") arg0, in("x11") arg1, in("x12") arg2, in("x17") which
             );
	}
}

fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> isize{
	let ret: isize;
	unsafe{
        asm!("ecall",
             lateout("x10") ret,
             in("x10") arg0, in("x11") arg1, in("x12") arg2, in("x17") which
            );
	}
	ret
}

pub fn set_timer(stime_value: u64){
	#[cfg(target_pointer_width = "32")]
	sbi_call(SBI_SET_TIMER, stime_value as usize, (stime_value >> 32), 0);

	#[cfg(target_pointer_width = "64")]
	sbi_call(SBI_SET_TIMER, stime_value as usize, 0, 0);
}
pub fn clear_ipi(){
	sbi_call(SBI_CLEAR_IPI, 0, 0, 0);
}

pub fn send_ipi(sipi_value: usize){
	sbi_call(SBI_SEND_IPI, sipi_value, 0, 0);
}

pub fn set_s_insn(entry: usize){
	sbi_call(SBI_SET_SINSN, entry, 0, 0);
}

pub fn shutdown(){
	sbi_call(SBI_SHUTDOWN, 0, 0, 0);
}

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;
const SBI_SET_SINSN: usize = 100;
