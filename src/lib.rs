#![cfg_attr(not(windows), forbid(unsafe_code))]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod about;
pub mod app;
pub mod cli;
pub mod config;
pub mod i18n;
pub mod messaging;
pub mod notify;
pub mod tray;
