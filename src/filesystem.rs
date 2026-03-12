use nix::mount::{MsFlags, mount};

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
    mount!(mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None::<&str>
    ))?;
    Ok(())
}
