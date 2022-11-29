// Enables lints disabled (allowed) by default to (possibly) catch more code
// errors/smells https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html

#![warn(absolute_paths_not_starting_with_crate)]
#![warn(box_pointers)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![feature(c_unwind)]
#![warn(ffi_unwind_calls)]
#![feature(strict_provenance)]
#![warn(fuzzy_provenance_casts)]
#![warn(lossy_provenance_casts)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![feature(must_not_suspend)]
#![warn(must_not_suspend)]
#![warn(non_ascii_idents)]
#![feature(non_exhaustive_omitted_patterns_lint)]
#![warn(non_exhaustive_omitted_patterns)]
#![warn(noop_method_call)]
#![warn(pointer_structural_match)]
#![warn(rust_2021_incompatible_closure_captures)]
#![warn(rust_2021_incompatible_or_patterns)]
#![warn(rust_2021_prefixes_incompatible_syntax)]
#![warn(rust_2021_prelude_collisions)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_macro_rules)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(unused_tuple_struct_fields)]
#![warn(variant_size_differences)]
#![feature(stmt_expr_attributes)]
#![feature(const_fn_floating_point_arithmetic)]

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use colored::Colorize;
use cookie_store as _;
use mimalloc::MiMalloc;
use trust_dns_resolver as _;

use log::debug;

mod minecraft_launcher_launcher;

mod constants;
mod utils;

mod catacombs_stat_boost_calculator;
mod master_skull_upgrade_helper;

mod rng_simulator;

mod survivability_calculator;

mod slayer_kill_goal_watcher;

#[cfg(test)]
mod tests;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[inline]
fn print_selections() {
    println!("Select which utility you want to run: ");
    println!(
        " {}. Upgrade price calculator for {}",
        "1".bright_blue(),
        "Master Skulls".bright_red()
    );
    println!(" {}. Catacombs stat boost calculator", "2".bright_blue());
    println!(" {}. RNG simulator", "3".bright_blue());
    println!(" {}. Survivability Calculator", "4".bright_blue());
    println!(" {}. Slayer kill goal watcher", "5".bright_blue());

    println!();
}

#[tokio::main]
#[inline]
async fn main() -> ExitCode {
    pretty_env_logger::init();
    debug!(
        "program version is {}",
        option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
    );

    debug!(
        "environment variables are {:#?}",
        env::vars().collect::<HashMap<String, String>>()
    );

    let args: Vec<String> = env::args().collect();

    debug!("given commandline arguments are {:#?}", args);

    if let Some(binary_name) = args.first() {
        debug!("binary name is {}", binary_name);

        if let Some(binary_file_name) = Path::new(binary_name).file_name() {
            if let Some(argument) = args.get(1) {
                if argument == "install-minecraft-launcher-launcher" {
                    return minecraft_launcher_launcher::install(
                        &binary_file_name.to_string_lossy(),
                        &args,
                    );
                }

                eprintln!("{}{argument}", "invalid argument: ".red());

                return ExitCode::FAILURE; // Exit because providing invalid
                                          // arguments should not fall through
            } // No arguments given, fall through to hypixel skyblock tools

            if binary_file_name == "minecraft-launcher" {
                // I'm too lazy to maintain 2 projects so this goes here even
                // though its basically another project
                return minecraft_launcher_launcher::launch();
            } // Binary name is not minecraft-launcher so
              // assume user wants the hypixel skyblock tools and
              // fall through
        } else {
            eprintln!(
                "{}",
                "warning: can't get file name path of running binary".yellow()
            );
        }
    } else {
        eprintln!("{}", "warning: can't get running binary string".yellow());
        // Fall through because we don't really need the binary name
    }

    let start = Instant::now();
    let start_without_user_input: &mut Option<Instant> = &mut Some(start);

    print_selections();

    let selection =
        utils::ask_int_input("Enter a number to select: ", Some(1), Some(5));

    if selection == 1
        && !master_skull_upgrade_helper::upgrade_calculator_for_master_skulls(
            start_without_user_input,
        )
        .await
    {
        eprintln!("Exiting with failure exit code");
        return ExitCode::FAILURE;
    }

    if selection == 2
        && !catacombs_stat_boost_calculator::catacombs_stat_boost_calculator(
            start_without_user_input,
        )
    {
        eprintln!("Exiting with failure exit code");
        return ExitCode::FAILURE;
    }

    if selection == 3
        && !rng_simulator::rng_simulator(start_without_user_input)
    {
        eprintln!("Exiting with failure exit code");
        return ExitCode::FAILURE;
    }

    if selection == 4
        && !survivability_calculator::survivability_calculator(
            start_without_user_input,
        )
    {
        eprintln!("Exiting with failure exit code");
        return ExitCode::FAILURE;
    }

    if selection == 5
        && !slayer_kill_goal_watcher::slayer_kill_goal_watcher(
            start_without_user_input,
        )
    {
        eprintln!("Exiting with failure exit code");
        return ExitCode::FAILURE;
    }

    let elapsed = start.elapsed();
    let mut elapsed_without_user_input = elapsed;

    if let Some(start_no_user_input) = *start_without_user_input {
        elapsed_without_user_input = start_no_user_input.elapsed();
    }

    println!();
    println!(
        "Program finished, took {:.2?} (without user input {:.2?}), exiting..",
        elapsed, elapsed_without_user_input
    );

    ExitCode::SUCCESS
}
