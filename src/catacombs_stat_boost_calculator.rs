use core::cmp;

use std::time::Instant;

use crate::constants::SECRETS_NEEDED_FOR_MAX_GENERALS_MEDALLION;

use colored::Colorize;

use crate::utils::ask_int_input;
use crate::utils::percentage_change;
use crate::utils::u32_to_i32;

#[inline]
pub(crate) fn catacombs_stat_boost_calculator(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    let catacombs_boost = get_cata_stat_boost(ask_int_input(
        "Enter your current Catacombs level: ",
        Some(0),
        Some(i32::MAX),
    ));

    let normal_stars_boost = 10
        * ask_int_input(
            "Enter the amount of normal stars your gear has: ",
            Some(0),
            Some(5),
        );

    let master_stars_boost = 5 * ask_int_input(
        "Enter the amount of master stars your gear has: ",
        Some(0),
        Some(5),
    );

    let generals_medallion_stat_boost = u32_to_i32(cmp::min(SECRETS_NEEDED_FOR_MAX_GENERALS_MEDALLION, ask_int_input("Enter the amount of secrets you have (Enter 0 if you don't have General's Medallion): ", Some(0), Some(i32::MAX))).checked_ilog10().unwrap_or(0)) + 1;

    let planned_catacombs_level_boost = get_cata_stat_boost(ask_int_input(
        "Enter your planned Catacombs Level: ",
        Some(0),
        Some(i32::MAX),
    ));

    let planned_normal_stars_boost = 10
        * ask_int_input(
            "Enter the amount of normal stars you plan your gear to have: ",
            Some(0),
            Some(5),
        );

    let planned_master_stars_boost = 5 * ask_int_input(
        "Enter the amount of master stars you plan your gear to have: ",
        Some(0),
        Some(5),
    );

    let planned_generals_medallion_stat_boost = u32_to_i32(cmp::min(SECRETS_NEEDED_FOR_MAX_GENERALS_MEDALLION, ask_int_input("Enter the amount of secrets you plan to have (Enter 0 if you don't plan to have General's Medallion): ", Some(0), Some(i32::MAX))).checked_ilog10().unwrap_or(0)) + 1;

    *start_without_user_input = Some(Instant::now());

    let total_stat_boost =
        catacombs_boost + normal_stars_boost + master_stars_boost + generals_medallion_stat_boost;

    let planned_total_stat_boost = planned_catacombs_level_boost
        + planned_normal_stars_boost
        + planned_master_stars_boost
        + planned_generals_medallion_stat_boost;

    println!();
    println!("{}{}{}", "Difference between your current and planned Catacombs level, Stars/Master Stars and General's Medallion boost in percent is %".bright_green(), percentage_change(From::from(total_stat_boost), From::from(planned_total_stat_boost)).to_string().bright_yellow(), ".".white());

    true
}

#[inline]
#[must_use]
fn get_cata_stat_boost(catacombs_level: i32) -> i32 {
    let mut cata_stat_boost = 0;

    for level in 0..=catacombs_level {
        cata_stat_boost += match level {
            0 => 10,

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
            51..=i32::MAX => 0,

            _ => {
                eprintln!(
                    "{}{level}",
                    "error: invalid catacombs level: ".red()
                );

                0
            },
        };
    }

    cata_stat_boost
}
