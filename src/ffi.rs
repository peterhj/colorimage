#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub mod gckimg {
include!(concat!(env!("OUT_DIR"), "/gckimg_bind.rs"));
}
