use libc::{SYS_write, syscall};

pub struct Echo {
    text: String,
}

impl Echo {

    pub fn new(text : String) -> Self {
        Echo { text: text + "\n"}
    }

    pub fn execute(&self) {
        let fd: libc::c_long = 1; // stdout
        let bytes = self.text.as_bytes();

        let ret = unsafe { syscall(SYS_write, fd, bytes.as_ptr(), bytes.len()) };

        if ret < 0 {
            eprintln!("syscall failed with return value {}", ret);
        }
    }
}
