// Copyright (c) 2017 Intel Corporation. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate nvpair_sys;
use nvpair_sys::*;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn zfs_type_dataset() -> ::std::os::raw::c_int {
    let zfs_type_t(v) = zfs_type_t_ZFS_TYPE_FILESYSTEM | zfs_type_t_ZFS_TYPE_VOLUME | zfs_type_t_ZFS_TYPE_SNAPSHOT;

    v as ::std::os::raw::c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::os::raw::{c_int, c_void};
    use std::ffi::{CStr, CString};

    fn create_libzfs_handle() -> *mut libzfs_handle_t {
        unsafe { libzfs_init() }
    }

    fn destroy_libzfs_handle(h: *mut libzfs_handle_t) {
        unsafe { libzfs_fini(h) }
    }

    #[test]
    fn open_close_handle() {
        let h = create_libzfs_handle();
        destroy_libzfs_handle(h);
    }

    #[test]
    fn pool_search_import_list_export() {
        let h = create_libzfs_handle();

        let (nvl, nvp) = unsafe {
            thread_init();
            let nvl = zpool_find_import(h, 0, ptr::null_mut());
            thread_fini();

            let nvp = nvlist_next_nvpair(nvl, ptr::null_mut());

            (nvl, nvp)
        };

        let name = unsafe {
            CStr::from_ptr(nvpair_name(nvp))
                .to_owned()
                .into_string()
                .unwrap()
        };
        assert_eq!(name, "test");

        let mut config = ptr::null_mut();
        let mut elem = ptr::null_mut();

        unsafe {
            loop {
                elem = nvlist_next_nvpair(nvl, elem);

                if elem == ptr::null_mut() {
                    break;
                }

                assert_eq!(nvpair_value_nvlist(elem, &mut config), 0);
            }
        }

        let code = unsafe { zpool_import(h, config, ptr::null(), ptr::null_mut()) };
        assert_eq!(code, 0);

        unsafe { nvlist_free(nvl) };

        unsafe extern "C" fn callback(handle: *mut zpool_handle_t, state: *mut c_void) -> c_int {
            let s = CStr::from_ptr((*handle).zpool_name.as_ptr());
            let s = s.to_owned().into_string().unwrap();

            let state = &mut *(state as *mut Vec<String>);
            state.push(s);

            zpool_close(handle);

            0
        }

        let mut state: Vec<String> = Vec::new();
        let state_ptr: *mut c_void = &mut state as *mut _ as *mut c_void;

        let code = unsafe { zpool_iter(h, Some(callback), state_ptr) };

        assert_eq!(code, 0);
        assert_eq!(state, vec!["test"]);

        let zpool_h = unsafe {
            let poolName = CString::new("test").unwrap();

            zpool_open_canfail(h, poolName.as_ptr())
        };

        unsafe { zpool_export(zpool_h, boolean::B_FALSE, ptr::null_mut()) };

        destroy_libzfs_handle(h);
    }
}