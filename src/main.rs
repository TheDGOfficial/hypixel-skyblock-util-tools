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

use futures::future::join_all;
use std::collections::HashMap;

use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process::ExitCode;

use colored::Colorize;
use core::time::Duration;
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

    println!();

    let selection = ask_int_input("Enter a number to select: ", Some(1), Some(2));

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

    let mut prices = HashMap::new();

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
    prices: &mut HashMap<usize, i64>,
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

    for (i, result) in (join_all(requests).await).into_iter().enumerate() {
        match result {
            Ok(response) => match response.text().await {
                Ok(response_body) => {
                    match serde_json::from_str::<Value>(&response_body) {
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
                            println!("{}{e}", "Error when parsing JSON: ".red());

                            return false;
                        },
                    }
                },

                Err(e) => {
                    println!("{}{e}", "Error when getting response body: ".red());

                    return false;
                },
            },

            Err(e) => {
                println!("{}{e}", "Error when getting response: ".red());

                return false;
            },
        }
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
    let min_with_default = min.unwrap_or(i32::MIN);
    let max_with_default = max.unwrap_or(i32::MAX);

    loop {
        print(question);

        let next_line = io::stdin().lock().lines().next();

        if let Some(result) = next_line {
            match result {
                Ok(line) =>
                    if let Ok(int_input) = line.parse::<i32>() {
                        if int_input >= min_with_default &&
                            int_input <= max_with_default
                        {
                            return int_input;
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
