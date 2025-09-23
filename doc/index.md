# Advanced Programming in the UNIX® Environment, 4th Edition

## Overview

### Key Additions over 3rd Edition

- io_uring → new I/O chapter.
- Namespaces + cgroups → containerization.
- fd-based interfaces (eventfd, timerfd, signalfd).
- Security: seccomp, Landlock, capabilities.
- eBPF & perf → observability.
- Filesystem events + extended attributes.
- Networking updates: batching syscalls, modern TCP

## Part I — Introduction & Basics

1.	Unix System Overview
    (Updated to cover Linux dominance, macOS/BSD divergence, containerization as “the new Unix.”)
2.	Unix Standardization and Implementations
    (POSIX 2008, SUSv4, glibc extensions, deprecation of legacy APIs.)
3.	File I/O
    (Keep classic read/write/lseek, add pread/pwrite, splice, sendfile.)
4.	Files and Directories
    (Introduce statx(2), extended attributes, ACLs.)
5.	Standard I/O Library
    (Update fmemopen, open_memstream, note lack of fd equivalents.)

## Part II — Processes & Threads

6.	Process Environment
    (Environment variables, getauxval, secure handling with modern secure_getenv.)
7.	Process Control
    (Fork/exec, add posix_spawn, Linux clone3, namespaces.)
8.	Process Relationships
    (Sessions, controlling terminals, containers vs init.)
9.	Signals
    (Classic APIs + signalfd, realtime signals, timer signals.)
10.	Threads
    (POSIX Threads, thread affinity, barriers, C11 <threads.h>.)
11.	Thread Control
    (Futexes, robust mutexes, atomics vs pthread sync.)
12.	Process Groups, Sessions, Jobs
    (Modern shells, job control with new signals.)

## Part III — Advanced I/O & IPC

13.	Advanced I/O
	- Nonblocking I/O
	- select/poll/epoll evolution
	- New chapter material: io_uring (design + usage)
	- eventfd, timerfd, inotify/fanotify
14.	Interprocess Communication (Classic)
    - (Pipes, FIFOs, SysV IPC, POSIX message queues.)
    - [SysV marked “legacy.”]
15.	Shared Memory & Synchronization
	- POSIX shm & semaphores
	- memfd_create, shm_open, userfaultfd (advanced)
16.	Sockets & Networking
    - IPv6 as default
	- sendmmsg/recvmmsg, SO_REUSEPORT
	- TCP Fast Open, MPTCP
	- Basic SCTP intro
17.	Advanced Sockets
	- Ancillary data, SCM_RIGHTS, passing fds
	- eBPF hooks in networking stack

## Part IV — Filesystems, Devices & Security

18.	Terminal I/O
    - (Still relevant: canonical vs raw mode.)
19.	Pseudo-terminals
    - (Add posix_openpt, container TTY handling.)
20.	Daemons & Modules
    - (Daemon creation, supervision systems, modern init systems mention.)
21.	Advanced Filesystem Interfaces
    - inotify and fanotify
	- Extended attributes (xattr), ACLs
	- fallocate, O_TMPFILE
22.	Device I/O
    - (Character/block devices, ioctl, sysfs/procfs exposure.)
23.	Security APIs
	- Capabilities
	- prctl flags
	- seccomp-bpf
	- Landlock LSM
	- Password handling (getrandom, Argon2 via libs)

## Part V — System Programming in Practice

24.	Timers & Clocks
(clock_gettime, new clock IDs, POSIX timers, timerfd.)
25.	Resource Management
- rlimits
- cgroups v2
- Namespaces for isolation
26.	Observability & Tracing
- ptrace (classic)
- perf events
- eBPF tracing
- strace, ltrace, modern observability tools
27.	Case Studies
- Building a multiprocess server (classic Stevens examples, updated with epoll/io_uring)
- Container sandbox demo (namespaces + seccomp)
- Logging and tracing with modern syscalls

## Appendices

- System Calls by Category (updated to 2025).
- POSIX vs Linux extensions.
- Quick reference: io_uring operations.
- Secure coding checklist.