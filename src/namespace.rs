use std::fs;

use nix::unistd::Pid;

use crate::error::SandboxError;

macro_rules! ns {
    ($expr:expr) => {
        $expr.map_err(|e| SandboxError::Namespace {
            source: e,
            call: stringify!($expr),
        })
    };
}

pub fn write_uid_map(child_pid: Pid) -> Result<(), SandboxError> {
    ns!(fs::write(
        format!("/proc/{}/uid_map", child_pid),
        "0 1000 1"
    ))?;
    Ok(())
}

pub fn write_gid_map(child_pid: Pid) -> Result<(), SandboxError> {
    // We must deny since the CAP_SETGID permissions may not be set.
    // This will prevent unpriviledged users from modifying groups to gain access.
    ns!(fs::write(format!("/proc/{}/setgroups", child_pid), "deny"))?;

    // Map GID 0 inside namespace → GID 1000 outside
    ns!(fs::write(
        format!("/proc/{}/gid_map", child_pid),
        "0 1000 1"
    ))?;
    Ok(())
}
