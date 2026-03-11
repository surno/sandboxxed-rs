use std::{
    error::Error,
    ffi::CString,
    os::fd::{AsFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
};

use nix::{
    sched::{CloneFlags, clone},
    sys::wait::waitpid,
    unistd::{Pid, close, execv, pipe, read, write},
};

pub struct Sandbox {
    command: CString,
    argv: Vec<CString>,
    network: bool,
}

pub struct SandboxBuilder {
    command: CString,
    argv: Vec<CString>,
    network: bool,
}

impl SandboxBuilder {
    pub fn new(command: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            command: CString::new(command)?,
            argv: vec![CString::new(command)?],
            network: true,
        })
    }

    pub fn network(mut self, network: bool) -> Self {
        self.network = network;
        self
    }

    pub fn add_args(mut self, argv: &Vec<String>) -> Self {
        self.argv
            .extend(argv.iter().map(|a| CString::new(a.as_str()).unwrap()));
        self
    }

    pub fn build(self) -> Result<Sandbox, Box<dyn Error>> {
        if self.command.as_bytes().is_empty() {
            return Err("Invalid Configuration: No command to run.".into());
        }
        Ok(Sandbox {
            command: self.command,
            argv: self.argv,
            network: self.network,
        })
    }
}

impl Sandbox {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let child_pid = self.spawn_child()?;
        let status = waitpid(child_pid, None)?;
        Ok(())
    }

    fn spawn_child(&self) -> Result<Pid, Box<dyn Error>> {
        let (read_fd, write_fd) = pipe()?;
        let raw_read_fd: RawFd = read_fd.into_raw_fd();
        let raw_write_fd: RawFd = write_fd.into_raw_fd();
        let mut stack = vec![0u8; 1024 * 1024];

        let mut flags =
            CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS;
        if !self.network {
            flags |= CloneFlags::CLONE_NEWNET;
        }

        let child_pid = unsafe {
            clone(
                Box::new(|| {
                    let write_fd = OwnedFd::from_raw_fd(raw_write_fd);
                    drop(write_fd);
                    self.setup_child(raw_read_fd).unwrap();

                    0
                }),
                &mut stack,
                flags,
                Some(nix::sys::signal::Signal::SIGCHLD as i32),
            )?
        };
        let read_fd = unsafe { OwnedFd::from_raw_fd(raw_read_fd) };
        drop(read_fd);
        let write_fd = unsafe { OwnedFd::from_raw_fd(raw_write_fd) };
        write(write_fd.as_fd(), &[1])?;
        drop(write_fd);
        return Ok(child_pid);
    }

    fn setup_child(&self, raw_read_fd: RawFd) -> Result<(), Box<dyn Error>> {
        let read_fd = unsafe { OwnedFd::from_raw_fd(raw_read_fd) };
        let mut buf = [0u8; 1];
        read(read_fd.as_fd(), &mut buf)?;
        drop(read_fd);

        execv(&self.command, &self.argv)?;
        unreachable!("Child should have execv, no longer able to proceed in current program.")
    }
}
