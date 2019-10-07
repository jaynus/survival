#![feature(custom_attribute)]
#![deny(clippy::pedantic, clippy::all)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

pub mod loaders;

pub mod components;
pub mod debug;
pub mod initializers;
pub mod renderer;
pub mod states;
pub mod systems;
pub mod ui;
