
#![allow(unused_parens, non_camel_case_types)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate stdweb_derive;

pub mod webgl2 {
    include!(concat!(env!("OUT_DIR"), "/webgl2_stdweb.rs"));
}