use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use crate::utils::*;

#[derive(Copy, Debug, Clone)]
enum Distro {
    Alpine,
    Arch,
    Debian,
    Redhat,
    Void,
    Unknown,
}

impl Distro {
    fn detect<P: AsRef<Path>>(zroot: P) -> Self {
        let zroot = zroot.as_ref();

        let supported = [
            (Distro::Alpine, "etc/alpine-release"),
            (Distro::Arch, "etc/arch-release"),
            (Distro::Debian, "etc/debian_version"),
            (Distro::Redhat, "etc/redhat-release"),
            (Distro::Void, "etc/void-release"),
        ];

        if let Some((d, _)) = supported.iter().find(|(_, p)| zroot.join(p).exists()) {
            println!("detected distro as {:?}", d);
            return *d;
        }

        Distro::Unknown
    }

    fn install<P: AsRef<Path>>(&self, zroot: P) -> Result<()> {
        let zroot = zroot.as_ref();

        match self {
            Self::Alpine => {
                let rclocal = zroot.join("etc/rc.local");
                copy_file("guest/lib/smartdc/joyent_rc.local", &rclocal, 0, 0, 0o755)?;
                let shutdown = zroot.join("sbin/shutdown");
                copy_file("guest/sbin/shutdown", &shutdown, 0, 0, 0o744)?;
            }
            Self::Arch => {
                let system = zroot.join("etc/systemd/system");
                mkdirp(&system, 0, 0, 0o755)?;
                let service = &system.join("joyent.service");
                copy_file("etc/systemd/system/joyent.service", &service, 0, 0, 0o755)?;
                let enable =
                    zroot.join("etc/systemd/system/multi-user.target.wants/joyent.service");
                create_symlink(&service, &enable, 0, 0)?;
            }
            Self::Debian => {
                let rclocal = zroot.join("etc/rc.local");
                copy_file("guest/lib/smartdc/joyent_rc.local", &rclocal, 0, 0, 0o755)?;
            }
            Self::Redhat => {
                let dst = zroot.join("etc/rc.local");
                copy_file("guest/lib/smartdc/joyent_rc.local", &dst, 0, 0, 0o755)?;
            }
            Self::Void => {
                let rclocal = zroot.join("etc/rc.local");
                copy_file("guest/lib/smartdc/joyent_rc.local", &rclocal, 0, 0, 0o755)?;
                let shutdown = zroot.join("sbin/shutdown");
                copy_file("guest/sbin/shutdown", &shutdown, 0, 0, 0o744)?;
            }
            Self::Unknown => {
                bail!("failed to detect supported Linux Distribution");
            }
        };

        Ok(())
    }
}

fn install_native_manpath<P: AsRef<Path>>(zroot: P) -> Result<()> {
    let zroot = zroot.as_ref();

    copy_file(
        "guest/etc/profile.d/native_manpath.sh",
        zroot.join("etc/profile.d/native_manpath.sh"),
        0,
        0,
        0o744,
    )?;
    Ok(())
}

fn install_smartdc<P: AsRef<Path>>(zroot: P) -> Result<()> {
    let zroot = zroot.as_ref();
    copy_dir("guest/lib/smartdc", zroot.join("lib"), 0, 0, 0o755)?;

    Ok(())
}

fn install_distro<P: AsRef<Path>>(zroot: P) -> Result<()> {
    let zroot = zroot.as_ref();

    let distro = Distro::detect(zroot);
    distro.install(zroot)?;

    Ok(())
}

fn install_mdata_commands<P: AsRef<Path>>(zroot: P) -> Result<()> {
    let zroot = zroot.as_ref();

    let paths = [
        "usr/sbin/mdata-get",
        "usr/sbin/mdata-put",
        "usr/sbin/mdata-delete",
        "usr/sbin/mdata-list",
    ];

    for p in &paths {
        let dst = zroot.join(p);
        let src = Path::new("/native").join(p);
        if dst.exists() {
            fs::remove_file(&dst)
                .with_context(|| format!("failed to unlink {}", &src.display()))?;
            println!("unlinked {}", &src.display());
        }
        create_symlink(&src, &dst, 0, 0)?;
    }

    Ok(())
}

pub fn install_tools<P: AsRef<Path>>(zroot: P) -> Result<()> {
    let zroot = zroot.as_ref();

    install_mdata_commands(zroot)?;
    install_native_manpath(zroot)?;
    install_smartdc(zroot)?;
    install_distro(&zroot)?;
    Ok(())
}
