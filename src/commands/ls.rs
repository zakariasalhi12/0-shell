use crate::ShellCommand;
use std::fs::{FileType, Metadata};
use std::fs::{read_dir, symlink_metadata};
use std::io::Error;
use std::io::{ErrorKind, Result};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use std::{self, fs};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, PartialEq, Eq)]
pub struct Ls {
    pub args: Vec<String>,
    pub opts: Vec<String>,
    pub all: bool,
    pub classify: bool,
    pub format: bool,
    pub valid_opts: bool,
}

struct EntryInfo {
    name: String,
    path: PathBuf,
    metadata: Metadata,
}

impl Ls {
    pub fn new(args: Vec<String>, opts: Vec<String>) -> Self {
        let mut res = Ls {
            args,
            opts,
            all: false,
            classify: false,
            format: false,
            valid_opts: true,
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
                            self.valid_opts = false;
                            return;
                        }
                    }
                }
            } else {
                self.valid_opts = false;
                return;
            }
        }
    }

    fn format_entry(&self, entry: &EntryInfo) -> String {
        let file_type = entry.metadata.file_type();
        let display_name = if self.classify {
            display_name_with_suffix(&entry.path, &entry.name, &file_type, &entry.metadata)
        } else {
            entry.name.clone()
        };

        if self.format {
            self.format_long_entry(&entry.metadata, &display_name)
        } else {
            format!("{}\r", display_name)
        }
    }

    fn format_long_entry(&self, meta: &Metadata, name: &str) -> String {
        let mode = meta.permissions().mode();
        let file_type = meta.file_type();
        let perm = build_perm_string(mode, &file_type);
        let nlink = meta.nlink();
        let uid = meta.uid();
        let gid = meta.gid();
        let size = meta.size();

        let datetime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .and_then(|d| chrono::NaiveDateTime::from_timestamp_opt(d.as_secs() as i64, 0))
            .map(|dt| dt.format("%b %e %H:%M").to_string())
            .unwrap_or_else(|| "???".to_string());

        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or(uid.to_string());
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().to_string())
            .unwrap_or(gid.to_string());

        format!(
            "{} {} {} {} {:>5} {} {}\r",
            file_type_char(&file_type) + &perm,
            nlink,
            user,
            group,
            size,
            datetime,
            name
        )
    }

    fn create_entry_info(name: String, path: PathBuf) -> Result<EntryInfo> {
        let metadata = symlink_metadata(&path)?;
        Ok(EntryInfo {
            name,
            path,
            metadata,
        })
    }

    fn handle_directory(&self, path: &Path) -> Result<()> {
        let mut entries = Vec::new();

        // Handle special entries . and .. if -a flag is set
        if self.all {
            for special in &[".", ".."] {
                let special_path = path.join(special);
                if let Ok(entry_info) = Self::create_entry_info(special.to_string(), special_path) {
                    println!("{}", self.format_entry(&entry_info));
                }
            }
        }

        // Read directory entries
        let dir_entries: Vec<fs::DirEntry> = read_dir(path)?.filter_map(Result::ok).collect();

        if !self.format && !self.classify && !self.all {
            // Sort for default alphabetical order
            let mut sorted_entries = dir_entries;
            sorted_entries.sort_by_key(|e| e.path());
            entries.extend(sorted_entries);
        } else {
            entries.extend(dir_entries);
        }

        for entry in entries {
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files unless -a
            if !self.all && file_name.starts_with('.') {
                continue;
            }

            if let Ok(entry_info) = Self::create_entry_info(file_name, entry.path()) {
                println!("{}", self.format_entry(&entry_info));
            }
        }

        Ok(())
    }

    fn handle_file(&self, path: &Path) -> Result<()> {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let entry_info = Self::create_entry_info(name, path.to_path_buf())?;
        println!("{}", self.format_entry(&entry_info));
        Ok(())
    }
}

impl ShellCommand for Ls {
    fn execute(&self) -> Result<()> {
        if !self.valid_opts {
            return Err(Error::new(ErrorKind::InvalidInput, "ls: invalid flag"));
        }
        println!("{:?}", self.args);
        let targets = if self.args.is_empty() {
            vec![".".to_string()]
        } else {
            self.args.clone()
        };

        for target in targets {
            let path = Path::new(&target);
            if path.is_dir() {
                self.handle_directory(path)?;
            } else {
                self.handle_file(path)?;
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
