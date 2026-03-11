use std::{
    ffi::CString,
    os::fd::{AsFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
};

use nix::{
    sched::{CloneFlags, clone},
    sys::wait::waitpid,
    unistd::{Pid, execv, pipe, read, write},
};

use crate::{
    error::SandboxError,
    namespace::{write_gid_map, write_uid_map},
};

macro_rules! setup {
    ($expr:expr) => {
        $expr.map_err(|e| SandboxError::Namespace {
            source: e,
            call: stringify!($expr),
        })
    };
}

macro_rules! spawn {
    ($expr:expr) => {
        $expr.map_err(|e| SandboxError::Spawn {
            source: e,
            call: stringify!($expr),
        })
    };
}

macro_rules! exec {
    ($expr:expr) => {
        $expr.map_err(|e| SandboxError::Exec {
            source: e,
            call: stringify!($expr),
        })
    };
}

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
    pub fn new(command: &str) -> Result<Self, SandboxError> {
        let c_command_str = CString::new(command)?;
        Ok(Self {
            command: c_command_str.clone(),
            argv: vec![c_command_str],
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

    pub fn build(self) -> Result<Sandbox, SandboxError> {
        if self.command.as_bytes().is_empty() {
            return Err(SandboxError::InvalidConfig(
                "Invalid Configuration: No command to run.",
            ));
        }
        Ok(Sandbox {
            command: self.command,
            argv: self.argv,
            network: self.network,
        })
    }
}

impl Sandbox {
    pub fn run(&self) -> Result<(), SandboxError> {
        let child_pid = self.spawn_child()?;
        let status = spawn!(waitpid(child_pid, None))?;
        Ok(())
    }

    fn spawn_child(&self) -> Result<Pid, SandboxError> {
        let (read_fd, write_fd) = setup!(pipe())?;
        let raw_read_fd: RawFd = read_fd.into_raw_fd();
        let raw_write_fd: RawFd = write_fd.into_raw_fd();
        let mut stack = vec![0u8; 1024 * 1024];

        let mut flags =
            CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS;
        if !self.network {
            flags |= CloneFlags::CLONE_NEWNET;
        }

        let child_pid = unsafe {
            spawn!(clone(
                Box::new(|| {
                    // Wait until the parent signlas that the namespace is complete.
                    let write_fd = OwnedFd::from_raw_fd(raw_write_fd);
                    drop(write_fd);
                    self.setup_child(raw_read_fd).unwrap();

                    0
                }),
                &mut stack,
                flags,
                Some(nix::sys::signal::Signal::SIGCHLD as i32),
            ))?
        };
        let read_fd = unsafe { OwnedFd::from_raw_fd(raw_read_fd) };
        drop(read_fd);

        write_uid_map(child_pid)?;
        write_gid_map(child_pid)?;

        let write_fd = unsafe { OwnedFd::from_raw_fd(raw_write_fd) };
        setup!(write(write_fd.as_fd(), &[1]))?;
        drop(write_fd);
        return Ok(child_pid);
    }

    fn setup_child(&self, raw_read_fd: RawFd) -> Result<(), SandboxError> {
        let read_fd = unsafe { OwnedFd::from_raw_fd(raw_read_fd) };
        let mut buf = [0u8; 1];
        setup!(read(read_fd.as_fd(), &mut buf))?;
        drop(read_fd);

        exec!(execv(&self.command, &self.argv))?;
        unreachable!("Child should have execv, no longer able to proceed in current program.")
    }
}
