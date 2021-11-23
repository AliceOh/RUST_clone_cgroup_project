# RUST_clone_cgroup - playing with Clone, Fork and Linux processes termination in Rust

The project covers the following scenarios:

- forking and awaiting the forking-generated child process termination;
- cloning with new namespaces and awaiting the cloning-generated child process termination;
- put the cloned child process into a cgroup (cgroup v2) with memory max set t0 4096 byte, and child process require less than 4K bytes, expect success.
- put the cloned child process into a cgroup (cgroup v2) with memory max set t0 4096 byte, and child process require more than 4K bytes, expect failure.

## Configuration Before running cgroup scenarios on cgroup v2
```bash
$ sudo su
# Command to set max memory in V2 cgroup as root
$ mkdir -p /sys/fs/cgroup/foo
$ nano  /sys/fs/cgroup/foo/memory.max 
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

```

