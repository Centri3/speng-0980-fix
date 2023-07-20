use std::ptr::null_mut;

pub use nvapi::*;
pub mod nvapi {
    #![allow(nonstandard_style)]
    #![allow(unused)]

    include! { concat!(env!("OUT_DIR"), "/bindings.rs") }
}

fn main() {
    let mut session = null_mut();

    assert_eq!(unsafe { NvAPI_DRS_CreateSession(&mut session) }, 0);
    assert!(!session.is_null());
    assert_eq!(unsafe { NvAPI_DRS_DestroySession(session) }, 0);

    0x20FF7493;
    // nvapi::NvAPI_DRS_GetSetting();
}
