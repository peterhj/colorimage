#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub mod wrapped {
include!(concat!(env!("OUT_DIR"), "/wrapped_bind.rs"));
}
