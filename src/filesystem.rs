use std::fs;

use nix::mount::{MntFlags, MsFlags, mount, umount2};

use crate::error::SandboxError;

macro_rules! mount {
    ($expr:expr) => {
        $expr.map_err(|e| SandboxError::Mount {
            source: e,
            call: stringify!($expr),
        })
    };
}

pub fn make_mounts_private() -> Result<(), SandboxError> {
    // First make mounts slave to disconnect from parent's shared propagation,
    // then make them private. Going directly to private from shared is denied
    // inside a user namespace.
    mount!(mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_SLAVE,
        None::<&str>
    ))?;
    mount!(mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None::<&str>
    ))?;
    Ok(())
}

pub fn bind_mount(source: &str, target: &str) -> Result<(), SandboxError> {
    (fs::create_dir_all(target)).map_err(|e| SandboxError::Setup {
        source: e,
        call: "create_dir_all(target)",
    })?;
    mount!(mount(
        Some(source),
        target,
        None::<&str>,
        MsFlags::MS_BIND,
        None::<&str>
    ))?;
    Ok(())
}

pub fn bind_mount_readonly(source: &str, target: &str) -> Result<(), SandboxError> {
    (fs::create_dir_all(target)).map_err(|e| SandboxError::Setup {
        source: e,
        call: "create_dir_all(target)",
    })?;
    mount!(mount(
        Some(source),
        target,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_RDONLY,
        None::<&str>
    ))?;
    Ok(())
}

pub fn mount_tempfs(target: &str) -> Result<(), SandboxError> {
    fs::create_dir_all(target).map_err(|e| SandboxError::Setup {
        source: e,
        call: "create_dir_all_tempfs",
    })?;

    mount!(mount(
        Some("tmpfs"),
        target,
        Some("tmpfs"),
        MsFlags::empty(),
        None::<&str>
    ))?;
    Ok(())
}

pub fn mount_proc(target: &str) -> Result<(), SandboxError> {
    fs::create_dir_all(target).map_err(|e| SandboxError::Setup {
        source: e,
        call: "create_dir_all_proc",
    })?;

    mount!(mount(
        Some("proc"),
        target,
        Some("proc"),
        MsFlags::empty(),
        None::<&str>
    ))?;
    Ok(())
}

pub fn unmount_old_root(target: &str) -> Result<(), SandboxError> {
    mount!(umount2(target, MntFlags::MNT_DETACH))?;
    fs::remove_dir(target).map_err(|e| SandboxError::Setup {
        source: e,
        call: "remove_dir(old_root)",
    })?;
    Ok(())
}
