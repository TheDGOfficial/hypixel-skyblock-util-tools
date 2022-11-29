use std::env;
use std::fs;
use std::process;

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

extern crate alloc;

use alloc::sync::Arc;

use crate::utils;
use colored::Colorize;

use sysinfo::Pid;
use sysinfo::PidExt;
use sysinfo::Process;
use sysinfo::ProcessExt;
use sysinfo::ProcessRefreshKind;
use sysinfo::System;
use sysinfo::SystemExt;

#[cfg(target_os = "linux")]
use sudo::RunningAs;

#[cfg(target_os = "linux")]
use cnproc::PidMonitor;

#[cfg(target_os = "linux")]
use cnproc::PidEvent;

use once_cell::sync::Lazy;

use notify_rust::Notification;
use notify_rust::Urgency;

// Since this program will be most likely not called with a (visible) terminal,
// send desktop notifications if any errors occur to not let a silent failure
// happen.
fn notify_error(description: &str) {
    if let Err(e) = Notification::new()
        .summary("Error")
        .body(description)
        .urgency(Urgency::Critical)
        .show()
    {
        // Hopefully we have a visible terminal or the user re launches the app
        // with one.
        eprintln!(
            "{}{e}: {description}",
            "error: can't send desktop notification to notify error: ".red()
        );
    }
}

#[cfg(not(target_os = "linux"))]
#[inline]
pub(crate) fn launch() -> ExitCode {
    notify_error("Minecraft Launcher launcher is only supported on Linux");

    ExitCode::FAILURE
}

// Minecraft Launcher Launcher launches Minecraft Launcher with various
// environment variables to fix bugs and improve performance, while also making
// it so when the game starts, all launcher processes will be killed. This will
// usually save around 300MB of memory from being wasted due to the launcher
// using an embedded browser to render its pages. After setting environment
// variables and launching the Launcher, this app will wait in the background
// for you to press Play and start the game in the launcher. Once you do that
// all launcher processes will be killed to save resources, and then after that
// this app will also quit, so only the java runtime (the actual game) is
// running.

// It will also automatically delete JavaCheck.jar to let you launch any game
// version with any Java version you desire.

// NOTE: Environment variables that are not supported, force disabled, etc.
// will just be ignored or not do anything at all.

// NOTE 2: Minecraft Launcher Launcher requires sudo. This not convenient, so
// you should do VISUAL=gnome-text-editor EDITOR="$VISUAL" sudo -E visudo and
// add yourusername ALL = (root) NOPASSWD: /usr/bin/minecraft-launcher to the
// last line. change gnome-text-editor with gedit if using old ubuntu versions,
// or another editor.

// NOTE 2.1: Despite that, it requires running it without sudo AND then
// escalating to sudo privileges because the launcher itself and java checker
// MUST run without sudo. Only the PID monitoring for the starting of Java
// process (game process) requires, and will, use sudo.

// TODO Future plans include checking for Bootstrap launcher updates.
#[inline]
#[cfg(target_os = "linux")]
pub(crate) fn launch() -> ExitCode {
    let user = sudo::check() == RunningAs::User;

    if user && find_launcher_processes(System::new(), false) {
        println!("Already open, exiting.");

        return ExitCode::SUCCESS;
    }

    if user {
        remove_javacheck();

        launch_launcher();
    }

    if !escalate_if_needed() {
        return ExitCode::FAILURE;
    }

    start_watching_java_process();

    ExitCode::SUCCESS
}

static KILLING_IN_PROGRESS: Lazy<Arc<AtomicBool>> =
    Lazy::new(|| Arc::new(AtomicBool::new(false)));

#[inline]
fn kill_launcher_process(launcher_process: &Process) {
    if launcher_process.kill() {
        println!("Killed process successfully");
    } else {
        notify_error(&format!(
            "couldn't kill Minecraft Launcher process named {} with PID {}",
            launcher_process.name(),
            launcher_process.pid()
        ));
    }
}

#[inline]
fn find_launcher_processes(mut sys: System, kill: bool) -> bool {
    if kill
        && KILLING_IN_PROGRESS.compare_exchange(
            false,
            true,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) != Ok(false)
    {
        notify_error("atomic operation failure (expected false, got true)");
    }

    sys.refresh_processes();

    let mut found = false;
    let self_pid = Pid::from_u32(process::id());

    #[allow(box_pointers)]
    for launcher_process in sys.processes_by_name(
        "minecraft-launc", /* Not a typo, process names are limited to 15
                            * characters in Linux as docs on the
                            * processes_by_name method suggests. */
    ) {
        if launcher_process.pid() != self_pid {
            println!(
                "Found launcher process {}. PID: {}",
                launcher_process.name(),
                launcher_process.pid()
            );

            if kill {
                kill_launcher_process(launcher_process);
            }

            found = true;
        }
    }

    // Workaround to also kill that one
    // process remaining that doesn't
    // use minecraft-launc name, but
    // uses exe
    for possible_stealth_launcher_process in sys.processes().values() {
        if possible_stealth_launcher_process.name() == "exe"
            && possible_stealth_launcher_process.pid() != self_pid
            && possible_stealth_launcher_process
                .cmd()
                .iter()
                .any(|element| element.contains(".minecraft"))
        {
            println!(
                "Found stealth launcher process {}. PID: {}",
                possible_stealth_launcher_process.name(),
                possible_stealth_launcher_process.pid()
            );

            if kill {
                kill_launcher_process(possible_stealth_launcher_process);
            }

            found = true;
        }
    }

    if kill
        && KILLING_IN_PROGRESS.compare_exchange(
            true,
            false,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) != Ok(true)
    {
        notify_error("atomic operation failure (expected true, got false)");
    }

    found
}

#[inline]
#[allow(unused_results)]
fn start_watching_java_process() {
    tokio::spawn(async move {
        println!("Starting monitoring");

        match PidMonitor::new() {
            Ok(mut monitor) => {
                let mut sys = System::new();

                loop {
                    if let Some(e) = monitor.recv() {
                        match e {
                            PidEvent::Exec(id) => {
                                let pid = Pid::from(id);

                                if sys.refresh_process_specifics(
                                    pid,
                                    ProcessRefreshKind::new(),
                                ) {
                                    if let Some(process) = sys.process(pid) {
                                        let name = process.name();

                                        if name == "java"
                                            && process.cmd().iter().any(
                                                |element| {
                                                    element
                                                        .contains(".minecraft")
                                                },
                                            )
                                        {
                                            find_launcher_processes(sys, true);
                                            break;
                                        }
                                    }
                                }
                            },

                            PidEvent::Exit(id) => {
                                let pid = Pid::from(id);

                                if let Some(process) = sys.process(pid) {
                                    let name = process.name();

                                    if name == "minecraft-launc"
                                        && !KILLING_IN_PROGRESS
                                            .load(Ordering::Relaxed)
                                    {
                                        break;
                                    }
                                }
                            },

                            PidEvent::Fork { .. } | PidEvent::Coredump(_) => {
                            },
                        }
                    } else {
                        notify_error(&format!(
                            "{}",
                            "no events to receive".yellow()
                        ));
                    }
                }

                ExitCode::SUCCESS
            },

            Err(e) => {
                notify_error(&format!(
                    "{}{e}",
                    "error while trying to create process event watcher: "
                        .red()
                ));

                ExitCode::FAILURE
            },
        }
    });
}

#[inline]
pub(crate) fn remove_javacheck() {
    home::home_dir().map_or_else(
        || {
            notify_error(&format!("{}", "can't find home directory: ".red()));
        },
        |home_folder| {
            let javacheck_path = utils::get_minecraft_dir_from_home_path(
                Path::new(&home_folder),
            )
            .join("launcher")
            .join("JavaCheck.jar");

            if javacheck_path.exists() {
                println!("Removing JavaCheck.jar");

                if let Err(e) = fs::remove_file(javacheck_path) {
                    notify_error(&format!(
                        "{}{e}",
                        "error while removing JavaCheck.jar: ".red()
                    ));
                }
            }
        },
    );
}

#[inline]
#[allow(unused_results)]
fn launch_launcher() {
    tokio::spawn(async move {
        if let Err(e) = Command::new("minecraft-launcher-real")
            .envs([
                ("vblank_mode", "0"), // Improves performance
                ("ALSOFT_DRIVERS", "pulse"), // Fixes audio delay when using pipewire
                ("LIBGL_DRI2_DISABLE", "true"), // Force use of DRI3 if available
                ("MESA_NO_ERROR", "true"), // Disable error checking for performance
                ("MESA_GL_VERSION_OVERRIDE", "4.3"), // Force increase advertised GL version for performance
                ("MESA_GLES_VERSION_OVERRIDE", "3.2"), // ^^
                ("MESA_SHADER_CACHE_DISABLE", "false"), // Force enable Shader Cache
                ("MESA_SHADER_CACHE_MAX_SIZE", "4G"), // Use a big value as limit for Shader Cache
                ("LD_PRELOAD", "/usr/local/lib/libmimalloc.so.2"), // Use mimalloc to increase memory/GC performance
            ])
            .spawn()
        {
            notify_error(
                &format!("{}{e}",
                "error while trying to start Minecraft Launcher: ".red()
            ));
        }
    });
}

#[inline]
fn escalate_if_needed() -> bool {
    #[allow(box_pointers)]
    if let Err(e) = sudo::escalate_if_needed() {
        notify_error(&format!("{}{e}", "error while trying to escalate to root permissions automatically: ".red()));

        return false;
    }

    true
}

#[inline]
#[cfg(not(target_os = "linux"))]
pub(crate) fn install(_: &str, _: &[String]) -> ExitCode {
    notify_error("Minecraft Launcher launcher is only supported on Linux");

    ExitCode::FAILURE
}

// This function installs the binary running this program itself to the
// /usr/bin/minecraft-launcher.
#[inline]
#[cfg(target_os = "linux")]
pub(crate) fn install(binary_file_name: &str, args: &[String]) -> ExitCode {
    if !escalate_if_needed() {
        return ExitCode::FAILURE;
    }

    println!("Starting install");

    return match env::current_exe() {
        Ok(self_path) => {
            if !self_path.exists() {
                eprintln!(
                    "{}",
                    "Current executable deleted, can't continue".red()
                );

                return ExitCode::FAILURE;
            }

            if let Some(self_path_file_name) = self_path.file_name() {
                let self_path_str = self_path_file_name.to_string_lossy();

                if *binary_file_name != self_path_str {
                    eprintln!("{}: current: {self_path_str}, original: {binary_file_name}", "error: current executable name and executable name passed to main differ".red());

                    return ExitCode::FAILURE;
                }

                if self_path_str.contains(char::REPLACEMENT_CHARACTER) {
                    eprintln!("non-unicode characters in executable path");

                    return ExitCode::FAILURE;
                }
            } else {
                eprintln!("can't get file name from current executable path");

                return ExitCode::FAILURE;
            }

            let bin_dir = Path::new("/usr").join("bin");

            if !bin_dir.exists() {
                eprintln!("bin directory doesn't exist, can't continue");

                return ExitCode::FAILURE;
            }

            let launcher_path = bin_dir.join("minecraft-launcher");

            if !launcher_path.exists() {
                eprintln!("Minecraft Launcher doesn't exist, can't continue. Please install it first.");

                return ExitCode::FAILURE;
            }

            println!(
                "Checking if already installed. This might take some time.."
            );

            match utils::is_same_file(&self_path, &launcher_path) {
                Ok(result) => {
                    if result {
                        println!("Already installed, nothing to do.");

                        return ExitCode::SUCCESS;
                    }

                    if find_launcher_processes(System::new(), true) {
                        println!("Killed launcher to proceed with install. Please restart it after install if desired.");
                    }

                    let real_launcher_path =
                        bin_dir.join("minecraft-launcher-real");

                    if args.contains(&"--upgrade".to_owned()) {
                        if let Err(e) =
                            fs::remove_file(real_launcher_path.clone())
                        {
                            eprintln!(
                                "{}{e}",
                                "error while removing real launcher: ".red()
                            );
                        }
                    }

                    if real_launcher_path.exists() {
                        println!("Real launcher already exists, will skip. If you've re-installed the launcher (bootstrap), please re-install again and then run the program with the --upgrade argument.");
                    } else if !utils::copy(&launcher_path, &real_launcher_path)
                    {
                        eprintln!("{}", "Install failed".red());

                        return ExitCode::FAILURE;
                    } else {
                        println!(
                            "Copied real launcher from {} to {} successfully",
                            launcher_path.to_string_lossy(),
                            real_launcher_path.to_string_lossy()
                        );
                    }

                    if !utils::copy(&self_path, &launcher_path) {
                        eprintln!("{}", "Install failed".red());

                        return ExitCode::FAILURE;
                    }

                    println!("Install successful");

                    ExitCode::SUCCESS
                },

                Err(e) => {
                    eprintln!("{}{e}", "error while comparing current executable with launcher path to check if they are same: ".red());

                    return ExitCode::FAILURE;
                },
            }
        },

        Err(e) => {
            eprintln!(
                "{}{e}",
                "error when getting current executable path: ".red()
            );

            ExitCode::FAILURE
        },
    };
}
