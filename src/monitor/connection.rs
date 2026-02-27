use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use anyhow::Result;

const PROC_PIDLISTFDS: i32 = 1;
const PROX_FDTYPE_SOCKET: u32 = 2;
const PROC_PIDFDSOCKETINFO: i32 = 3;
const INI_IPV4: u8 = 0x1;
const INI_IPV6: u8 = 0x2;
const SOCKINFO_TCP: i32 = 2;
const SOCKINFO_IN: i32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcFdInfo {
    proc_fd: i32,
    proc_fdtype: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VinfoStat {
    vst_dev: u32,
    vst_mode: u16,
    vst_nlink: u16,
    vst_ino: u64,
    vst_uid: u32,
    vst_gid: u32,
    vst_atime: i64,
    vst_atimensec: i64,
    vst_mtime: i64,
    vst_mtimensec: i64,
    vst_ctime: i64,
    vst_ctimensec: i64,
    vst_birthtime: i64,
    vst_birthtimensec: i64,
    vst_size: i64,
    vst_blocks: i64,
    vst_blksize: i32,
    vst_flags: u32,
    vst_gen: u32,
    vst_rdev: u32,
    vst_qspare: [i64; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcFileInfo {
    fi_openflags: u32,
    fi_status: u32,
    fi_offset: i64,
    fi_type: i32,
    fi_guardflags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SockbufInfo {
    sbi_cc: u32,
    sbi_hiwat: u32,
    sbi_mbcnt: u32,
    sbi_mbmax: u32,
    sbi_lowat: u32,
    sbi_flags: i16,
    sbi_timeo: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct In4In6Addr {
    i46a_pad32: [u32; 3],
    i46a_addr4: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
union InAddrUnion {
    ina_46: In4In6Addr,
    ina_6: [u8; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InSockInfo {
    insi_fport: i32,
    insi_lport: i32,
    insi_gencnt: u64,
    insi_flags: u32,
    insi_flow: u32,
    insi_vflag: u8,
    insi_ip_ttl: u8,
    rfu_1: u32,
    insi_faddr: InAddrUnion,
    insi_laddr: InAddrUnion,
    insi_v4: u8,
    insi_v6: [u8; 12],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TcpSockInfo {
    tcpsi_ini: InSockInfo,
    tcpsi_state: i32,
    tcpsi_timer: [i32; 4],
    tcpsi_mss: i32,
    tcpsi_flags: u32,
    rfu_1: u32,
    tcpsi_tp: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
union SoiProto {
    pri_in: InSockInfo,
    pri_tcp: TcpSockInfo,
    _pad: [u8; 528],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SocketInfo {
    soi_stat: VinfoStat,
    soi_so: u64,
    soi_pcb: u64,
    soi_type: i32,
    soi_protocol: i32,
    soi_family: i32,
    soi_options: i16,
    soi_linger: i16,
    soi_state: i16,
    soi_qlen: i16,
    soi_incqlen: i16,
    soi_qlimit: i16,
    soi_timeo: i16,
    soi_error: u16,
    soi_oobmark: u32,
    soi_rcv: SockbufInfo,
    soi_snd: SockbufInfo,
    soi_kind: i32,
    rfu_1: u32,
    soi_proto: SoiProto,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SocketFdInfo {
    pfi: ProcFileInfo,
    psi: SocketInfo,
}

unsafe extern "C" {
    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut libc::c_void,
        buffersize: i32,
    ) -> i32;

    fn proc_pidfdinfo(
        pid: i32,
        fd: i32,
        flavor: i32,
        buffer: *mut libc::c_void,
        buffersize: i32,
    ) -> i32;
}

pub fn list_fds(pid: i32) -> Result<Vec<(i32, u32)>> {
    let buf_size = unsafe { proc_pidinfo(pid, PROC_PIDLISTFDS, 0, std::ptr::null_mut(), 0) };
    if buf_size <= 0 {
        return Ok(Vec::new());
    }

    let count = buf_size as usize / mem::size_of::<ProcFdInfo>();
    let mut fds: Vec<ProcFdInfo> = vec![
        ProcFdInfo {
            proc_fd: 0,
            proc_fdtype: 0,
        };
        count + 16
    ];

    let actual = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDLISTFDS,
            0,
            fds.as_mut_ptr() as *mut libc::c_void,
            (fds.len() * mem::size_of::<ProcFdInfo>()) as i32,
        )
    };
    if actual <= 0 {
        return Ok(Vec::new());
    }

    let actual_count = actual as usize / mem::size_of::<ProcFdInfo>();
    Ok(fds[..actual_count]
        .iter()
        .map(|fd| (fd.proc_fd, fd.proc_fdtype))
        .collect())
}

pub fn get_socket_remote_addr(pid: i32, fd: i32) -> Result<Option<IpAddr>> {
    let mut info: SocketFdInfo = unsafe { mem::zeroed() };
    let size = mem::size_of::<SocketFdInfo>() as i32;

    let ret = unsafe {
        proc_pidfdinfo(
            pid,
            fd,
            PROC_PIDFDSOCKETINFO,
            &mut info as *mut SocketFdInfo as *mut libc::c_void,
            size,
        )
    };

    if ret < size {
        return Ok(None);
    }

    let family = info.psi.soi_family;
    if family != libc::AF_INET && family != libc::AF_INET6 {
        return Ok(None);
    }

    let in_info = match info.psi.soi_kind {
        SOCKINFO_TCP => unsafe { &info.psi.soi_proto.pri_tcp.tcpsi_ini },
        SOCKINFO_IN => unsafe { &info.psi.soi_proto.pri_in },
        _ => return Ok(None),
    };

    let addr = extract_remote_addr(in_info)?;
    Ok(addr)
}

fn extract_remote_addr(info: &InSockInfo) -> Result<Option<IpAddr>> {
    if info.insi_vflag & INI_IPV4 != 0 {
        let bytes = unsafe { info.insi_faddr.ina_46.i46a_addr4 };
        let addr = Ipv4Addr::from(bytes);
        if addr.is_unspecified() {
            return Ok(None);
        }
        Ok(Some(IpAddr::V4(addr)))
    } else if info.insi_vflag & INI_IPV6 != 0 {
        let bytes = unsafe { info.insi_faddr.ina_6 };
        let addr = Ipv6Addr::from(bytes);
        if addr.is_unspecified() {
            return Ok(None);
        }
        Ok(Some(IpAddr::V6(addr)))
    } else {
        Ok(None)
    }
}

pub fn collect_remote_addrs(pid: i32) -> Vec<IpAddr> {
    let fds = match list_fds(pid) {
        Ok(fds) => fds,
        Err(_) => return Vec::new(),
    };

    let mut addrs = Vec::new();
    for (fd, fdtype) in fds {
        if fdtype != PROX_FDTYPE_SOCKET {
            continue;
        }
        if let Ok(Some(addr)) = get_socket_remote_addr(pid, fd) {
            addrs.push(addr);
        }
    }
    addrs
}
