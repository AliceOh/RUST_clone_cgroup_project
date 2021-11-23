use std::time::Duration;
use std::thread::sleep;
use std::process::Command;
use std::fs;
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{getpid, getppid};
use nix::sched::{self, CloneFlags};

/// allocate an array and new a "ip" process
fn child() -> isize {

    println!("[clone child] Hello from child process with pid: {} and parent pid:{}",   getpid(), getppid());

    println!("[clone child] Try to allocate big array");
    let _v = Box::new([0i32; 600]);
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


    let child_pid = sched::clone(Box::new(child), stack, flags, 
                Some(Signal::SIGCHLD as i32)) // without SIGCHLD signal, waitpid gives error "ECHILD: No child processes"
                .expect("Failed to spawn the child");
    
    println!("[main] I am the parent process with pid: {} and I cloned a child with PID {}.", getpid(), child_pid);

    let pid_string = (i32::from(child_pid)).to_string();

    // println!("Wait 10 seconds for the child process to up, before changing cgroup");
    // sleep(Duration::from_secs(10));

    fs::write("/sys/fs/cgroup/foo/cgroup.procs", pid_string).expect("Unable to write file");

    let data = fs::read_to_string("/sys/fs/cgroup/foo/cgroup.procs").expect("Unable to read file");
    
    println!("[main] read cgroup.procs get {}", data);

    // println!("Wait 10 seconds for the child process to run, after changing cgroup");
    // sleep(Duration::from_secs(10));

     
    println!("[main] I'll be doing my own stuff while waiting for the child {} termination...", child_pid);
    loop {
        match waitpid(child_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {
                println!("[main] Child is still alive, do my own stuff while waiting.");
                // ... replace sleep with the work to be done in main
                sleep(Duration::from_millis(500));
            }

            Ok(status) => {
                println!("[main] Child exited with status {:?}.", status);
                break;
            }

            Err(err) => panic!("[main] waitpid() failed: {}", err),
        }
    }

    println!("[main] Bye Bye!");
}
