use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use libc::{_exit, kill, pause, prctl, sigemptyset, sigset_t, sigwait, write, STDOUT_FILENO};
use nix::sys::signal::{
    sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal, SIGCHLD, SIGCONT, SIGSTOP, SIGUSR1,
};
use std::ffi::c_void;

use nix::sched::{self, CloneFlags};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{getpid, getppid, Pid};

use std::fs;

/// signal handling functions

extern "C" fn handle_sigusr1(_: libc::c_int) {
    print_signal_safe("[clone child] Received Parent signal!\n");
}

extern "C" fn handle_sigchld(_: libc::c_int) {
    print_signal_safe("[main] What a surprise! Got SIGCHLD!\n");
    match waitpid(Pid::from_raw(-1), None) {
        Ok(_) => {
            print_signal_safe("[main] Child exited.\n");
            print_signal_safe("[main] Bye Bye!\n");
            exit_signal_safe(0);
        }
        Err(_) => {
            print_signal_safe("[main] waitpid() failed.\n");
            exit_signal_safe(1);
        }
    }
}

/// allocate an array and new a "ip" process
fn child() -> isize {
    println!(
        "[clone child] Hello from child process with pid: {} and parent pid:{}",
        getpid(),
        getppid()
    );

    // set signal handler for pause
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigusr1),
        SaFlags::empty(),
        SigSet::empty(),
    );

    if let Err(err) = unsafe { sigaction(SIGUSR1, &sig_action) } {
        panic!("[clone child] sigaction() failed: {}", err);
    };
    println!("[clone child] Wait for signal from parent");
    // wait for signal
    unsafe {
        pause();
    }

    println!("[clone child] Signal was delivered - pause is over");

    println!("[clone child] Try to allocate big array");
    let _v = Box::new([0i32; 6000]);
    println!("[clone child] Yeah, get my array memory successfully!");

    Command::new("ip")
        .arg("link")
        .spawn()
        .expect("ip command failed to start");

    0 // return 0
}

fn main() {
    const STACK_SIZE: usize = 1024 * 1024;
    let ref mut stack = [0; STACK_SIZE];

    let flags = CloneFlags::CLONE_NEWUSER
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWCGROUP;

    let child_pid = sched::clone(Box::new(child), stack, flags, Some(Signal::SIGCHLD as i32)) // without SIGCHLD signal, waitpid gives error "ECHILD: No child processes"
        .expect("Failed to spawn the child");

    println!(
        "[main] I am the parent process with pid: {} and I cloned a child with PID {}.",
        getpid(),
        child_pid
    );

    // set signal handler for child termination
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigchld),
        SaFlags::empty(),
        SigSet::empty(),
    );

    if let Err(err) = unsafe { sigaction(SIGCHLD, &sig_action) } {
        panic!("[main] sigaction() failed: {}", err);
    };

    // set memory limit of child process
    let pid_string = (i32::from(child_pid)).to_string();

    fs::write("/sys/fs/cgroup/foo/cgroup.procs", pid_string).expect("Unable to write file");

    let data = fs::read_to_string("/sys/fs/cgroup/foo/cgroup.procs").expect("Unable to read file");
    println!("[main] read cgroup.procs get {}", data);

    // send signal to child process
    println!("SIGUSR1 child_pid.as_raw() = {}", child_pid.as_raw());
    unsafe {
        kill(child_pid.as_raw(), SIGUSR1 as i32); // resume the child process
    }

    // infinite loop
    println!("[main] I'll be doing my own stuff...");
    loop {
        println!("[main] Do my own stuff.");
        // ... replace sleep with the payload
        sleep(Duration::from_millis(500));
    }
}

fn print_signal_safe(s: &str) {
    unsafe {
        write(STDOUT_FILENO, s.as_ptr() as (*const c_void), s.len());
    }
}

fn exit_signal_safe(status: i32) {
    unsafe {
        _exit(status);
    }
}
