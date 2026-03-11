use std::{ffi::NulError, io};

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("namespace setup failed in '{call}': {source}")]
    Namespace {
        source: nix::Error,
        call: &'static str,
    },
    #[error("sandbox spawn failed in '{call}': {source}")]
    Spawn {
        source: nix::Error,
        call: &'static str,
    },
    #[error("sandbox setup failed in '{call}': {source}")]
    Setup {
        source: io::Error,
        call: &'static str,
    },
    #[error("invalid configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("internal configuration failue: {0}")]
    ConfigInternal(#[from] NulError),
    #[error("exec failed in '{call}' : {source}")]
    Exec {
        source: nix::Error,
        call: &'static str,
    },
}
