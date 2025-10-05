# eBPF

With bpftrace (user-space wrapper over bpf(2)):

```shell
sudo bpftrace -e 'tracepoint:syscalls:sys_enter_openat { printf("openat: %s\n", str(args->filename)); }'
```

-> Minimal example: logs all filenames opened by processes.
In C, you’d use bpf(2) with a BPF_PROG_TYPE_TRACEPOINT.