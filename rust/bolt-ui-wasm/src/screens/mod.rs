// EW2 REUSE: Copied from bolt-ui/src/screens/mod.rs — identical.
pub mod connect;
pub mod transfer;
pub mod verify;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Connect,
    Transfer,
    Verify,
}
