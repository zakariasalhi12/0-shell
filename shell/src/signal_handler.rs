use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::consts::signal::*;
use signal_hook::flag;
use nix::unistd::{setpgid, Pid};
use nix::sys::signal::{kill, Signal};
use nix::unistd::getpgrp;
use std::sync::Mutex;
use std::collections::HashMap;

pub struct SignalHandler {
    pub sigint_received: Arc<AtomicBool>,
    pub sigtstp_received: Arc<AtomicBool>,
    pub foreground_pid: Arc<Mutex<Option<u32>>>,
}

impl SignalHandler {
    pub fn new() -> Self {
        let sigint_received = Arc::new(AtomicBool::new(false));
        let sigtstp_received = Arc::new(AtomicBool::new(false));
        let foreground_pid = Arc::new(Mutex::new(None));

        // Set up signal handlers
        let sigint_flag = sigint_received.clone();
        let sigtstp_flag = sigtstp_received.clone();
        let fg_pid = foreground_pid.clone();

        // Handle SIGINT (Ctrl+C)
        flag::register_usr1(SIGINT, move || {
            sigint_flag.store(true, Ordering::Relaxed);
        }).expect("Error setting SIGINT handler");

        // Handle SIGTSTP (Ctrl+Z)  
        flag::register_usr2(SIGTSTP, move || {
            sigtstp_flag.store(true, Ordering::Relaxed);
        }).expect("Error setting SIGTSTP handler");

        // Handle SIGCHLD for process cleanup
        let fg_pid_clone = fg_pid.clone();
        flag::register_usr1(SIGCHLD, move || {
            // This will be handled in the main loop
        }).expect("Error setting SIGCHLD handler");

        SignalHandler {
            sigint_received,
            sigtstp_received,
            foreground_pid,
        }
    }

    pub fn set_foreground_pid(&self, pid: Option<u32>) {
        if let Ok(mut fg_pid) = self.foreground_pid.lock() {
            *fg_pid = pid;
        }
    }

    pub fn get_foreground_pid(&self) -> Option<u32> {
        if let Ok(fg_pid) = self.foreground_pid.lock() {
            *fg_pid
        } else {
            None
        }
    }

    pub fn handle_signals(&self) -> (bool, bool) {
        let sigint = self.sigint_received.swap(false, Ordering::Relaxed);
        let sigtstp = self.sigtstp_received.swap(false, Ordering::Relaxed);
        (sigint, sigtstp)
    }

    pub fn send_signal_to_foreground(&self, signal: Signal) -> Result<(), String> {
        if let Some(pid) = self.get_foreground_pid() {
            let nix_pid = Pid::from_raw(pid as i32);
            kill(nix_pid, signal)
                .map_err(|e| format!("Failed to send signal: {}", e))?;
            Ok(())
        } else {
            Err("No foreground process".to_string())
        }
    }
}
