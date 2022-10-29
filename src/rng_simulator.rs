use crate::utils::ask_float_input;
use crate::utils::ask_int_input;
use crate::utils::cap;
use crate::utils::compare_f64;
use crate::utils::conditional_value_or_default;
use crate::utils::f64_to_i32;
use crate::utils::get_odds;
use crate::utils::has_unique_elements;
use crate::utils::mean;
use crate::utils::median;
use crate::utils::mode;
use crate::utils::percent_of;
use crate::utils::percentage_change;
use crate::utils::range;
use crate::utils::usize_to_f64;

use crate::constants::CHIMERA_DROP_CHANCE;
use crate::constants::JUDGEMENT_CORE_DROP_CHANCE;
use crate::constants::NECRONS_HANDLE_DROP_CHANCE;
use crate::constants::NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE;
use crate::constants::OVERFLUX_CAPACITOR_DROP_CHANCE;
use crate::constants::WARDEN_HEART_DROP_CHANCE;

use colored::Colorize;
use jandom::Random;
use std::time::Instant;

fn print_drops_selection() {
    println!();
    println!("Select which item you want to simulate RNG: ");

    println!(
        " {}. Chimera (%{CHIMERA_DROP_CHANCE}) [1/{}]",
        "1".bright_blue(),
        get_odds(CHIMERA_DROP_CHANCE)
    );
    println!(
        " {}. Judgement Core (%{JUDGEMENT_CORE_DROP_CHANCE}) [1/{}]",
        "2".bright_blue(),
        get_odds(JUDGEMENT_CORE_DROP_CHANCE)
    );
    println!(
        " {}. Warden Heart (%{WARDEN_HEART_DROP_CHANCE}) [1/{}]",
        "3".bright_blue(),
        get_odds(WARDEN_HEART_DROP_CHANCE)
    );
    println!(
        " {}. Overflux Capacitor (%{OVERFLUX_CAPACITOR_DROP_CHANCE}) [1/{}]",
        "4".bright_blue(),
        get_odds(OVERFLUX_CAPACITOR_DROP_CHANCE)
    );
    println!(
        " {}. Necron's Handle (%{NECRONS_HANDLE_DROP_CHANCE}) [1/{}]",
        "5".bright_blue(),
        get_odds(NECRONS_HANDLE_DROP_CHANCE)
    );
    println!(" {}. Necron's Handle (Master Mode) (%{NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE}) [1/{}]", "6".bright_blue(), get_odds(NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE));
    println!(" {}. Custom", "7".bright_blue());

    println!();
}

pub(crate) fn rng_simulator(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    print_drops_selection();

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

    let odds = get_odds(drop_rate_with_magic_find_and_looting);

    println!();
    println!(
        "Odds with Magic Find and Looting: {}/{}. Rolling {} times:",
        "1".bright_green(),
        odds.to_string().bright_red(),
        rolls.to_string().yellow()
    );
    println!();

    let all_succeeded_magic_find_values: &mut Vec<i32> =
        &mut Vec::with_capacity(TryInto::try_into(rolls).unwrap_or(0x7FFF_FFFF));
    let meter_succeeded_rolls: &mut Vec<i32> =
        &mut Vec::with_capacity(TryInto::try_into(rolls).unwrap_or(0x7FFF_FFFF));

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

    let percent = 100.0 -
        f64::abs(percentage_change(usize_to_f64(max_drops), f64::from(drops)));

    if rolls > 0 {
        println!();
    }

    println!("Out of {rolls} rolls, {drops} rolls succeeded.");

    if !percent.is_nan() {
        println!("You got %{} of the possible drops ({drops}/{max_drops}) with maximum magic find, with your magic find.", percent.to_string().yellow());
    }

    if !all_succeeded_magic_find_values.is_empty() {
        print_statistics(
            get_odds(original_drop_chance),
            all_succeeded_magic_find_values,
            meter_succeeded_rolls,
            rng_meter_percent,
        );
    }

    true
}

fn do_rolls_and_get_drops(
    original_drop_chance: f64, original_rng_meter_percent: f64,
    looting_extra_chance: i32, rolls: i32, magic_find: i32,
    all_succeeded_magic_find_values: &mut Vec<i32>,
    meter_succeeded_rolls: &mut Vec<i32>,
) -> i32 {
    let mut drops = 0;
    let mut rand = Random::default();

    let mut reset_meter_at_least_once = false;
    let mut last_reset_at = 0;

    let odds = get_odds(original_drop_chance);
    let original_rng_meter_progress =
        percent_of(odds, original_rng_meter_percent);

    for roll in 1..=rolls {
        let progress = f64_to_i32(
            if reset_meter_at_least_once {
                f64::from(roll - last_reset_at)
            } else {
                original_rng_meter_progress + f64::from(roll)
            }
            .trunc(),
        );

        let rng_meter_percent = 100.0 -
            f64::abs(percentage_change(odds, cap(f64::from(progress), odds)));

        let final_drop_chance = if rng_meter_percent >= 100.0 {
            100.0
        } else {
            let multiplier = if compare_f64(original_rng_meter_percent, -1.0) {
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

        let magic_number = rand.next_f64(); // future perf ref: this call is basically free, main bottleneck is io on
                                            // the println! and other code
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
            // bit of io bottleneck
            println!(
                "Roll #{}: {}, can't succeed even with max Magic Find.",
                roll.to_string().yellow(),
                "FAIL".bright_red()
            );
        } else {
            all_succeeded_magic_find_values
                .push(minimum_magic_find_needed_to_success);

            if success {
                // bit of io bottleneck
                println!(
                    "Roll #{}: {}, minimum magic find to succeed is {}. RNG Meter: %{}",
                    roll.to_string().yellow(),
                    "PASS".bright_green(),
                    minimum_magic_find_needed_to_success.to_string().green(),
                    rng_meter_percent
                );
            } else {
                // bit of io bottleneck
                println!("Roll #{}: {}, minimum magic find to succeed is {} which is higher than yours.", roll.to_string().yellow(), "FAIL".bright_red(), minimum_magic_find_needed_to_success.to_string().bright_red());
            }
        }
    }

    drops
}

fn print_statistics(
    odds: f64, all_succeeded_magic_find_values: &mut Vec<i32>,
    meter_succeeded_rolls: &mut Vec<i32>, original_rng_meter: f64,
) {
    println!();

    let mean_succeed_magic_find = mean(all_succeeded_magic_find_values);

    println!("Mean (Average) Succeed Magic Find: {mean_succeed_magic_find}");
    if let Some(median_succeed_magic_find) =
        median(all_succeeded_magic_find_values)
    {
        println!(
            "Median (Middle) Succeed Magic Find: {median_succeed_magic_find}"
        );
    }

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

    if !meter_succeeded_rolls.is_empty() {
        println!();
    }

    if compare_f64(original_rng_meter, -1.0) {
        println!("{}: The RNG Meter doesn't work on this drop type, so values below are based on if the RNG meter existed as a percentage to expected amount of rolls to get the drop, but didn't actually guarantee drops or modify chances.", "Note".red());
        println!();
    }

    let mean_succeed_rolls = mean(meter_succeeded_rolls);

    let mean_succeed_meter =
        100.0 - f64::abs(percentage_change(odds, cap(mean_succeed_rolls, odds)));

    if !mean_succeed_rolls.is_nan() && !mean_succeed_meter.is_nan() {
        println!("Mean (Average) Amount of Rolls until Succeed: {mean_succeed_rolls} (%{mean_succeed_meter} RNG Meter)");
    }

    if let Some(median_succeed_rolls) = median(meter_succeeded_rolls) {
        let median_succeed_meter = 100.0 -
            f64::abs(percentage_change(odds, cap(median_succeed_rolls, odds)));

        println!("Median (Middle) Amount of Rolls until Succeed: {median_succeed_rolls} (%{median_succeed_meter} RNG Meter)");
    }

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
