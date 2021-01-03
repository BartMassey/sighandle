use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use lazy_static::lazy_static;
use parking_lot::ReentrantMutex;
use crossbeam_queue::ArrayQueue;
use libc::{sigaction, siginfo_t, ucontext_t,
           c_int, sighandler_t, SA_SIGINFO, SIGINT, pid_t};

lazy_static! {
    static ref SIGQUEUE: ReentrantMutex<ArrayQueue<pid_t>> =
        ReentrantMutex::new(ArrayQueue::new(2));
}

static mut MISSES: AtomicUsize = AtomicUsize::new(0);

unsafe fn handle_sigint(
    _signum: c_int,
    siginfo: * const siginfo_t,
    _sigcontext: * const ucontext_t,
) {
    let pid = siginfo.as_ref().unwrap().si_pid();
    let records = SIGQUEUE.lock();
    records.push(pid).unwrap_or_else(|_| {
        let _ = MISSES.fetch_add(1, SeqCst);
    });
}

fn install_handler() {
    unsafe {
        // https://stackoverflow.com/a/34377103/364875
        let mut sa: sigaction = MaybeUninit::zeroed().assume_init();
        sa.sa_sigaction =
            handle_sigint
            as *const unsafe fn(c_int, *const siginfo_t, *const ucontext_t)
            as sighandler_t;
        sa.sa_flags = SA_SIGINFO;
        let result = sigaction(
            SIGINT,
            &sa as *const sigaction,
            0 as *mut sigaction,
        );
        assert_eq!(result, 0);
    }
    println!("handling SIGINT");
}

fn main() {
    install_handler();
    let records = SIGQUEUE.lock();
    let mut _count = 0;
    for _ in 0..(1u64 << 63) {
        if let Some(pid) = records.pop() {
            let misses = unsafe { MISSES.load(SeqCst) };
            println!("{} ({})", pid, misses);
        }
        _count += 1;
    }
}
