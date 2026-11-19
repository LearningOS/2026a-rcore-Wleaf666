//! Process management syscalls
// use core::pat;

// use riscv::addr::{VirtAddr, VirtualAddress};

// use riscv::addr::VirtAddr;

use crate::mm::{PTEFlags, PageTable, VirtAddr};
use crate::task::get_syscall_count;
use crate::task::{mmap,munmap};
use crate::mm::MapPermission;

use crate::timer::get_time_us;
use crate::{
    mm::translated_byte_buffer,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let time_sec = us / 1_000_000;
    let time_usec = us % 1_000_000;

    let mut src = [0u8; 16];
    src[..8].copy_from_slice(&time_sec.to_ne_bytes());
    src[8..].copy_from_slice(&time_usec.to_ne_bytes());

    let buffers = translated_byte_buffer(current_user_token(), _ts as *const u8, src.len());

    let mut copied = 0;

    for bytes in buffers {
        let end = copied + bytes.len();
        bytes.copy_from_slice(&src[copied..end]);
        copied = end;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            let va = VirtAddr::from(_id);
            let offset = va.page_offset();
            let vpn = va.floor();

            let page_table = PageTable::from_token(current_user_token());
            let Some(pte) = page_table.translate(vpn)else{
                return -1;
            };
            if !pte
                .flags()
                .contains(PTEFlags::V | PTEFlags::U | PTEFlags::R)
            {
                return -1;
            }

            pte.ppn().get_bytes_array()[offset] as isize
        }
        1 => {
            let va = VirtAddr::from(_id);
            let offset = va.page_offset();
            let vpn = va.floor();

            let page_table = PageTable::from_token(current_user_token());
            let Some(pte) = page_table.translate(vpn)else{
                return -1;
            };
            if !pte
                .flags()
                .contains(PTEFlags::V | PTEFlags::U | PTEFlags::W)
            {
                return -1;
            }

            pte.ppn().get_bytes_array()[offset] = _data as u8;
            0
        }
        2 => get_syscall_count(_id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _port&!0x7!=0 || _port&0x7==0{
        return -1
    }
    let permission = MapPermission::from_bits_truncate(((_port as u8)<<1)|(1<<4));
    let _end = VirtAddr::from(_start+_len);
    let _start = VirtAddr::from(_start);
    if !_start.aligned(){
        return -1;
    }
    mmap(_start, _end, permission)
}


// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let _end = VirtAddr::from(_start+_len);
    let _start = VirtAddr::from(_start);
    if !_start.aligned(){
        return -1;
    }
    munmap(_start, _end,)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
