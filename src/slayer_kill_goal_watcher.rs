use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::time::Duration;

extern crate alloc;

use alloc::sync::Arc;

use std::fs;
use std::fs::File;

use colored::Colorize;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::utils::ask_int_input;
use crate::utils::get_minecraft_dir;
use crate::utils::lines_from_file_from_end;
use crate::utils::nano_time;
use crate::utils::num_cpus;
use crate::utils::read_file;
use crate::utils::u128_to_u64;
use crate::utils::write_file;
use arboard::Clipboard;
use futures::channel::mpsc::channel;
use futures::channel::mpsc::Receiver;
use futures::SinkExt;
use futures::StreamExt;
use notify::Config;
use notify::Event;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Watcher;
use notify_rust::Notification;
use notify_rust::Urgency;
use once_cell::sync::Lazy;
use serde_json::Error;

// TODO
// Make it count individual twilight arrow poison amount gained, currently it
// only counts the twilight arrow poison drops i.e lets say you did 2 bosses
// and got 64 arrow poisons each, total of 128 poisons, current program will
// say you got 2 drops

// NOTE: It will say RARE DROP! (Twilight Arrow Poison) (+325% Magic Find!) if
// x64 dropped, otherwise it will append the amount like; RARE DROP! (62x
// Twilight Arrow Poison) (+325% Magic Find!)

#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
)]
struct VoidgloomData {
    start_time: u128,
    end_time: u128,

    bosses_done: i32,

    twilight_arrow_poisons: i32,
    endersnake_runes: i32,
    summoning_eyes: i32,
    mana_steals: i32,
    transmission_tuners: i32,
    null_atoms: i32,
    hazmat_endermans: i32,
    espressos: i32,
    smarty_pants: i32,
    end_runes: i32,
    handy_blood_chalices: i32,
    sinful_dices: i32,
    artifact_upgraders: i32,
    etherwarp_mergers: i32,
    void_conqueror_skins: i32,
    judgement_cores: i32,
    enchant_runes: i32,
    ender_slayer_tier_sevens: i32,
}

#[inline]
fn duration_diff(duration1: Duration, duration2: Duration) -> Duration {
    if duration1 > duration2 {
        return duration1 - duration2;
    }

    duration2 - duration1
}

#[inline]
fn print_values(data: &VoidgloomData) {
    println!();

    let end_duration = Duration::from_nanos(u128_to_u64(data.end_time));
    let start_duration = Duration::from_nanos(u128_to_u64(data.start_time));

    println!("Duration: {:.2?}", duration_diff(start_duration, end_duration));
    println!("Bosses done: {}", data.bosses_done);
    println!();

    println!("Drops:");

    println!(
        " {}{}",
        "Twilight Arrow Poison Drops: ".bright_green(),
        data.twilight_arrow_poisons
    );

    println!(
        " {}{}",
        "Endersnake Runes: ".bright_purple(),
        data.endersnake_runes
    );
    println!(" {}{}", "Summoning Eyes: ".purple(), data.summoning_eyes);

    println!(" {}{}", "Mana Steal I Books: ".cyan(), data.mana_steals);

    println!(
        " {}{}",
        "Transmission Tuners: ".bright_purple(),
        data.transmission_tuners
    );
    println!(" {}{}", "Null Atoms: ".bright_blue(), data.null_atoms);

    println!(
        " {}{}",
        "Hazmat Enderman Power Stones: ".yellow(),
        data.hazmat_endermans
    );

    println!(
        " {}{}",
        "Pocket Espresso Machines: ".bright_cyan(),
        data.espressos
    );
    println!(" {}{}", "Smarty Pants I Books: ".cyan(), data.smarty_pants);

    println!(" {}{}", "End Runes: ".purple(), data.end_runes);
    println!(
        " {}{}",
        "Handy Blood Chalices: ".red(),
        data.handy_blood_chalices
    );

    println!(" {}{}", "Sinful Dices: ".red(), data.sinful_dices);
    println!(
        " {}{}",
        "Artifact Upgraders: ".bright_purple(),
        data.artifact_upgraders
    );

    println!(
        " {}{}",
        "Etherwarp Mergers: ".bright_purple(),
        data.etherwarp_mergers
    );

    println!(
        " {}{}",
        "Void Conqueror Enderman Pet Skins: ".bright_purple(),
        data.void_conqueror_skins
    );

    println!(
        " {}{}",
        "Judgement Cores: ".bright_yellow(),
        data.judgement_cores
    );
    println!(" {}{}", "Enchant Runes: ".white(), data.enchant_runes);

    println!(
        " {}{}",
        "Ender Slayer VII Books: ".bright_red(),
        data.ender_slayer_tier_sevens
    );
}

#[inline]
fn print_statistics(label: &str, file: &Path) {
    if file.exists() {
        if let Some(json) = read_file(file) {
            if let Some(data) = load_data(json.as_str()) {
                println!();
                println!("-- Statistics for {label}");
                print_values(&data);
            }
        }
    } else {
        eprintln!("{}{label}", "No statistics for ".red());
    }
}

#[inline]
fn load_data(json: &str) -> Option<VoidgloomData> {
    let data_result: Result<VoidgloomData, Error> = serde_json::from_str(json);

    match data_result {
        Ok(data) => Some(data),

        Err(e) => {
            eprintln!("{}{e}", "error: can't parse JSON from string: ".red());

            None
        },
    }
}

#[inline]
fn get_log_file_path() -> Option<PathBuf> {
    if let Some(minecraft_dir) = get_minecraft_dir() {
        return Some(minecraft_dir.join("logs").join("latest.log"));
    }

    None
}

#[inline]
fn get_data_dir() -> PathBuf {
    PathBuf::from(Path::new("data"))
}

#[inline]
fn get_global_data_file() -> PathBuf {
    get_data_dir().join("global.json")
}

#[inline]
fn get_last_session_data_file() -> PathBuf {
    get_data_dir().join("last_session.json")
}

#[inline]
fn get_unique_old_session_path() -> PathBuf {
    let previous_sessions_folder = get_data_dir().join("previous-sessions");

    if previous_sessions_folder.exists()
        || ensure_created(&previous_sessions_folder)
    {
        for index in 1..=i32::MAX {
            let path =
                previous_sessions_folder.join(format!("session-{index}.json"));

            if !path.exists() {
                return path;
            }
        }
    }

    eprintln!("{}", "warning: can't find a suitable unique session id, using 'no-id' as id instead".yellow());
    previous_sessions_folder.join("session-no-id.json")
}

#[inline]
fn ensure_created(path: &Path) -> bool {
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!("{}{e}", "error: can't create data directory: ".red());

        return false;
    }

    true
}

#[inline]
fn get_last_session(
    last_session_file: &Path,
    session_data: &mut VoidgloomData,
    print_warnings: bool,
) -> bool {
    // Load last session
    if last_session_file.exists() {
        if let Some(file_content) = read_file(last_session_file) {
            if let Some(last_session_data) = load_data(file_content.as_str()) {
                *session_data = last_session_data;
            } else {
                // Most probably corrupted data. Errors will be already
                // printed by the function call.
                return false;
            }
        } else {
            // Most probably a permission error. Errors will be already
            // printed by the function call.
            return false;
        }
    } else {
        // Caller's problem, just emit a warning if desired
        if print_warnings {
            eprintln!("{}", "warning: can't find last session, will continue with a new session".yellow());
        }
    }

    true
}

#[inline]
fn get_global_data(
    global_data: &mut VoidgloomData,
    print_warnings: bool,
) -> bool {
    // Load global data
    return if get_global_data_file().exists() {
        read_file(&get_global_data_file()).map_or(false, |file_content| {
            load_data(file_content.as_str()).map_or(false, |data| {
                *global_data = data;

                true
            })
        })
    } else {
        // Caller's problem, just emit a warning if desired
        if print_warnings {
            eprintln!("{}", "warning: can't find global data file".yellow());
        }

        false
    };
}

#[inline]
fn print_selections() {
    println!();
    println!("Select what you want to do: ");

    println!(" {}. Continue from last session", "1".bright_blue());
    println!(" {}. Start a new session", "2".bright_blue());

    println!(" {}. Reset all global data", "3".bright_blue());
    println!(" {}. View statistics", "4".bright_blue());
}

#[inline]
pub(crate) fn slayer_kill_goal_watcher(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    print_selections();

    let selection =
        ask_int_input("Enter a number to select: ", Some(1), Some(4));
    *start_without_user_input = Some(Instant::now());

    let data_folder = get_data_dir();

    if !ensure_created(&data_folder) {
        return false;
    }

    let global_data_file = get_global_data_file();
    let last_session_file = get_last_session_data_file();

    let session_data = &mut VoidgloomData::default();
    let global_data = &mut VoidgloomData::default();

    if selection == 1 {
        if !get_last_session(&last_session_file, session_data, true) {
            return false;
        }
    } else if selection == 2 {
        if get_last_session(&last_session_file, session_data, false) {
            session_data.end_time = nano_time().unwrap_or(0);

            if !save_session_data_to_file(session_data) {
                eprintln!("{}", "warning: file save to end previous session failed, look above for possible errors".yellow());
            }

            // Save previous session
            if let Err(e) =
                fs::copy(last_session_file, get_unique_old_session_path())
            {
                eprintln!(
                    "{}{e}",
                    "error: can't copy previous session to save it: ".red()
                );
            }

            // Reset current session
            *session_data = VoidgloomData::default();
        }
    } else if selection == 3 {
        // Reset global data
        if global_data_file.exists() {
            match fs::remove_file(global_data_file) {
                Ok(()) => {
                    println!(
                        "{}",
                        "Successfully reset global data.".bright_green()
                    );
                },

                Err(e) => {
                    eprintln!(
                        "{}{e}",
                        "error: can't reset global data: ".red()
                    );
                },
            }
        } else {
            eprintln!("{}", "No data to remove".bright_red());
        }

        return true;
    } else if selection == 4 {
        print_all_statistics(
            &global_data_file,
            &last_session_file,
            session_data,
            global_data,
        );

        return true;
    } else {
        eprintln!("{}{selection}", "error: invalid selection: ".red());
    }

    if !global_data_file.exists() {
        if let Err(e) = File::create(global_data_file) {
            eprintln!("{}{e}", "error: can't create global data file: ".red());
        }

        if !save_global_data_to_file(global_data) {
            eprintln!("{}", "warning: initialization of newly created global data file with default values failed, look above for possible errors".yellow());
        }
    }

    if session_data.start_time == 0 {
        if let Some(epoch) = nano_time() {
            // Warning will be written by the util method if we can't get the
            // time
            session_data.start_time = epoch;

            if !save_session_data_to_file(session_data) {
                eprintln!("{}", "warning: saving of session data to reflect session start time failed, look above for possible errors".yellow());
            }
        }
    }

    if get_global_data(global_data, true) && global_data.start_time == 0 {
        if let Some(epoch) = nano_time() {
            // Warning will be written by the util method if we can't get the
            // time
            global_data.start_time = epoch;

            if !save_global_data_to_file(global_data) {
                eprintln!("{}", "warning: saving of global data to reflect start time failed, look above for possible errors".yellow());
            }
        }
    }

    register_watcher_with_new_clipboard(session_data, global_data);

    true
}

#[inline]
fn register_watcher_with_new_clipboard(
    session_data: &mut VoidgloomData,
    global_data: &mut VoidgloomData,
) {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            register_watcher(session_data, global_data, &mut clipboard);
        },

        Err(e) => {
            eprintln!(
                "{}{e}",
                "error while creating clipboard context: ".red()
            );
        },
    }
}

#[inline]
fn register_watcher(
    session_data: &mut VoidgloomData,
    global_data: &mut VoidgloomData,
    clipboard: &mut Clipboard,
) {
    futures::executor::block_on(async {
        if let Some(path) = get_log_file_path() {
            if let Err(e) =
                async_watch(path, session_data, global_data, clipboard).await
            {
                eprintln!("{}{e}", "watch error: ".red());
            }
        }
    });
}

#[inline]
fn print_all_statistics(
    global_data_file: &Path,
    last_session_file: &Path,
    session_data: &mut VoidgloomData,
    global_data: &mut VoidgloomData,
) {
    // Print statistics about last session and all sessions from global
    // data.
    if get_last_session(last_session_file, session_data, false) {
        let mut changed = false;

        if session_data.end_time == 0 && get_last_session_data_file().exists()
        {
            if let Some(epoch) = nano_time() {
                // Warning will be written by util method if we can't get time
                session_data.end_time = epoch;
                changed = true;

                if !save_session_data_to_file(session_data) {
                    eprintln!("{}", "warning: saving of session data to reflect end time failed, look above for possible errors".yellow());
                }
            };
        }

        print_statistics("Last session", last_session_file);

        if changed {
            session_data.end_time = 0;
            if !save_session_data_to_file(session_data) {
                eprintln!("{}", "warning: saving of session data to revert end time failed, look above for possible errors".yellow());
            }
        }
    }

    if get_global_data(global_data, false) {
        let mut global_changed = false;

        if global_data.end_time == 0 && global_data_file.exists() {
            if let Some(epoch) = nano_time() {
                // Warning will be written by util method if we can't get time
                global_data.end_time = epoch;
                global_changed = true;

                if !save_global_data_to_file(global_data) {
                    eprintln!("{}", "warning: saving of global data to reflect end time failed, look above for possible errors".yellow());
                }
            };
        }

        print_statistics("Global", global_data_file);

        if global_changed {
            global_data.end_time = 0;
            if !save_global_data_to_file(global_data) {
                eprintln!("{}", "warning: saving of global data to revert end time failed, look above for possible errors".yellow());
            }
        }
    }
}

#[inline]
fn save_global_data_to_file(global_data: &VoidgloomData) -> bool {
    match serde_json::to_string_pretty(global_data) {
        Ok(json) =>
            if !write_file(&get_global_data_file(), json.as_str()) {
                return false;
            },

        Err(e) => {
            eprintln!(
                "{}{e}",
                "error: can't convert global data to json: ".red()
            );
        },
    }

    true
}

#[inline]
fn save_session_data_to_file(data: &VoidgloomData) -> bool {
    match serde_json::to_string_pretty(data) {
        Ok(json) =>
            if !write_file(&get_last_session_data_file(), json.as_str()) {
                return false;
            },

        Err(e) => {
            eprintln!("{}{e}", "error: can't convert data to json: ".red());
        },
    }

    true
}

static PRINTED_MSG: Lazy<Arc<AtomicBool>> =
    Lazy::new(|| Arc::new(AtomicBool::new(false)));

#[inline]
fn save_data_to_file(
    data: &VoidgloomData,
    global_data: &VoidgloomData,
) -> bool {
    // println!("Saving session and global data as changes occurred");

    if !save_session_data_to_file(data) {
        return false;
    }

    if !save_global_data_to_file(global_data) {
        return false;
    }

    if !PRINTED_MSG.load(Ordering::Relaxed) {
        PRINTED_MSG.store(true, Ordering::Relaxed);

        println!("Completed bosses: {}", data.bosses_done);
    }

    true
}

#[inline]
fn remove_color_codes(text: &str) -> String {
    text.replace("\u{a7}a", "")
        .replace("\u{a7}b", "")
        .replace("\u{a7}c", "")
        .replace("\u{a7}d", "")
        .replace("\u{a7}e", "")
        .replace("\u{a7}f", "")
        .replace("\u{a7}0", "")
        .replace("\u{a7}1", "")
        .replace("\u{a7}2", "")
        .replace("\u{a7}3", "")
        .replace("\u{a7}4", "")
        .replace("\u{a7}5", "")
        .replace("\u{a7}6", "")
        .replace("\u{a7}7", "")
        .replace("\u{a7}8", "")
        .replace("\u{a7}9", "")
        // Format codes
        .replace("\u{a7}k", "")
        .replace("\u{a7}l", "")
        .replace("\u{a7}m", "")
        .replace("\u{a7}n", "")
        .replace("\u{a7}o", "")
        .replace("\u{a7}r", "")
}

#[inline]
fn crop_letters(s: &str, pos: usize) -> &str {
    match s.char_indices().nth(pos) {
        #[allow(clippy::indexing_slicing, clippy::string_slice)]
        Some((position, _)) => &s[position..],
        None => "",
    }
}

#[inline]
fn crop_netty(mut s: String) -> String {
    for index in 0..=(num_cpus() * 2) {
        s = s.replace(
            &format!("] Netty Epoll Client IO #{index}/INFO] CHAT] "),
            "",
        );
    }

    s
}

#[inline]
fn remove_hook() {
    eprintln!(
        "{}",
        "warning: log file seems to be removed, this usually happens when the log is gzipping, stopping program..".yellow()
    );

    if let Err(e) = Notification::new()
        .summary("Watcher is stopping")
        .body("Please restart it manually if you want to continue watching slayer drops.")
        .urgency(Urgency::Critical)
        .show() {
        eprintln!("{}{e}", "error: can't send desktop notification: ".red());
    }
}

#[inline]
fn copy_to_clipboard(clipboard: &mut Clipboard, text: &str) {
    if let Err(e) = clipboard.set_text(text) {
        eprintln!("{}{e}", "error while setting clipboard contents: ".red());
    }
}

#[inline]
fn refresh_data_from_logs(
    session_data: &mut VoidgloomData,
    global_data: &mut VoidgloomData,
    clipboard: &mut Clipboard,
) -> bool {
    get_log_file_path().map_or_else(
        || {
            eprintln!("{}", "can't get log file path".yellow());

            false
        },
        |path| {
            lines_from_file_from_end(&path, 1, false).first().map_or_else(
                || {
                    eprintln!(
                        "{}",
                        "warning: can't get the added line from log file"
                            .yellow()
                    );

                    false
                },
                |added_log_message| {
                    if !added_log_message.contains("\u{a7}7:")
                        && !added_log_message.contains("\u{a7}f:")
                        && !added_log_message.contains("To")
                        && !added_log_message.contains("From")
                        && (added_log_message
                            .contains("SLAYER QUEST COMPLETE!")
                            || added_log_message
                                .replace("DROP!  ", "DROP! ")
                                .contains("DROP! "))
                    {
                        let session_data_orig = *session_data;
                        parse_log_line(
                            session_data,
                            global_data,
                            added_log_message,
                        );

                        let modified = *session_data != session_data_orig;

                        if modified
                            && !save_data_to_file(session_data, global_data)
                        {
                            eprintln!("{}", "Save failed".red());
                        }

                        if (added_log_message.contains("RARE DROP!")
                            || added_log_message.contains("INSANE DROP!")
                            || added_log_message.contains("PET DROP!"))
                            && !added_log_message
                                .contains("Enchanted Ender Pearl")
                            && !added_log_message.contains("Griffin Feather")
                            && !added_log_message.contains("Chimera")
                        // Copy the Enchanted Book one as it includes magic
                        // find and its cooler
                        {
                            copy_to_clipboard(
                            clipboard,
                            &crop_netty(
                                crop_letters(
                                    &remove_color_codes(added_log_message)
                                        .replace(
                                            "] [Client thread/INFO]: [CHAT] ",
                                            "",
                                        )
                                        .replace(['[', ':'], "")
                                        .replace(
                                            "RARE DROP!  ",
                                            "RARE DROP! ",
                                        ),
                                    6,
                                )
                                .to_owned(),
                            ),
                        );
                        }
                    }

                    true
                },
            )
        },
    )
}

#[inline]
fn parse_log_line(
    session_data: &mut VoidgloomData,
    global_data: &mut VoidgloomData,
    added_log_message: &str,
) {
    #[allow(clippy::else_if_without_else)]
    if added_log_message.contains("SLAYER QUEST COMPLETE!") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.bosses_done += 1);

        PRINTED_MSG.store(false, Ordering::Relaxed);
    } else if added_log_message.contains("Twilight Arrow Poison") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.twilight_arrow_poisons += 1);
    } else if added_log_message.contains("Endersnake Rune") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.endersnake_runes += 1);
    }
    // The drop rarity of eye from boss can change depending on if
    // loot-share or own boss, or if it is  selected on meter,
    // so avoid the drop rarity and just check containing DROP!. Why
    // not just Summoning Eye? Because Zealot dropped eyes give that
    // message too.

    // The last not containing RARE DROP! check is for when you drop
    // eye from Voidling Extremists, Zealots or Zealot Bruisers.

    // The replacing of DROP! with 2 spaces into DROP! with 1 space is that
    // well, Hypixel sends the message with 2 spaces, or some mod on my end
    // modifies it, no idea.
    else if added_log_message
        .replace("DROP!  ", "DROP! ")
        .contains("DROP! (Summoning Eye)")
        && (!added_log_message.contains("RARE DROP!")
            || (added_log_message.contains("VERY RARE DROP!")
                || added_log_message.contains("CRAZY RARE DROP!")))
    {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.summoning_eyes += 1);
    } else if added_log_message.contains("Mana Steal I") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.mana_steals += 1);
    } else if added_log_message.contains("Transmission Tuner") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.transmission_tuners += 1);
    } else if added_log_message.contains("Null Atom") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.null_atoms += 1);
    } else if added_log_message.contains("Hazmat Enderman") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.hazmat_endermans += 1);
    } else if added_log_message.contains("Pocket Espresso Machine") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.espressos += 1);
    } else if added_log_message.contains("Smarty Pants I") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.smarty_pants += 1);
    } else if added_log_message.contains("End Rune") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.end_runes += 1);
    } else if added_log_message.contains("Handy Blood Chalice") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.handy_blood_chalices += 1);
    } else if added_log_message.contains("Sinful Dice") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.sinful_dices += 1);
    } else if added_log_message
        .contains("Exceedingly Rare Ender Artifact Upgrader")
    {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.artifact_upgraders += 1);
    } else if added_log_message.contains("Etherwarp Merger") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.etherwarp_mergers += 1);
    } else if added_log_message.contains("Void Conqueror Enderman Skin") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.void_conqueror_skins += 1);
    } else if added_log_message.contains("Judgement Core") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.judgement_cores += 1);
    } else if added_log_message.contains("Enchant Rune") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.enchant_runes += 1);
    } else if added_log_message.contains("Ender Slayer VII") {
        [session_data, global_data]
            .iter_mut()
            .for_each(|data| data.ender_slayer_tier_sevens += 1);
    }
}

#[inline]
fn async_watcher(
) -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                match tx.send(res).await {
                    Ok(()) => {},

                    Err(e) => {
                        eprintln!("{}{e}", "watch send error: ".red());
                    },
                }
            });
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

#[inline]
async fn async_watch<P: AsRef<Path> + Send>(
    path: P,
    session_data: &mut VoidgloomData,
    global_data: &mut VoidgloomData,
    clipboard: &mut Clipboard,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        if let Some(res) = rx.next().await {
            match res {
                Ok(event) =>
                    if event.kind.is_modify() {
                        if !refresh_data_from_logs(
                            session_data,
                            global_data,
                            clipboard,
                        ) {
                            remove_hook();

                            println!("stopping watching as requested");
                            break;
                        }
                    } else if event.kind.is_remove() || event.kind.is_create()
                    {
                        remove_hook();
                        break;
                    } else if event.kind.is_access() {
                        // Do nothing if not modified
                    } else if event.kind.is_other() {
                        eprintln!(
                            "{}{:#?}",
                            "warning: unsupported event received".yellow(),
                            event
                        );
                    } else {
                        eprintln!(
                            "{}{:#?}",
                            "warning: unknown event received".yellow(),
                            event
                        );
                    },
                Err(e) => eprintln!("{}{e}", "watch error: ".red()),
            }
        } else {
            remove_hook();

            eprintln!("{}", "no events left to receive, breaking".yellow());
            break;
        }
    }

    Ok(())
}
