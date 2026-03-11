use std::ffi::CString;

use clap::{ Parser};
use nix::{libc::{c_char, execv}, sched::{CloneFlags, unshare}};

#[derive(Parser, Debug)]
#[command(version, about, long_about = "Run a process in a sandboxxed environment with configurable isolation.", trailing_var_arg = true)]
struct Args {
    /// The program to execute, will be resolved by $PATH
    process: String,

    /// The arguments to pass to the programe.
    args: Vec<String>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {   
    let args = Args::parse();

    unshare(CloneFlags::empty())?;
    let pid = unsafe { 
        // use the ffi cstring because it will ensure that the null terminator is provided.
        let c_process_name = CString::new(args.process.as_str())?;
        let name: *const c_char =  c_process_name.as_ptr();

        let c_args: Vec<CString> = std::iter::once(c_process_name.clone())
                                        .chain(args.args.iter().map(|arg| 
                                            CString::new(arg.as_str()).unwrap()
                                        ))
                                        .collect();

        let mut argv:  Vec<*const c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();
        // according to the execv man pages, the argument vector must end with a null terminator.
        argv.push(std::ptr::null());
    
        execv(name, argv.as_ptr()) 
    };

    println!("Started process: {}", pid);
    return Ok(());
}
