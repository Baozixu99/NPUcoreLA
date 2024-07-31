#![no_std]
#![no_main]
use user_lib::{exit, exec, fork, wait, waitpid, yield_, shutdown};

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    exit(main());
}

#[no_mangle]
fn main() -> i32 {
    let path = "/bin/bash\0";
    let environ = [
        "SHELL=/bash\0".as_ptr(),
        "PWD=/\0".as_ptr(),
        "LOGNAME=root\0".as_ptr(),
        "MOTD_SHOWN=pam\0".as_ptr(),
        "HOME=/root\0".as_ptr(),
        "LANG=C.UTF-8\0".as_ptr(),
        "TERM=vt220\0".as_ptr(),
        "USER=root\0".as_ptr(),
        "SHLVL=0\0".as_ptr(),
        "OLDPWD=/root\0".as_ptr(),
        "PS1=\x1b[1m\x1b[32mNPUcore+\x1b[0m:\x1b[1m\x1b[34m\\w\x1b[0m\\$ \0".as_ptr(),
        "_=/bin/bash\0".as_ptr(),
        "PATH=/:/bin\0".as_ptr(),
        "LD_LIBRARY_PATH=/\0".as_ptr(),
        core::ptr::null(),
    ];
    let schedule_text: &str= "
    echo bbb > lat_sig\0
    ./time-test\0
    ./busybox_testcode.sh\0
    ./lua_testcode.sh\0
    ./lmbench_testcode.sh\0
    ./libctest_testcode.sh\0
    ./libc-bench\0
    ";
    let mut exit_code: i32 = 0;
    for line in schedule_text.lines(){
        let argv = [
            path.as_ptr(),
            "-c\0".as_ptr(),
            line.as_ptr(),
            core::ptr::null(),
        ];
        let pid = fork();
        if pid == 0 {
            exec(path, &argv, &environ);
        } else {
            waitpid(pid as usize, &mut exit_code);
        }
    }
    shutdown();
    0
    // let mut exit_code: i32 = 0;
    // let pid = fork();
    // if pid == 0 {
    //     exec(path, &[path.as_ptr() as *const u8, "-c\0".as_ptr(), "./run-all.sh\0".as_ptr(), core::ptr::null()], &environ);
    //     //exec(path, &[path.as_ptr() as *const u8, core::ptr::null()], &environ);
    // } else {
    //     waitpid(pid as usize, &mut exit_code);
    // }
    // shutdown();
    // 0
}
