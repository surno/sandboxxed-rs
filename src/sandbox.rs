use std::{error::Error, ffi::CString};

use nix::{
    sys::wait::{WaitStatus, waitpid},
    unistd::{ForkResult, execv, fork, getpid},
};

pub fn start_sandbox(process: &String, argv: &Vec<String>) -> Result<(), Box<dyn Error>> {
    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            println!("Child process: {}", getpid());
            // set up isolation and then execv
            // use the ffi cstring because it will ensure that the null terminator is provided.
            let c_process_name = CString::new(process.as_str())?;
            let c_argv: Vec<CString> = std::iter::once(c_process_name.clone())
                .chain(argv.iter().map(|arg| CString::new(arg.as_str()).unwrap()))
                .collect();
            execv(&c_process_name, &c_argv)?;
        }
        Ok(ForkResult::Parent { child }) => {
            // Parent
            println!("Started child: {}, parent: {}", child, getpid());
            loop {
                match waitpid(child, None) {
                    Ok(WaitStatus::Exited(pid, exit_code)) => {
                        println!("Child: {}, Exit: {}", pid, exit_code);
                        break;
                    }
                    Ok(status) => {
                        println!("Unexpected wait status: {:?}", status);
                    }
                    Err(e) => {
                        panic!("Unexpected wait from the child!: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            print!("Unable to fork into child: {}", e);
        }
    }

    Ok(())
}
