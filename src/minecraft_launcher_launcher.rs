use std::env;
use std::fs;
use std::process;
use std::thread;

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::time::Duration;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

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

#[cfg(target_os = "linux")]
use procfs::process::Process as ProcfsProcess;

#[cfg(target_os = "linux")]
use procfs::process::FDTarget::Path as ProcfsPath;

use once_cell::sync::Lazy;

use notify_rust::Notification;
use notify_rust::Urgency;

use log::debug;

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
        // Try sending a backup notification, in case sending of the first one
        // fails because description contained invalid characters or was at the
        // description limit
        if let Err(other_e) = Notification::new()
            .summary("Error")
            .body("Couldn't send a notification about an error")
            .urgency(Urgency::Critical)
            .show()
        {
            eprintln!(
                "{}{e}: {other_e}: {description}",
                "error: backup notification failed to send: ".red()
            );
        }

        // Hopefully we have a visible terminal or the user re launches the app
        // with one.
        eprintln!(
            "{}{e}: {description}",
            "error: can't send desktop notification to notify error: ".red()
        );
    }
}

fn notify_status(description: &str) {
    if let Err(e) = Notification::new()
        .summary("Status")
        .body(description)
        .urgency(Urgency::Low)
        .show()
    {
        // Hopefully we have a visible terminal or the user re launches the app
        // with one.
        eprintln!(
            "{}{e}: {description}",
            "error: can't send desktop notification to notify status: ".red()
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

static KILLING_IN_PROGRESS: Lazy<AtomicBool> =
    Lazy::new(|| AtomicBool::new(false));

#[cfg(not(target_os = "linux"))]
#[inline]
#[must_use]
fn is_launcher_profiles_file_open_in_process(_: &Process) -> bool {
    false
}

#[cfg(target_os = "linux")]
#[inline]
#[must_use]
fn is_launcher_profiles_file_open_in_process(process: &Process) -> bool {
    if let Some(home_folder) = home::home_dir() {
        let launcher_profiles_path =
            utils::get_minecraft_dir_from_home_path(Path::new(&home_folder))
                .join("launcher_profiles.json");

        if launcher_profiles_path.exists() {
            if let Ok(i32_pid) = i32::try_from(process.pid().as_u32()) {
                if let Ok(procfs_process) = ProcfsProcess::new(i32_pid) {
                    if let Ok(open_file_list) = procfs_process.fd() {
                        for file in open_file_list {
                            if let Ok(file_info) = file {
                                if let ProcfsPath(target) = file_info.target {
                                    if target == launcher_profiles_path {
                                        return true;
                                    } else if target
                                        .to_string_lossy()
                                        .contains("launcher_profiles.json")
                                    {
                                        notify_error(&format!("process has file {} open that is not detected by the Eq operator, comparing to {}", target.to_string_lossy(), launcher_profiles_path.to_string_lossy()));
                                        return true;
                                    }

                                    debug!(
                                        "process has file {} open",
                                        target.to_string_lossy()
                                    );
                                } else {
                                    debug!("{}", "Process has a non-file or file without a path (such as in RAM) open, skipping".yellow());
                                }
                            } else {
                                notify_error(&format!("Couldn't get details about an open file owned by process named {} with PID {}", process.name(), process.pid()));
                            }
                        }

                        println!("Process doesn't have launcher_profiles.json file open.");
                    } else {
                        notify_error(&format!("Couldn't get procfs list of open files for process named {} with PID {}", process.name(), process.pid()));
                    }
                } else {
                    eprintln!("Couldn't get procfs process for process named {} with PID {}. Killed?", process.name(), process.pid()); // Process might be already quit, not a fatal error
                }
            } else {
                notify_error(&format!(
                    "Can't convert usize PID to i32 PID: {}",
                    process.pid()
                ));
            }
        } else {
            notify_error("can't find launcher_profiles.json, ignore if this the first start of the launcher");
        }
    } else {
        notify_error("can't find home directory");
    }

    false
}

#[inline]
fn await_launcher_profile_save_in_process(process: &Process) {
    while is_launcher_profiles_file_open_in_process(process) {
        notify_status("Waiting for launcher profile save..");
        thread::sleep(Duration::from_millis(500));
    }
}

#[inline]
fn kill_launcher_process(launcher_process: &Process) {
    await_launcher_profile_save_in_process(launcher_process);

    if launcher_process.kill() {
        println!("Killed process successfully");
    } else {
        eprintln!("Couldn't kill Minecraft Launcher process named {} with PID {}. Already killed?", launcher_process.name(), launcher_process.pid());
        // Can happen if already killed, not a fatal error.
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
                .any(|element| element.contains("--launcherui"))
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
    println!("Starting monitoring");

    match PidMonitor::new() {
        Ok(mut monitor) => {
            let mut sys = System::new();

            loop {
                if let Some(e) = monitor.recv() {
                    match e {
                        PidEvent::Exec(id) => {
                            if let Ok(id_u32) = u32::try_from(id) {
                                let pid = Pid::from_u32(id_u32);

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
                                                        .contains("-Dminecraft.launcher.brand=minecraft-launcher")
                                                },
                                            )
                                        {
                                            find_launcher_processes(sys, true);
                                            break;
                                        }
                                    }
                                }
                            } else {
                                notify_error(&format!(
                                    "Can't convert i32 PID to usize PID: {id}"
                                ));
                            }
                        },

                        PidEvent::Exit(id) => {
                            if let Ok(id_u32) = u32::try_from(id) {
                                let pid = Pid::from_u32(id_u32);

                                if let Some(process) = sys.process(pid) {
                                    let name = process.name();

                                    if name == "minecraft-launc"
                                        && !KILLING_IN_PROGRESS
                                            .load(Ordering::Relaxed)
                                    {
                                        break;
                                    }
                                }
                            } else {
                                notify_error(&format!(
                                    "Can't convert i32 PID to usize PID: {id}"
                                ));
                            }
                        },

                        PidEvent::Fork { .. } | PidEvent::Coredump(_) => {},
                    }
                } else {
                    notify_error("no events to receive");
                }
            }
        },

        Err(e) => {
            notify_error(&format!(
                "error while trying to create process event watcher: {e}"
            ));
        },
    }
}

#[inline]
pub(crate) fn remove_javacheck() {
    home::home_dir().map_or_else(
        || {
            notify_error("can't find home directory");
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
                        "error while removing JavaCheck.jar: {e}"
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
        let mut envs = HashMap::from([
            ("vblank_mode", "0"), // Improves performance
            ("__GL_SYNC_TO_VBLANK", "0"), // Same as the above environment variable, but also works on NVIDIA closed source drivers.
            ("ALSOFT_DRIVERS", "pulse"), /* Fixes audio delay when
                                   * using pipewire */
            ("LIBGL_DRI2_DISABLE", "true"), // Force use of DRI3 if available
            ("MESA_NO_ERROR", "true"),      /* Disable error checking for
                                             * performance */
            ("MESA_GL_VERSION_OVERRIDE", "4.3"), /* Force increase
                                                  * advertised GL version
                                                  * for performance */
            ("MESA_GLES_VERSION_OVERRIDE", "3.2"), // ^^
            ("MESA_SHADER_CACHE_DISABLE", "false"), /* Force enable Shader
                                                    * Cache */
            ("MESA_SHADER_CACHE_MAX_SIZE", "4G"), /* Use a big value as limit for Shader Cache */
            ("LD_PRELOAD", "/usr/local/lib/libmimalloc.so.2"), /* Use mimalloc to increase memory/GC performance */
        ]);

        if let Ok(value) =
            env::var("MC_LAUNCHER_LAUNCHER_NO_GL_VERSION_OVERRIDE")
        {
            if value == "true" {
                println!("Not overriding advertised GL versions.");

                envs.remove("MESA_GL_VERSION_OVERRIDE");
                envs.remove("MESA_GLES_VERSION_OVERRIDE");
            }
        }

        if let Err(e) =
            Command::new("nice")
                .envs(envs)
                .arg("-n")
                .arg("-6")
                .arg("minecraft-launcher-real")
                .spawn()
        {
            notify_error(&format!(
                "error while trying to start Minecraft Launcher: {e}"
            ));
        }
    });
}

#[inline]
fn escalate_if_needed() -> bool {
    #[allow(box_pointers)]
    if let Err(e) = sudo::escalate_if_needed() {
        notify_error(&format!("error while trying to escalate to root permissions automatically: {e}"));

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
