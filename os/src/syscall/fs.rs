const FD_STDOUT: usize = 1;

// YOUR JOB: 修改 sys_write 使之通过测试
const STACK_SIZE: usize = 0x1000;

unsafe fn r_sp() -> usize {
    let mut sp: usize;
    asm!("mv {}, sp", out(reg) sp);
    sp
}

// unsafe fn r_data() -> usize {
//     let mut data: usize;
//     asm!("la {}, start_data", out(reg) data);
//     data
// }

unsafe fn stack_range() -> (usize, usize) {
    let sp = r_sp();
    let top = (sp + STACK_SIZE - 1) & (!(STACK_SIZE - 1));
    (top - STACK_SIZE, top)
}

pub fn in_range(small: (usize, usize), big: (usize, usize)) -> bool {
    small.0 >= big.0 && small.1 <= big.1
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // println!("OS :  sys_write");
    // extern "C" {
    //     fn start_data();
    //     fn end_data();
    // }
    match fd {
        FD_STDOUT => {
            // let (botton, top) = unsafe { stack_range() };
            // if !in_range((buf as usize, buf as usize + len), (botton, top))
            //     || !in_range((buf as usize, buf as usize + len), (start_data as usize, end_data as usize))
            // {
            //     return -1;
            // }
            // println!("data : {}", unsafe { r_data() });
            // println!("1");
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            // println!("2");
            print!("{}", str);
            len as isize
        }
        _ => {
            // panic!("Unsupported fd in sys_write!");
            return -1;
        }
    }
}
