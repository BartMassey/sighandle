use std::process::abort;
use std::ptr;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_queue::ArrayQueue;
use libc::{sigaction, siginfo_t, ucontext_t,
           c_int, sighandler_t, SA_SIGINFO, SIGINT, pid_t};

static mut SIGQUEUE: * mut ArrayQueue<pid_t> = ptr::null_mut();
static mut MISSES: AtomicUsize = AtomicUsize::new(0);

unsafe fn handle_sigint(
    _signum: c_int,
    siginfo: * const siginfo_t,
    _sigcontext: * const ucontext_t,
) {
    let pid = siginfo.as_ref().unwrap_or_else(|| abort()).si_pid();
    (*SIGQUEUE).push(pid).unwrap_or_else(|_| {
        let _ = MISSES.fetch_add(1, Ordering::SeqCst);
    });
}

fn install_handler() {
    unsafe {
        // Set up pid queue.
        SIGQUEUE = Box::leak(Box::new(ArrayQueue::new(1)));

        // Set up signal handler.
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
            ptr::null_mut(),
        );
        assert_eq!(result, 0);
    }
    println!("handling SIGINT");
}

fn main() {
    install_handler();
    for _ in 0..(1u64 << 63) {
        let pid = unsafe { (*SIGQUEUE).pop() };
        if let Some(pid) = pid {
            let misses = unsafe { MISSES.load(Ordering::Relaxed) };
            println!("{} ({})", pid, misses);
        }
    }
}
