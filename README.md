# sighandle: Rust signal handler with PID queueing
Bart Massey

This code corresponds to `/u/Michal_Vaner`'s Reddit
[post](https://www.reddit.com/r/rust/comments/kpi3vk/dark_side_of_posix_apis/)
talking about issues getting UNIX signal handling to process
PIDs.

The code consists of a custom signal handler for `SIGINT`
that puts the sending process ID in a non-allocating
lock-free queue (`crossbeam_queue::ArrayQueue`) and a driver
that polls the queue for PIDs.

The program can be hit with a `SIGINT` while running by
hitting the interrupt key. There is also a shell script,
`trigger.sh`, that throws 1000 `SIGINT`s at the program as
fast as possible.

When the driver receives a `PID` from the queue, it will
print it along with a count of the number of `PID`s
reportedly flushed by the signal handler because of lack of
queue space. This latter number is very much advisory — also
keep in mind that the OS kernel will itself compress
consecutive signals to some degree.

With small queue sizes — 1-3 on my box with the program
compiled debug — this program will eventually hang if
`trigger.sh` is run enough times. While I haven't yet
analyzed the hang, it appears to me offhand to be either a
bug in `ArrayQueue` or a failure to meet the `ArrayQueue`
use case in a way I don't understand. I have not replicated
the hang with larger queue lengths or when compiled in
release mode.

-----

This work is released under the "MIT License". Please see
the file `LICENSE` in this distribution for license terms.
