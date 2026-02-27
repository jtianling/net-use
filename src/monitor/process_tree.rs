use std::collections::HashSet;
use std::ffi::CStr;
use std::mem;

use anyhow::Result;

use crate::types::ProcessInfo;

unsafe extern "C" {
    fn proc_listallpids(buffer: *mut libc::c_void, buffersize: i32) -> i32;
    fn proc_listchildpids(ppid: i32, buffer: *mut libc::c_void, buffersize: i32) -> i32;
    fn proc_pidpath(pid: i32, buffer: *mut libc::c_void, buffersize: u32) -> i32;
    fn proc_name(pid: i32, buffer: *mut libc::c_void, buffersize: u32) -> i32;
}

pub fn list_all_pids() -> Result<Vec<i32>> {
    let count = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Ok(Vec::new());
    }

    let mut pids: Vec<i32> = vec![0; (count as usize) + 64];
    let actual = unsafe {
        proc_listallpids(
            pids.as_mut_ptr() as *mut libc::c_void,
            (pids.len() * mem::size_of::<i32>()) as i32,
        )
    };
    if actual <= 0 {
        return Ok(Vec::new());
    }

    pids.truncate(actual as usize);
    Ok(pids)
}

pub fn get_pid_path(pid: i32) -> Option<String> {
    let mut buf = [0u8; 4096];
    let ret = unsafe { proc_pidpath(pid, buf.as_mut_ptr() as *mut libc::c_void, buf.len() as u32) };
    if ret <= 0 {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
    Some(c_str.to_string_lossy().into_owned())
}

pub fn get_pid_name(pid: i32) -> Option<String> {
    let mut buf = [0u8; 256];
    let ret = unsafe { proc_name(pid, buf.as_mut_ptr() as *mut libc::c_void, buf.len() as u32) };
    if ret <= 0 {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
    Some(c_str.to_string_lossy().into_owned())
}

pub fn list_child_pids(ppid: i32) -> Vec<i32> {
    let count = unsafe { proc_listchildpids(ppid, std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }

    let mut pids: Vec<i32> = vec![0; (count as usize) + 16];
    let actual = unsafe {
        proc_listchildpids(
            ppid,
            pids.as_mut_ptr() as *mut libc::c_void,
            (pids.len() * mem::size_of::<i32>()) as i32,
        )
    };
    if actual <= 0 {
        return Vec::new();
    }

    let actual_count = actual as usize / mem::size_of::<i32>();
    pids.truncate(actual_count);
    pids.retain(|&p| p > 0);
    pids
}

pub fn collect_descendants(root_pid: i32) -> HashSet<i32> {
    let mut result = HashSet::new();
    let mut queue = vec![root_pid];

    while let Some(pid) = queue.pop() {
        result.insert(pid);
        for child in list_child_pids(pid) {
            if !result.contains(&child) {
                queue.push(child);
            }
        }
    }

    result
}

pub fn get_process_info(pid: i32) -> Option<ProcessInfo> {
    let name = get_pid_name(pid)?;
    Some(ProcessInfo { pid, name })
}

pub fn find_pids_by_name(name: &str) -> Result<Vec<i32>> {
    let all_pids = list_all_pids()?;
    let mut matched = Vec::new();
    for pid in all_pids {
        if let Some(proc_name) = get_pid_name(pid)
            && proc_name == name
        {
            matched.push(pid);
        }
    }
    Ok(matched)
}

pub fn find_pids_by_executable_path_prefix(prefix: &str) -> Result<Vec<i32>> {
    let all_pids = list_all_pids()?;
    let mut matched = Vec::new();
    for pid in all_pids {
        if let Some(path) = get_pid_path(pid)
            && path.starts_with(prefix)
        {
            matched.push(pid);
        }
    }
    Ok(matched)
}
