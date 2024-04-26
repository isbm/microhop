use nix::{mount::MsFlags, sys::statvfs, unistd};
use std::{fs, io::Error};
use sys_mount::Mount;
use walkdir::WalkDir;

/// Utilities for the root filesystem operations.
///
/// This module is intended to do all the basic operations those are typically
/// done by external utils, such as mount, umount, switch root etc.

/// Returns filesystem type
fn fs_type(p: &str) -> Result<u64, Error> {
    Ok(statvfs::statvfs(p)?.filesystem_id())
}

/// Recursively removes everything from the ramfs
fn rmrf() -> Result<(), Error> {
    fn is_sys(e: &str) -> bool {
        let sr = "/sysroot";
        for d in ["/proc", "/sys", "/dev", sr] {
            if e != "/"
                || e.starts_with(d)
                || e.starts_with(format!("{}{}", sr, d).as_str())
                || e.starts_with(format!("{}{}/", sr, d).as_str())
            {
                return true;
            }
        }

        false
    }

    WalkDir::new("/").into_iter().flat_map(|r| r.ok()).for_each(|e| {
        let p = e.path().as_os_str().to_str().unwrap_or_default();
        if let Ok(fst) = fs_type(p) {
            if fst == 0 && !is_sys(p) && e.path().is_dir() {
                //println!("{} : {:?}", fst, e.path());
                fs::remove_dir_all(e.path()).unwrap_or_default();
            }
        }
    });
    Ok(())
}

/// Mounts mountpoint
pub fn mount(fstype: &str, dev: &str, dst: &str) -> Result<(), Error> {
    if let Err(err) = Mount::builder().fstype(fstype).mount(dev, dst) {
        return Err(Error::new(std::io::ErrorKind::NotConnected, format!("Failed to mount {}: {}", fstype, err)));
    } else {
        log::debug!("Mounted {} at {} as {}", dev, dst, fstype);
    }

    Ok(())
}

/// Un-mount a mountpoint.
pub fn umount(dst: &str) -> Result<(), Error> {
    Ok(nix::mount::umount(dst)?)
}

/// Switches root
pub fn pivot(temp: &str, fstype: &str) -> Result<(), Error> {
    rmrf()?;
    log::debug!("free ramfs");

    unistd::chdir(temp)?;
    nix::mount::mount(Some("."), "/", Some(fstype), MsFlags::MS_MOVE, Option::<&str>::None)?;
    unistd::chroot(".")?;
    log::debug!("enter the rootfs");

    Ok(())
}
