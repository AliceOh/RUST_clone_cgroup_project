use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::ForkResult::{Child, Parent};
use nix::unistd::{fork, getpid, getppid};

fn main() {
    let child_pid = match fork() {
        Ok(Child) => {
            //////////////////////
            //      child       //
            //////////////////////
            println!(
                "[fork child] I'm alive! My PID is {} and PPID is {}.",
                getpid(),
                getppid()
            );

            println!("[fork child] I'm gonna sleep for a while and then just exit...");
            sleep(Duration::from_secs(2));
            exit(0);
        }

        Ok(Parent { child, .. }) => {
            println!(
                "[main] I am the parent process with pid: {} and I forked a child with PID {}.",
                getpid(),
                child
            );
            child  // return child Pid
        }

        Err(err) => {
            panic!("[main] fork() failed: {}", err);
        }
    };
    println!("[main] I'll be doing my own stuff while waiting for the child termination...");
    loop {
        match waitpid(child_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {
                println!("[main] Child is still alive, do my own stuff while waiting.");
                // ... replace sleep with the payload
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
