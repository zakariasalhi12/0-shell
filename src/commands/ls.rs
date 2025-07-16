use crate::ShellCommand;
use std::fs::{DirEntry, FileType, Metadata};
use std::fs::{read_dir, symlink_metadata};
use std::io::Error;
use std::io::{ErrorKind, Result};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{self, env, fs};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, PartialEq, Eq)]
pub struct Ls {
    pub args: Vec<String>,
    pub opts: Vec<String>,
    pub all: bool,
    pub classify: bool,
    pub format: bool,
    pub Valid_opts: bool,
}

impl Ls {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        let mut res = Ls {
            args,
            opts,
            all: false,
            classify: false,
            format: false,
            Valid_opts: true,
        };
        res.parse_flags();
        res
    }
    pub fn parse_flags(&mut self) {
        for f in &self.opts {
            if f.starts_with('-') && f.len() > 1 {
                for ch in f.chars().skip(1) {
                    match ch {
                        'a' => self.all = true,
                        'F' => self.classify = true,
                        'l' => self.format = true,
                        _ => {
                            self.Valid_opts = false;
                            return;
                        }
                    }
                }
            } else {
                self.Valid_opts = false;
                return;
            }
        }
    }
}

impl ShellCommand for Ls {
    // Add `users` crate in Cargo.toml

    fn execute(&self) -> Result<()> {
        if !self.Valid_opts {
            return Err(Error::new(ErrorKind::InvalidInput, "ls: invalid flag"));
        }

        let targets = if self.args.is_empty() {
            vec![".".to_string()]
        } else {
            self.args.clone()
        };

        for target in targets {
            let path = Path::new(&target);
            if path.is_dir() {
                let mut entries: Vec<fs::DirEntry> = Vec::new();

                if self.all {
                    // Manually add "." and ".."
                    for special in &[".", ".."] {
                        let special_path = path.join(special);
                        if let Ok(meta) = fs::symlink_metadata(&special_path) {
                            // We need to create a pseudo DirEntry, so instead store (name, metadata) or handle this outside
                            // Here we handle it specially before pushing read_dir entries
                            let display_name = if self.classify {
                                display_name_with_suffix(
                                    &special_path,
                                    special,
                                    &meta.file_type(),
                                    &meta,
                                )
                            } else {
                                special.to_string()
                            };

                            if self.format {
                                print_entry_long(&meta, special, &special_path, self.classify);
                            } else {
                                println!("{}\r", display_name);
                            }
                        }
                    }
                }

                // Now read actual entries from the directory
                entries.extend(read_dir(path)?.filter_map(Result::ok));

                if !self.format && !self.classify && !self.all {
                    entries.sort_by_key(|e| e.path()); // default alphabetical
                }

                for entry in entries {
                    let meta = symlink_metadata(entry.path())?;
                    let file_type = meta.file_type();
                    let file_name = entry.file_name();
                    let file_name_str = if self.classify {
                        display_name_with_suffix(
                            &entry.path(),
                            &file_name.to_string_lossy(),
                            &file_type,
                            &meta,
                        )
                    } else {
                        file_name.to_string_lossy().to_string()
                    };

                    // Skip hidden files unless -a
                    if !self.all && file_name_str.starts_with('.') {
                        continue;
                    }

                    if self.format {
                        let mode = meta.permissions().mode();
                        let file_type_char = file_type_char(&file_type);
                        let perm = build_perm_string(mode, &file_type);
                        let nlink = meta.nlink();
                        let uid = meta.uid();
                        let gid = meta.gid();
                        let size = meta.size();
                        let mtime = meta
                            .modified()?
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let user = get_user_by_uid(uid)
                            .map(|u| u.name().to_string_lossy().to_string())
                            .unwrap_or(uid.to_string());
                        let group = get_group_by_gid(gid)
                            .map(|g| g.name().to_string_lossy().to_string())
                            .unwrap_or(gid.to_string());

                        let datetime = chrono::NaiveDateTime::from_timestamp_opt(mtime as i64, 0)
                            .map(|dt| dt.format("%b %e %H:%M").to_string())
                            .unwrap_or("???".to_string());
                        // let display_name =

                        println!(
                            "{} {} {} {} {:>5} {} {}\r",
                            file_type_char + &perm,
                            nlink,
                            user,
                            group,
                            size,
                            datetime,
                            file_name_str
                        );
                    } else {
                        println!("{}\r", file_name_str);
                    }
                }
            } else {
                let meta = symlink_metadata(path)?;
                let name = path.file_name().unwrap().to_string_lossy();
                if self.format {
                    let mode = meta.permissions().mode();
                    let file_type = meta.file_type();
                    let perm = build_perm_string(mode, &file_type);
                    let nlink = meta.nlink();
                    let uid = meta.uid();
                    let gid = meta.gid();
                    let size = meta.size();
                    let mtime = meta
                        .modified()?
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let user = get_user_by_uid(uid)
                        .map(|u| u.name().to_string_lossy().to_string())
                        .unwrap_or(uid.to_string());
                    let group = get_group_by_gid(gid)
                        .map(|g| g.name().to_string_lossy().to_string())
                        .unwrap_or(gid.to_string());

                    let datetime = chrono::NaiveDateTime::from_timestamp_opt(mtime as i64, 0)
                        .map(|dt| dt.format("%b %e %H:%M").to_string())
                        .unwrap_or("???".to_string());

                    println!(
                        "{} {} {} {} {:>5} {} {}\r",
                        file_type_char(&file_type) + &perm,
                        nlink,
                        user,
                        group,
                        size,
                        datetime,
                        name
                    );
                } else {
                    println!("{}\r", name);
                }
            }
        }

        Ok(())
    }
}

fn file_type_char(ft: &std::fs::FileType) -> String {
    if ft.is_dir() {
        "d".to_string()
    } else if ft.is_symlink() {
        "l".to_string()
    } else if ft.is_block_device() {
        "b".to_string()
    } else if ft.is_char_device() {
        "c".to_string()
    } else if ft.is_socket() {
        "s".to_string()
    } else if ft.is_fifo() {
        "p".to_string()
    } else {
        "-".to_string()
    }
}

fn build_perm_string(mode: u32, file_type: &std::fs::FileType) -> String {
    let mut perm = String::with_capacity(9);
    let suid = mode & 0o4000 != 0;
    let sgid = mode & 0o2000 != 0;
    let sticky = mode & 0o1000 != 0;

    let rwx = |bit: u32, xbit: u32, special: bool, special_char: char, default: char| {
        let read = if bit & 0o4 != 0 { 'r' } else { '-' };
        let write = if bit & 0o2 != 0 { 'w' } else { '-' };
        let exec = if bit & 0o1 != 0 {
            if special { special_char } else { 'x' }
        } else {
            if special {
                special_char.to_ascii_uppercase()
            } else {
                '-'
            }
        };
        format!("{}{}{}", read, write, exec)
    };

    perm += &rwx((mode >> 6) & 0o7, 0o100, suid, 's', 'x');
    perm += &rwx((mode >> 3) & 0o7, 0o010, sgid, 's', 'x');
    perm += &rwx((mode >> 0) & 0o7, 0o001, sticky, 't', 'x');
    perm
}

fn display_name_with_suffix(
    path: &Path,
    file_name: &str,
    file_type: &FileType,
    metadata: &Metadata,
) -> String {
    let mut suffix = "";

    if file_type.is_dir() {
        suffix = "/";
    } else if file_type.is_symlink() {
        suffix = "@";
    } else if file_type.is_socket() {
        suffix = "=";
    } else if file_type.is_fifo() {
        suffix = "|";
    } else if file_type.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
        suffix = "*";
    }

    format!("{}{}", file_name, suffix)
}

fn print_entry_long(meta: &fs::Metadata, name: &str, path: &Path, classify: bool) {
    let mode = meta.permissions().mode();
    let file_type = meta.file_type();
    let perm = build_perm_string(mode, &file_type);
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.size();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| chrono::NaiveDateTime::from_timestamp_opt(d.as_secs() as i64, 0))
        .flatten()
        .map(|dt| dt.format("%b %e %H:%M").to_string())
        .unwrap_or_else(|| "???".to_string());

    let user = get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or(uid.to_string());
    let group = get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or(gid.to_string());

    let name_display = if classify {
        display_name_with_suffix(path, name, &file_type, meta)
    } else {
        name.to_string()
    };

    println!(
        "{} {} {} {} {:>5} {} {}\r",
        file_type_char(&file_type) + &perm,
        nlink,
        user,
        group,
        size,
        mtime,
        name_display
    );
}
