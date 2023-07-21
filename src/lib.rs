use gag::Redirect;
use std::arch::global_asm;
use std::env::current_exe;
use std::fs::File;
use std::mem::MaybeUninit;
use std::process::Command;
use std::process::Stdio;
use std::ptr::addr_of_mut;
use std::thread;
use widestring::u16cstr;
use widestring::U16CString;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

macro_rules! forward {
    ($($name:expr => $ord:expr),+$(,)?) => {
        $(
            global_asm!(concat!(
                ".section .drectve\n.ascii \"-export:",
                $name,
                "=zlibwapi_orig.",
                $name,
                ",@",
                $ord,
                "\0\""
            ));
        )+
    };
}

forward!(
    "unzClose" => 62,
    "unzCloseCurrentFile" => 72,
    "unzGetCurrentFileInfo" => 64,
    "unzGetFilePos" => 100,
    "unzGetGlobalInfo" => 63,
    "unzGoToFilePos" => 101,
    "unzGoToFirstFile" => 65,
    "unzGoToNextFile" => 66,
    "unzOpen" => 61,
    "unzOpenCurrentFile" => 67,
    "unzReadCurrentFile" => 68,
);

pub use nvapi::*;
pub mod nvapi {
    #![allow(nonstandard_style)]
    #![allow(unused)]

    include! { concat!(env!("OUT_DIR"), "/bindings.rs") }
}

#[no_mangle]
unsafe extern "system" fn DllMain(_: HMODULE, reason: u32, _: usize) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(main);
    }

    true
}

fn main() {
    // NOTE: Writing to stdout is a terrible idea, this was originally gonna be a separate program that would be opened by `DllMain` but I managed to get it to work here. But I'm surprised this doesn't immediately panic! This should be changed to using `libc_print`.

    let mut log = File::create("fix.log").unwrap();
    Redirect::stdout(log).unwrap();

    let mut session = MaybeUninit::<NvDRSSessionHandle>::uninit();

    assert_eq!(unsafe { NvAPI_Initialize() }, 0);
    assert_eq!(
        unsafe { NvAPI_DRS_CreateSession(addr_of_mut!(session).cast()) },
        0,
        "failed to create session",
    );

    // SAFETY: Does not contain NULL
    let mut profile_name = unsafe { U16CString::from_str_unchecked("SpaceEngine 0.980 FIX") };
    let mut profile_name_array = [0u16; 2048];

    for (i, byte) in profile_name.as_slice().iter().enumerate() {
        profile_name_array[i] = *byte;
    }

    // SAFETY: Depends on `NvAPI_DRS_CreateSession` to initialize it
    let session = unsafe { session.assume_init() };
    let mut profile = MaybeUninit::<NvDRSProfileHandle>::uninit();

    let result = unsafe {
        NvAPI_DRS_FindProfileByName(
            session,
            profile_name.as_mut_ptr(),
            addr_of_mut!(profile).cast(),
        )
    };

    let extension_limit_id = 0x20FF7493 as NvU32;
    let mut extension_limit = NVDRS_SETTING {
        version: make_nvapi_version!(NVDRS_SETTING, 1),
        settingId: extension_limit_id,
        settingType: _NVDRS_SETTING_TYPE_NVDRS_DWORD_TYPE,
        settingLocation: _NVDRS_SETTING_LOCATION_NVDRS_CURRENT_PROFILE_LOCATION,
        isCurrentPredefined: 0,
        isPredefinedValid: 0,
        ..Default::default()
    };

    match result {
        0 => {}
        -163 => {
            println!("profile is missing, creating new...");

            assert_eq!(
                unsafe {
                    NvAPI_DRS_CreateProfile(
                        session,
                        &mut NVDRS_PROFILE {
                            version: make_nvapi_version!(NVDRS_PROFILE, 1),
                            profileName: profile_name_array,
                            ..Default::default()
                        },
                        addr_of_mut!(profile).cast(),
                    )
                },
                0,
                "failed to create profile",
            );

            println!("created profile! name: {}", profile_name.display());
        }
        _ => panic!("encountered generic error while looking for profile: {result}"),
    }

    // SAFETY: Depends on `NvAPI_DRS_CreateProfile` or `NvAPI_DRS_FindProfileByName` to initialize
    // it. We diverge if there are any errors so this should be fine.
    let profile = unsafe { profile.assume_init() };

    assert_eq!(
        unsafe { NvAPI_DRS_GetSetting(session, profile, extension_limit_id, &mut extension_limit) },
        0,
        "failed to get Extension string limit!",
    );

    // SAFETY: Type is DWORD
    if unsafe { extension_limit.__bindgen_anon_1.u32PredefinedValue } == 0xA474 {
        println!(
            "Extension string limit is already set to `0xA474`; no further changes are needed"
        );
    } else {
        println!("updating extension string limit...");

        extension_limit.__bindgen_anon_1.u32PredefinedValue = 0xA474;
        extension_limit.__bindgen_anon_2.u32CurrentValue = 0xA474;

        assert_eq!(
            unsafe { NvAPI_DRS_SetSetting(session, profile, &mut extension_limit) },
            0,
            "failed to set Extension string limit!",
        );
    }

    let mut app_name_array = [0u16; 2048];

    for (i, byte) in current_exe()
        .unwrap()
        .to_str()
        .unwrap()
        .encode_utf16()
        .collect::<Vec<_>>()
        .iter()
        .enumerate()
    {
        app_name_array[i] = *byte;
    }

    println!("adding app to profile...");
    assert_eq!(
        unsafe {
            NvAPI_DRS_CreateApplication(
                session,
                profile,
                &mut NVDRS_APPLICATION {
                    version: make_nvapi_version!(NVDRS_APPLICATION, 4),
                    isPredefined: 0,
                    appName: app_name_array,
                    ..Default::default()
                },
            )
        },
        0,
        "failed to add app {}",
        current_exe().unwrap().display()
    );
    println!("done!");

    println!("saving...");
    assert_eq!(unsafe { NvAPI_DRS_SaveSettings(session) }, 0);
    println!("done!");

    assert_eq!(
        unsafe { NvAPI_DRS_DestroySession(session) },
        0,
        "failed to destroy session. :("
    );
    assert_eq!(unsafe { NvAPI_Unload() }, 0);
}

#[macro_export]
macro_rules! make_nvapi_version {
    ($type:path,$ver:expr) => {
        ::std::mem::size_of::<$type>() as u32 | ($ver as u32) << 16
    };
}
