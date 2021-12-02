use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use libc::{sighandler_t, c_int, c_void, _exit, clone, kill, pause, prctl, sigemptyset, sigset_t, sigwait, write, 
    STDOUT_FILENO, PR_SET_KEEPCAPS  };
use nix::sys::signal::{
    sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal, SIGCHLD, SIGCONT, SIGSTOP, SIGUSR1,
};

use nix::sched::{self, CloneFlags};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{getpid, getppid, Pid};

use std::fs;



/// signal handling functions

extern "C" fn handle_sigusr1(_: c_int) {
    print_signal_safe("[clone child] Received Parent signal!\n");
}

extern "C" fn handle_sigchld(_: c_int) {
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

// extern fn handler(_: c_int) {}

// fn get_handler() -> sighandler_t {
//     handler as extern fn(c_int) as *mut c_void as sighandler_t
// }

/// allocate an array and new a "ip" process
fn child() -> isize {
    println!(
        "[clone child] Hello from child process with pid: {} and parent pid:{}",
        getpid(),
        getppid()
    );

    unsafe {
        prctl(PR_SET_KEEPCAPS, 1, 0, 0, 0);
    }
    println!("[clone child ] aft prctl");


    let output = Command::new("cat")
    // .arg("/sys/fs/cgroup/foo")
    .arg("/sys/fs/cgroup/foo/memory.max")
    .output()
    .expect("failed to execute process");


    if let Ok(s) = String::from_utf8(output.stdout) {
        println!("{}", s);
    }


    // mount -t cgroup2 none /mnt/cgroup2

    let output = Command::new("mount")
    .arg("-t")
    .arg("cgroup2")
    .arg("none")
    .arg("/sys/fs/cgroup")
    .output()
    .expect("failed to execute process");

    println!("[clone child ] after mount");

    Command::new("mkdir")
    .arg("-p")
    .arg("/sys/fs/cgroup/cg1")
    .output()
    .expect("failed to execute process");

    println!("[clone child ] after mkdir");


    let output = Command::new("ls")
    .arg("/sys/fs/cgroup/cg1")
    .output()
    .expect("failed to execute process");

    println!("[clone child ] aft ls cg1 process");

    if let Ok(s) = String::from_utf8(output.stdout) {
        println!("{}", s);
    }

    let output = Command::new("ls")
    .arg("-la")
    .arg("/sys/fs/cgroup/foo/cgroup.procs")
    .output()
    .expect("failed to execute process");

    println!("[clone child ] aft ls foo process");

    if let Ok(s) = String::from_utf8(output.stdout) {
        println!("{}", s);
    }    

    sleep(Duration::from_secs(2));

    // // set signal handler for pause
    // let sig_action = SigAction::new(
    //     SigHandler::Handler(handle_sigusr1),
    //     SaFlags::empty(),
    //     SigSet::empty(),
    // );

    // if let Err(err) = unsafe { sigaction(SIGUSR1, &sig_action) } {
    //     panic!("[clone child] sigaction() failed: {}", err);
    // };
    // println!("[clone child] Wait for signal from parent");
    // // wait for signal
    // unsafe {
    //     pause();
    // }


    println!("[clone child ] before read memory limit of child process");



    // set memory limit of child process
    let pid_string = (i32::from(getpid())).to_string();
    println!("pid_string is {}", pid_string);

    // fs::write("/sys/fs/cgroup/cg1/cgroup.procs", pid_string).expect("Unable to write file");

    // let data = fs::read_to_string("/sys/fs/cgroup/cg1/cgroup.procs").expect("Unable to read file");
    // println!("[clone child ] read cgroup.procs get {}", data);

    fs::write("/sys/fs/cgroup/foo/cgroup.procs", pid_string).expect("Unable to write file");
    let data = fs::read_to_string("/sys/fs/cgroup/foo/cgroup.procs").expect("Unable to read file");
    println!("[clone child ] read cgroup.procs get {}", data);


    // println!("[clone child] Signal was delivered - pause is over");

    println!("[clone child] Try to allocate big array");
    let _v = Box::new([0i32; 600]);
    println!("[clone child] Yeah, get my array memory successfully!");

    Command::new("ip")
        .arg("link")
        .spawn()
        .expect("ip command failed to start");

        sleep(Duration::from_secs(1));

    0 // return 0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let wasm_bytes = std::fs::read("add.wasm")?;

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

    // // set memory limit of child process
    // let pid_string = (i32::from(child_pid)).to_string();

    // fs::write("/sys/fs/cgroup/foo/cgroup.procs", pid_string).expect("Unable to write file");

    // let data = fs::read_to_string("/sys/fs/cgroup/foo/cgroup.procs").expect("Unable to read file");
    // println!("[main] read cgroup.procs get {}", data);

    // // send signal to child process
    // println!("SIGUSR1 child_pid.as_raw() = {}", child_pid.as_raw());
    // unsafe {
    //     kill(child_pid.as_raw(), SIGUSR1 as i32); // resume the child process
    // }

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
