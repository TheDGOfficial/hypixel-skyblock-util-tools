// Enables lints disabled (allowed) by default to (possibly) catch more code
// errors/smells https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html

#![warn(absolute_paths_not_starting_with_crate)]
#![warn(box_pointers)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
//#![feature(ffi_unwind_calls)]
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
//#![warn(missing_docs)]
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

//#[cfg(target_env = "msvc")]
use mimalloc::MiMalloc;

//#[cfg(target_env = "msvc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

//#[cfg(not(target_env = "msvc"))]
// use jemallocator::Jemalloc;

//#[cfg(not(target_env = "msvc"))]
//#[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

use cookie_store as _;
use trust_dns_resolver as _;

use core::cmp::min;

use futures::stream::FuturesOrdered;
use futures::StreamExt;

use fxhash::FxHashMap;
use fxhash::FxHashSet;

use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process::ExitCode;

use colored::Colorize;
use core::hash::Hash;
use core::time::Duration;
use jandom::Random;
use std::time::Instant;

#[tokio::main]
async fn main() -> ExitCode {
    let start = Instant::now();
    let start_without_user_input: &mut Option<Instant> = &mut Some(start);

    println!("Select which utility you want to run: ");
    println!(
        " {}. Upgrade price calculator for {}",
        "1".bright_blue(),
        "Master Skulls".bright_red()
    );
    println!(" {}. Catacombs stat boost calculator", "2".bright_blue());
    println!(" {}. RNG simulator", "3".bright_blue());

    println!();

    let selection = ask_int_input("Enter a number to select: ", Some(1), Some(3));

    if selection == 1 &&
        !upgrade_calculator_for_master_skulls(start_without_user_input).await
    {
        return ExitCode::FAILURE;
    }

    if selection == 2 &&
        !catacombs_stat_boost_calculator(start_without_user_input)
    {
        return ExitCode::FAILURE;
    }

    if selection == 3 && !rng_simulator(start_without_user_input) {
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

const CHIMERA_DROP_CHANCE: f64 = 1.0;
const JUDGEMENT_CORE_DROP_CHANCE: f64 = 0.0565;
const WARDEN_HEART_DROP_CHANCE: f64 = 0.0130;
const OVERFLUX_CAPACITOR_DROP_CHANCE: f64 = 0.0406;
const NECRONS_HANDLE_DROP_CHANCE: f64 = 0.0964;
const NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE: f64 = 0.1106;

fn rng_simulator(start_without_user_input: &mut Option<Instant>) -> bool {
    println!();
    println!("Select which item you want to simulate RNG: ");

    println!(" {}. Chimera (%{CHIMERA_DROP_CHANCE})", "1".bright_blue());
    println!(
        " {}. Judgement Core (%{JUDGEMENT_CORE_DROP_CHANCE})",
        "2".bright_blue()
    );
    println!(
        " {}. Warden Heart (%{WARDEN_HEART_DROP_CHANCE})",
        "3".bright_blue()
    );
    println!(
        " {}. Overflux Capacitor (%{OVERFLUX_CAPACITOR_DROP_CHANCE})",
        "4".bright_blue()
    );
    println!(
        " {}. Necron's Handle (%{NECRONS_HANDLE_DROP_CHANCE})",
        "5".bright_blue()
    );
    println!(" {}. Necron's Handle (Master Mode) (%{NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE})", "6".bright_blue());
    println!(" {}. Custom", "7".bright_blue());

    println!();

    let selection = ask_int_input("Enter a number to select: ", Some(1), Some(7));

    let mut no_looting = true;

    let mut drop_chance = match selection {
        1 => {
            no_looting = false;

            CHIMERA_DROP_CHANCE
        },
        2 => JUDGEMENT_CORE_DROP_CHANCE,
        3 => WARDEN_HEART_DROP_CHANCE,
        4 => OVERFLUX_CAPACITOR_DROP_CHANCE,
        5 => NECRONS_HANDLE_DROP_CHANCE,
        6 => NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE,
        7 => {
            no_looting = false;

            ask_float_input("Enter custom drop chance: ", None, None)
        },

        _ => {
            println!("{}{selection}", "error: invalid selection: ".red());

            0.0
        },
    };

    let original_drop_chance = drop_chance;
    let mut rng_meter_percent = -1.0;

    if selection != 1 && selection != 7 {
        rng_meter_percent = ask_float_input(
            "Enter your current RNG meter completion percentage for this drop: ",
            Some(0.0),
            Some(100.0),
        );

        if rng_meter_percent >= 100.0 {
            drop_chance = 100.0;
        } else {
            let multiplier = 1.0 + ((2.0 * rng_meter_percent) / 100.0);

            drop_chance *= multiplier;
        }
    }

    let magic_find =
        ask_int_input("What is your Magic Find? (0-900): ", Some(0), Some(900));

    let looting_extra_chance = 15 *
        conditional_value_or_default(
            !no_looting,
            || {
                ask_int_input("What is your Looting level? (if it works on this drop, 0-5): ", Some(0), Some(5))
            },
            0,
        );

    let rolls = ask_int_input("How many rolls you want to do?: ", Some(0), None);

    *start_without_user_input = Some(Instant::now());

    let drop_rate_with_magic_find =
        drop_chance + percent_of(drop_chance, f64::from(magic_find));

    let drop_rate_with_magic_find_and_looting = drop_rate_with_magic_find +
        percent_of(drop_rate_with_magic_find, f64::from(looting_extra_chance));

    let odds = 100.0 / drop_rate_with_magic_find_and_looting;

    println!();
    println!(
        "Odds with Magic Find and Looting: {}/{}. Rolling {} times:",
        "1".bright_green(),
        odds.to_string().bright_red(),
        rolls.to_string().yellow()
    );
    println!();

    let all_succeeded_magic_find_values: &mut Vec<i32> = &mut vec![];
    let meter_succeeded_rolls: &mut Vec<i32> = &mut vec![];

    let drops = do_rolls_and_get_drops(
        original_drop_chance,
        rng_meter_percent,
        looting_extra_chance,
        rolls,
        magic_find,
        all_succeeded_magic_find_values,
        meter_succeeded_rolls,
    );

    let max_drops = all_succeeded_magic_find_values.len();

    let percent =
        100.0 - f64::abs(percentage_change(max_drops as f64, f64::from(drops)));

    if rolls > 0 {
        println!();
    }

    println!("Out of {rolls} rolls, {drops} rolls succeeded.");

    if !percent.is_nan() {
        println!("You got %{} of the possible drops ({drops}/{max_drops}) with maximum magic find, with your magic find.", percent.to_string().yellow());
    }

    if !all_succeeded_magic_find_values.is_empty() {
        print_statistics(
            100.0 / original_drop_chance,
            all_succeeded_magic_find_values,
            meter_succeeded_rolls,
            rng_meter_percent,
        );
    }

    true
}

fn print_statistics(
    odds: f64, all_succeeded_magic_find_values: &mut Vec<i32>,
    meter_succeeded_rolls: &mut Vec<i32>, original_rng_meter: f64,
) {
    println!();

    let mean_succeed_magic_find = mean(all_succeeded_magic_find_values);
    let median_succeed_magic_find = median(all_succeeded_magic_find_values);

    println!("Mean (Average) Succeed Magic Find: {mean_succeed_magic_find}");
    println!("Median (Middle) Succeed Magic Find: {median_succeed_magic_find}");

    if !has_unique_elements(all_succeeded_magic_find_values) {
        if let Some(mode_succeed_magic_find) =
            mode(all_succeeded_magic_find_values)
        {
            println!(
                "Mode (Most Repeated) Succeed Magic Find: {mode_succeed_magic_find}"
            );
        }
    }

    if all_succeeded_magic_find_values.len() > 1 {
        if let Some(range_succeed_magic_find) =
            range(all_succeeded_magic_find_values)
        {
            println!("Range (Difference between smallest and highest) Succeed Magic Findd: {range_succeed_magic_find}");
        }
    }

    println!();

    if original_rng_meter == -1.0 {
        println!("{}: The RNG Meter doesn't work on this drop type, so values below are based on if the RNG meter existed as a percentage to expected amount of rolls to get the drop, but didn't actually guarantee drops or modify chances.", "Note".red());
        println!();
    }

    let mean_succeed_rolls = mean(meter_succeeded_rolls);
    let median_succeed_rolls = median(meter_succeeded_rolls);

    let mean_succeed_meter =
        100.0 - f64::abs(percentage_change(odds, cap(mean_succeed_rolls, odds)));

    println!(
        "Mean (Average) Amount of Rolls until Succeed: {mean_succeed_rolls} (%{mean_succeed_meter} RNG Meter)"
    );

    let median_succeed_meter = 100.0 -
        f64::abs(percentage_change(odds, cap(median_succeed_rolls, odds)));

    println!(
        "Median (Middle) Amount of Rolls until Succeed: {median_succeed_rolls} (%{median_succeed_meter} RNG Meter)"
    );

    if !has_unique_elements(meter_succeeded_rolls) {
        if let Some(mode_succeed_rolls) = mode(meter_succeeded_rolls) {
            let mode_succeed_meter = 100.0 -
                f64::abs(percentage_change(
                    odds,
                    cap(f64::from(mode_succeed_rolls), odds),
                ));

            println!(
                "Mode (Most Repeated) Amount of Rolls until Succeed: {mode_succeed_rolls} (%{mode_succeed_meter} RNG Meter)"
            );
        }
    }

    if meter_succeeded_rolls.len() > 1 {
        if let Some(range_succeed_rolls) = range(meter_succeeded_rolls) {
            let range_succeed_meter = 100.0 -
                f64::abs(percentage_change(
                    odds,
                    cap(f64::from(range_succeed_rolls), odds),
                ));

            println!("Range (Difference between smallest and highest) Amount of Rolls until Succeed: {range_succeed_rolls} (%{range_succeed_meter} RNG Meter)");
        }
    }

    if let Some(max) = meter_succeeded_rolls.iter().max() {
        let max_meter = 100.0 -
            f64::abs(percentage_change(
                odds,
                cap(f64::from(max.to_owned()), odds),
            ));
        println!("Maximum Amount of Rolls before Succeed: {max} (%{max_meter} RNG Meter)");
    }
}

fn has_unique_elements<T>(iter: &T) -> bool
where
    T: IntoIterator + Clone,
    T::Item: Eq + Hash,
{
    let mut unique = FxHashSet::default();
    iter.clone().into_iter().all(move |x| unique.insert(x))
}

// fn generate_java_seed() -> i64 {
// i64::try_from(
// 3_447_679_086_515_839_964 ^
// SystemTime::now()
// .duration_since(UNIX_EPOCH)
// .expect("Time went backwards")
// .as_nanos(),
// )
// .expect("error: failed converting u128 to i64 to generate java seed")
// }

fn cap(number: f64, cap: f64) -> f64 {
    if number > cap {
        return cap;
    }

    number
}

fn do_rolls_and_get_drops(
    original_drop_chance: f64, original_rng_meter_percent: f64,
    looting_extra_chance: i32, rolls: i32, magic_find: i32,
    all_succeeded_magic_find_values: &mut Vec<i32>,
    meter_succeeded_rolls: &mut Vec<i32>,
) -> i32 {
    let mut drops = 0;
    // let mut rand = Random::new(generate_java_seed());
    let mut rand = Random::default();

    let mut reset_meter_at_least_once = false;
    let mut last_reset_at = 0;

    for roll in 1..=rolls {
        let odds = 100.0 / original_drop_chance;
        let original_rng_meter_progress =
            percent_of(odds, original_rng_meter_percent);

        let progress = if reset_meter_at_least_once {
            f64::from(roll - last_reset_at)
        } else {
            original_rng_meter_progress + f64::from(roll)
        }
        .round() as i32;

        let rng_meter_percent = 100.0 -
            f64::abs(percentage_change(odds, cap(f64::from(progress), odds)));

        let final_drop_chance = if rng_meter_percent >= 100.0 {
            100.0
        } else {
            let multiplier = if original_rng_meter_percent == -1.0 {
                1.0
            } else {
                1.0 + ((2.0 * rng_meter_percent) / 100.0)
            };

            original_drop_chance * multiplier
        };

        let drop_rate_with_magic_find = final_drop_chance +
            percent_of(final_drop_chance, f64::from(magic_find));

        let new_drop_rate_with_magic_find_and_looting = drop_rate_with_magic_find +
            percent_of(
                drop_rate_with_magic_find,
                f64::from(looting_extra_chance),
            );

        let magic_number = rand.next_f64();
        let success =
            if magic_number < new_drop_rate_with_magic_find_and_looting / 100.0 {
                drops += 1;

                reset_meter_at_least_once = true;
                last_reset_at = roll;

                meter_succeeded_rolls.push(progress);

                true
            } else {
                false
            };

        let mut minimum_magic_find_needed_to_success = 901;

        for mf in 0..=900 {
            let drop_rate_with_this_magic_find =
                final_drop_chance + percent_of(final_drop_chance, f64::from(mf));
            let drop_rate_with_this_magic_find_and_looting =
                drop_rate_with_this_magic_find +
                    percent_of(
                        drop_rate_with_this_magic_find,
                        f64::from(looting_extra_chance),
                    );

            if magic_number < drop_rate_with_this_magic_find_and_looting / 100.0 &&
                mf < minimum_magic_find_needed_to_success
            {
                minimum_magic_find_needed_to_success = mf;
            }
        }

        if minimum_magic_find_needed_to_success == 901 {
            // println!(
            // "Roll #{}: {}, can't succeed even with max Magic Find.",
            // roll.to_string().yellow(),
            // "FAIL".bright_red()
            // );
        } else {
            all_succeeded_magic_find_values
                .push(minimum_magic_find_needed_to_success);

            if success {
                println!(
                    "Roll #{}: {}, minimum magic find to succeed is {}. RNG Meter: %{}",
                    roll.to_string().yellow(),
                    "PASS".bright_green(),
                    minimum_magic_find_needed_to_success.to_string().green(),
                    rng_meter_percent
                );
            } else {
                println!("Roll #{}: {}, minimum magic find to succeed is {} which is higher than yours.", roll.to_string().yellow(), "FAIL".bright_red(), minimum_magic_find_needed_to_success.to_string().bright_red());
            }
        }
    }

    drops
}

fn mean(array: &Vec<i32>) -> f64 {
    f64::from(array.iter().sum::<i32>()) / array.len() as f64 // TODO use conv
                                                              // crate instead
                                                              // and error on
                                                              // overflow on all
                                                              // conversions
}

fn median(array: &mut Vec<i32>) -> f64 {
    array.sort_unstable();

    if array.len() % 2 == 0 {
        let left = array.len() / 2 - 1;
        let right = array.len() / 2;

        #[allow(clippy::indexing_slicing)]
        {
            f64::from(array[left] + array[right]) / 2.0
        }
    } else {
        #[allow(clippy::indexing_slicing)]
        {
            f64::from(array[(array.len() / 2)])
        }
    }
}

// Returns the most occurring value in an array.
// Returns None if the array is empty.
fn mode(array: &Vec<i32>) -> Option<i32> {
    let mut occurrences = FxHashMap::default();

    for &value in array {
        *occurrences.entry(value).or_insert(0) += 1;
    }

    occurrences
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(val, _)| val)
}

// Returns difference between maximum and minimum values in an array.
// Returns None if the array is empty.
fn range(array: &[i32]) -> Option<i32> {
    if let Some(min) = array.iter().min() {
        if let Some(max) = array.iter().max() {
            return Some(max - min);
        }
    }

    None
}

fn conditional_value_or_default<T>(
    condition: bool, value: fn() -> T, default: T,
) -> T {
    if condition {
        return value();
    }

    default
}

fn catacombs_stat_boost_calculator(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    let catacombs_boost = get_cata_stat_boost(ask_int_input(
        "Enter your current Catacombs level: ",
        Some(0),
        Some(50),
    ));

    let normal_stars_boost = 10 *
        ask_int_input(
            "Enter the amount of normal stars your gear has: ",
            Some(0),
            Some(5),
        );

    let master_stars_boost = 5 * ask_int_input(
        "Enter the amount of master stars your gear has: ",
        Some(0),
        Some(5),
    );

    let total_stat_boost =
        catacombs_boost + normal_stars_boost + master_stars_boost;

    let planned_catacombs_level_boost = get_cata_stat_boost(ask_int_input(
        "Enter your planned Catacombs Level: ",
        Some(0),
        Some(50),
    ));

    let planned_normal_stars_boost = 10 *
        ask_int_input(
            "Enter the amount of normal stars you plan your gear to have: ",
            Some(0),
            Some(5),
        );

    let planned_master_stars_boost = 5 * ask_int_input(
        "Enter the amount of master stars you plan your gear to have: ",
        Some(0),
        Some(5),
    );

    let planned_total_stat_boost = planned_catacombs_level_boost +
        planned_normal_stars_boost +
        planned_master_stars_boost;

    match f64::try_from(total_stat_boost) {
        Ok(total_now) => match f64::try_from(planned_total_stat_boost) {
            Ok(total_planned) => {
                println!();
                println!("{}{}{}", "Difference between your current and planned Catacombs level and Stars/Master Stars in percent is %".bright_green(), percentage_change(total_now, total_planned).to_string().bright_yellow(), ".".white());
            },

            Err(e) => {
                println!("{}{e}", "Error converting i32 to f64: ".red());

                return false;
            },
        },

        Err(e) => {
            println!("{}{e}", "Error converting i32 to f64: ".red());

            return false;
        },
    }

    *start_without_user_input = Some(Instant::now());

    true
}

// fn percent_of(number: f64, percent: f64) -> f64 {
//    (number / 100.0) * percent
//}

fn percent_of(number: f64, percent: f64) -> f64 {
    (number / 100.0) * percent
}

fn percentage_change(starting_number: f64, ending_number: f64) -> f64 {
    ((ending_number - starting_number) / f64::abs(starting_number)) * 100.0
}

fn get_cata_stat_boost(catacombs_level: i32) -> i32 {
    let mut cata_stat_boost = 0;

    for level in 1..=catacombs_level {
        cata_stat_boost += match level {
            1..=5 => 4,
            6..=10 => 5,
            11..=15 => 6,
            16..=20 => 7,
            21..=25 => 8,
            26..=30 => 9,
            31..=35 => 10,
            36..=40 => 12,
            41..=45 => 14,

            46 => 16,
            47 => 17,
            48 => 18,
            49 => 19,
            50 => 20,

            _ => {
                println!("{}{level}", "error: invalid catacombs level: ".red());

                0
            },
        };
    }

    cata_stat_boost
}

async fn upgrade_calculator_for_master_skulls(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    let current_tier =
        ask_int_input("Enter your current Master Skull tier: ", Some(1), Some(7));
    let minimum_upgrade_tier = min(current_tier + 1, 7);

    let target_tier = if minimum_upgrade_tier == 7 {
        7
    } else {
        ask_int_input(
            "Enter your target Master Skull tier: ",
            Some(minimum_upgrade_tier),
            Some(7),
        )
    };

    *start_without_user_input = Some(Instant::now());

    if current_tier == target_tier {
        println!(
            "{}",
            "You already have the Tier 7 Master Skull, exiting.".bright_green()
        );

        return true;
    }

    let mut prices = FxHashMap::default();

    let critical_error_occurred =
        !do_requests_and_extract_prices(&mut prices).await;

    let mut lowest_price_per_tier_one_so_far = i64::MAX;
    let mut best_tier_to_buy_and_combine = 0;

    println!();

    for tier in 1..8 {
        prices.get(&tier).map_or_else(|| {
            if !critical_error_occurred {
                println!("{}{}{}", "No one is selling Master Skull - Tier ".bright_red(), tier.to_string().bright_red(), "!".bright_red());
            }
        }, |price| {
            let previous_tier = tier - 1;
            let mut tier_ones_required_to_craft_this_tier = 1;

            for _ in 0..(tier - 1) {
                tier_ones_required_to_craft_this_tier *= 4;
            }

            let price_per_tier_one = price / tier_ones_required_to_craft_this_tier;

            if price_per_tier_one < lowest_price_per_tier_one_so_far {
                lowest_price_per_tier_one_so_far = price_per_tier_one;
                best_tier_to_buy_and_combine = tier;
            }

            if tier == 1 {
                println!("Master Skull - Tier {tier} is priced {}", with_comma_separators(&price.to_string()).unwrap_or_else(|| price.to_string()).yellow());
            } else {
                println!("Master Skull - Tier {tier} is priced {}, equals to {} coins for 4x Tier {previous_tier} skulls, or {} coins for {tier_ones_required_to_craft_this_tier}x Tier 1 skulls", with_comma_separators(&price.to_string()).unwrap_or_else(|| price.to_string()).yellow(), with_comma_separators(&(price / 4).to_string()).unwrap_or_else(|| (price / 4).to_string()).yellow(), with_comma_separators(&price_per_tier_one.to_string()).unwrap_or_else(|| price_per_tier_one.to_string()).yellow());
            }
        });
    }

    let mut total_required_amount = 1;

    match usize::try_from(target_tier) {
        Ok(tier) =>
            for _ in best_tier_to_buy_and_combine..tier {
                total_required_amount *= 4;
            },

        Err(e) => {
            println!("{}{e}", "Error converting i32 to usize: ".red());
        },
    }

    let mut total_required_amount_for_current = 1;

    match usize::try_from(current_tier) {
        Ok(tier) =>
            for _ in best_tier_to_buy_and_combine..tier {
                total_required_amount_for_current *= 4;
            },

        Err(e) => {
            println!("{}{e}", "Error converting i32 to usize: ".red());
        },
    }

    total_required_amount -= total_required_amount_for_current;

    prices.get(&best_tier_to_buy_and_combine).map_or_else(|| {
        if !critical_error_occurred {
            println!("{}", "Can't find a best tier to buy and combine. No one selling any Master Skulls at all?".bright_red());
        }
    }, |price| {
        let upgrade_cost = price * total_required_amount;

        println!();
        println!("The best tier to buy and combine is Tier {best_tier_to_buy_and_combine}. To upgrade from Master Skull - Tier {current_tier} to Master Skull - Tier {target_tier} combining Master Skull - Tier {best_tier_to_buy_and_combine}s, you need to buy and combine {total_required_amount}x of Master Skull - Tier {best_tier_to_buy_and_combine}s, which would cost you {} coins.", with_comma_separators(&upgrade_cost.to_string()).unwrap_or_else(|| upgrade_cost.to_string()).yellow());
    });

    if critical_error_occurred {
        println!(
            "{}",
            "Critical error(s) occurred while running the program. Please read above for details.".red()
        );

        return false;
    }

    true
}

async fn do_requests_and_extract_prices(
    prices: &mut FxHashMap<usize, i64>,
) -> bool {
    let mut requests = Vec::new();

    match reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .brotli(true)
        .build()
    {
        Ok(client) =>
            for i in 1..8 {
                let id = format!("MASTER_SKULL_TIER_{i}");

                requests.push(
                    client
                        .get("https://api.slothpixel.me/api/skyblock/auctions")
                        .query(&vec![
                            ("limit", "1"),
                            ("page", "1"),
                            ("sortOrder", "asc"),
                            ("sortBy", "starting_bid"),
                            ("id", &id),
                            ("bin", "true"),
                            ("category", "accessories"),
                        ])
                        .timeout(Duration::from_secs(10))
                        .header("Accept", "application/json; charset=utf-8")
                        .header("Accept-Encoding", "br")
                        .header("Accept-Language", "en-US")
                        .header("Connection", "keep-alive")
                        .header("DNT", "1")
                        .header("Upgrade-Insecure-Requests", "1")
                        .header("User-Agent", "Mozilla/5.0")
                        .send(),
                );
            },

        Err(e) => {
            println!("{}{e}", "Error when building http client: ".red());
        },
    }

    let mut completion_stream = requests
        .into_iter()
        .map(tokio::spawn)
        .collect::<FuturesOrdered<_>>();
    let mut i = 0;

    while let Some(result_of_task) = completion_stream.next().await {
        match result_of_task {
            Ok(result_of_request) => {
                match result_of_request {
                    Ok(response) => {
                        match response.text().await {
                            Ok(response_body) => {
                                match serde_json::from_str::<Value>(
                                    &response_body,
                                ) {
                                    Ok(json) => {
                                        json.get("matching_query").map_or_else(|| {
                                    println!("{}{response_body}", "error: can't find matching_query field in JSON: ".red());
                                }, |matching_query| {
                                    matching_query.as_i64().map_or_else(|| {
                                        println!("{}{matching_query}", "error: matching_query field value is not an i64: ".red());
                                    }, |matches| {
                                        if matches >= 1 { // Available for sale
                                            json.get("auctions").map_or_else(|| {
                                                println!("{}{response_body}", "error: can't find auctions field in JSON: ".red());
                                            }, |auctions| {
                                                auctions.as_array().map_or_else(|| {
                                                    println!("{}{auctions}", "error: auctions field is not an array: ".red());
                                                }, |auctions_array| {
                                                    auctions_array.get(0).map_or_else(|| {
                                                        println!("{}{response_body}", "error: can't find the first auction in the auctions list while matching_query was >= 1: ".red());
                                                    }, |auction| {
                                                        auction.as_object().map_or_else(|| {
                                                            println!("{}{auction}", "error: auction data is not a Map: ".red());
                                                        }, |auction_map| {
                                                            auction_map.get("starting_bid").map_or_else(|| {
                                                                println!("{}{response_body}", "error: can't find starting_bid field in auction JSON: ".red());
                                                            }, |starting_bid| {
                                                                starting_bid.as_i64().map_or_else(|| {
                                                                    println!("{}{starting_bid}", "error: starting_bid field is not an i64: ".red());
                                                                }, |price| {
                                                                    if prices.insert(i + 1, price).is_some() {
                                                                        println!("error: duplicate value at index {}, updating the value and continuing", i + 1);
                                                                    }
                                                                });
                                                            });
                                                        });
                                                    });
                                                });
                                            });
                                        }
                                    });
                                });
                                    },

                                    Err(e) => {
                                        println!(
                                            "{}{e}",
                                            "Error when parsing JSON: ".red()
                                        );

                                        return false;
                                    },
                                }
                            },

                            Err(e) => {
                                println!(
                                    "{}{e}",
                                    "Error when getting response body: ".red()
                                );

                                return false;
                            },
                        }
                    },

                    Err(e) => {
                        println!("{}{e}", "Error when getting response: ".red());

                        return false;
                    },
                }
            },

            Err(e) => {
                println!("{}{e}", "Error on task execution: ".red());

                return false;
            },
        }

        i += 1;
    }

    true
}

/// Add thousands comma separators to a number. The number must match the
/// following regex: `^-?\d*(\.\d*)?$`. Returns None if it does not match that
/// format. Note that empty strings and just `-` are allowed.
fn with_comma_separators(s: &str) -> Option<String> {
    // Position of the `.`
    let dot = s.bytes().position(|c| c == b'.').unwrap_or(s.len());
    // Is the number negative (starts with `-`)?
    let negative = s.bytes().next() == Some(b'-');
    // The dot cannot be at the front if it is negative.
    assert!(!(negative && dot == 0));
    // Number of integer digits remaining (between the `-` or start and the `.`).
    let mut integer_digits_remaining = dot - usize::from(negative);
    // Output. Add capacity for commas. It's a slight over-estimate but that's
    // fine.
    let mut out = String::with_capacity(s.len() + integer_digits_remaining / 3);

    // We can iterate on bytes because everything must be ASCII. Slightly faster.
    for (i, c) in s.bytes().enumerate() {
        match c {
            b'-' => {
                // `-` can only occur at the start of the string.
                if i != 0 {
                    return None;
                }
            },
            b'.' => {
                // Check we only have a dot at the expected position.
                // This return may happen if there are multiple dots.
                if i != dot {
                    return None;
                }
            },
            b'0'..=b'9' => {
                // Possibly add a comma.
                if integer_digits_remaining > 0 {
                    // Don't add a comma at the start of the string.
                    if i != usize::from(negative) &&
                        integer_digits_remaining % 3 == 0
                    {
                        out.push(',');
                    }
                    integer_digits_remaining -= 1;
                }
            },
            _ => {
                // No other characters allowed.
                return None;
            },
        }
        out.push(char::from(c));
    }
    Some(out)
}

// fn debug_print_json_value_type(value: &Value) {
// println!("is null? {}", value.is_null());
// println!("is array? {}", value.is_array());
// println!("is string? {}", value.is_string());
// println!("is i64? {}", value.is_i64());
// println!("is u64? {}", value.is_u64());
// println!("is f64? {}", value.is_f64());
// println!("is boolean? {}", value.is_boolean());
// println!("is number? {}", value.is_number());
// println!("is is object? {}", value.is_object());
// }

fn print(text: &str) {
    print!("{text}");
    if let Err(e) = io::stdout().flush() {
        println!("{}{e}", "Unable to flush stdout: ".red());
    }
}

fn ask_int_input(question: &str, min: Option<i32>, max: Option<i32>) -> i32 {
    // There's no i32::try_from<float> in the standard library,
    // but the conversion shouldn't fail anyway, and we want the truncating
    // behaviour here.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::as_conversions)]
    {
        ask_float_input(
            question,
            convert_i32_option_to_f64_option(min),
            convert_i32_option_to_f64_option(max),
        )
        .trunc() as i32
    }
}

fn convert_i32_option_to_f64_option(option: Option<i32>) -> Option<f64> {
    if let Some(value) = option {
        match f64::try_from(value) {
            Ok(float) => {
                return Some(float);
            },

            Err(e) => {
                println!("{}{e}", "Error converting i32 to f64: ".red());
            },
        }
    }

    None
}

fn ask_float_input(question: &str, min: Option<f64>, max: Option<f64>) -> f64 {
    let min_with_default = min.unwrap_or(f64::MIN);
    let max_with_default = max.unwrap_or(f64::MAX);

    loop {
        print(question);

        let next_line = io::stdin().lock().lines().next();

        if let Some(result) = next_line {
            match result {
                Ok(line) =>
                    if let Ok(float_input) = line.parse::<f64>() {
                        if float_input >= min_with_default &&
                            float_input <= max_with_default
                        {
                            return float_input;
                        }

                        println!("{}{}{}{}", "Invalid selection. Please enter a selection between ".bright_red(), min_with_default.to_string().bright_red(), " and ".bright_red(), max_with_default.to_string().bright_red());
                    } else {
                        println!("{}", "Invalid value given. Please enter a valid whole number!".bright_red());
                    },

                Err(e) => {
                    println!(
                        "{}{e}",
                        "Error when getting line input: ".bright_red()
                    );
                },
            }
        } else {
            println!("{}", "error: no more lines".bright_red());
        }

        println!();
    }
}
