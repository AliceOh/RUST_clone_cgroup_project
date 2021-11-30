# RUST_clone_cgroup - playing with Clone, Fork, Linux processes termination and Cgroup in Rust

The project covers the following scenarios:

- forking and awaiting the forking-generated child process termination;
- cloning with new namespaces and awaiting the cloning-generated child process termination;
- Through cloned child process delay to wait for cgroup changing, put the cloned child process into a cgroup (cgroup v2) with memory max set to 4096 byte, and child process require less than 4K bytes, expect success.
- Through cloned child process delay to wait for cgroup changing, put the cloned child process into a cgroup (cgroup v2) with memory max set to 4096 byte, and child process require more than 4K bytes, expect failure.
- Through cloned child process synchronize with parent process to wait for cgroup changing, which is to put the cloned child process into a cgroup (cgroup v2) with memory max set to 4096 byte by parent process, and child process require less than 4K bytes, expect success. (clone_wait_signal_cgroup_memory)
- Through cloned child process synchronize with parent process to wait for cgroup changing, which is to put the cloned child process into a cgroup (cgroup v2) with memory max set to 4096 byte by parent process, and child process require MORE than 4K bytes, expect child process termination by system. (clone_wait_signal_cgroup_memory_fail)

## Configuration Before running cgroup scenarios on cgroup v2
```bash
$ sudo su
# Command to set max memory in V2 cgroup as root
# mkdir -p /sys/fs/cgroup/foo
# nano  /sys/fs/cgroup/foo/memory.max 
Replace "max" with "4096"
```

## Usage
```bash
$ cargo build

# cloning with new namespaces and awaiting the cloning-generated child process termination
$ cargo run --bin clone_waitpid

# forking and awaiting the forking-generated child process termination
$ cargo run --bin fork_waitpid

# cgroup: cloning and limiting the momory of the cloned child process
$ sudo ./target/debug/clone_waitpid_cgroup_memory 

# cgroup: cloning and limiting the momory of the cloned child process, and child process require more than limit, expecting error reported: Child exited with status Signaled(Pid(31887), SIGKILL, false).
$ sudo ./target/debug/clone_waitpid_cgroup_memory_fail

# clone process and sync between child and parent process with signals. Child process wait for cgroup setting ready before proceeding to allocate memory, and signal its termination to parent process after done
$ sudo ./target/debug/clone_wait_signal_cgroup_memory

# clone process and sync between child and parent process with signals. Child process wait for cgroup setting ready before proceeding to allocate memory more than limit, and signal its termination by system.
$ sudo ./target/debug/clone_wait_signal_cgroup_memory_fail

```

