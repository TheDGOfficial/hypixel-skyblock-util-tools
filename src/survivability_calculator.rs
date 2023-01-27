use std::time::Instant;

use colored::Colorize;

use crate::constants::F3_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::F4_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::F5_FELS_DAMAGE;
use crate::constants::F5_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::F6_FELS_DAMAGE;
use crate::constants::F6_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::F7_FELS_DAMAGE;
use crate::constants::F7_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::M3_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::M4_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::M5_FELS_DAMAGE;
use crate::constants::M5_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::M6_FELS_DAMAGE;
use crate::constants::M6_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::M7_FELS_DAMAGE;
use crate::constants::M7_SHADOW_ASSASSIN_DAMAGE;
use crate::constants::VOIDGLOOM_SERAPH_TIER_1_TOTAL_DAMAGE;
use crate::constants::VOIDGLOOM_SERAPH_TIER_2_TOTAL_DAMAGE;
use crate::constants::VOIDGLOOM_SERAPH_TIER_3_TOTAL_DAMAGE;
use crate::constants::VOIDGLOOM_SERAPH_TIER_4_TOTAL_DAMAGE;
use crate::utils::ask_int_input;
use crate::utils::compare_f64;
use crate::utils::f64_to_i32;
use crate::utils::with_comma_separators;

#[inline]
pub(crate) fn survivability_calculator(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    println!();
    println!("Select your enemy: ");
    println!(" {}. Shadow Assassin", "1".bright_blue(),);
    println!(" {}. Fels", "2".bright_blue(),);
    println!(" {}. Voidgloom Seraph", "3".bright_blue(),);
    println!(" {}. Custom", "4".bright_blue());

    let selection =
        ask_int_input("Enter a number to select: ", Some(1), Some(4));
    let enemy_damage_per_hit = get_enemy_damage_per_hit(selection);

    let health = ask_int_input("What's your Health?: ", Some(100), None);
    let defense = ask_int_input("What's your Defense?: ", Some(0), None);

    let effective_health = calculate_effective_health(health, defense);
    let hits_to_die =
        calculate_hits_to_die(effective_health, enemy_damage_per_hit);

    let crit_damage = ask_int_input("What's your Crit Damage? (If you don't plan to use Wither Shield, enter 0): ", Some(0), None);

    *start_without_user_input = Some(Instant::now());

    let wither_shield_health =
        f64_to_i32((f64::from(crit_damage) * 1.5).trunc());
    let total_health_with_wither_shield = health + wither_shield_health;

    let effective_health_with_wither_shield =
        calculate_effective_health(total_health_with_wither_shield, defense);
    let hits_to_die_with_wither_shield = calculate_hits_to_die(
        effective_health_with_wither_shield,
        enemy_damage_per_hit,
    );

    println!();
    println!("{} Even though Wither Shield is factored in, all other factors aren't, even the Wither Shield's secret %10 damage reduction ability. These not factored in conditions include: any sort of healing or damage reduction other than Defense, and special variants/attacks. Also, hits to die means that, you will be dead after taking that amount of hits, not that you will survive that amount of hits.", "Note:".red());
    println!();
    println!("Your enemy can and will do up to {} damage in one hit. You have {} Effective Health, taking {hits_to_die} (rounded down to {}) hits to die. With Wither Shield and full health, these values become {} Effective Health and taking {hits_to_die_with_wither_shield} (rounded down to {}) hits to die.",
             with_comma_separators(&enemy_damage_per_hit.to_string()).unwrap_or_else(|| enemy_damage_per_hit.to_string()),
             with_comma_separators(&effective_health.to_string()).unwrap_or_else(|| effective_health.to_string()),
             hits_to_die.trunc(),
             with_comma_separators(&effective_health_with_wither_shield.to_string()).unwrap_or_else(|| effective_health_with_wither_shield.to_string()),
             hits_to_die_with_wither_shield.trunc(),
    );

    let next_effective_health_milestone =
        calculate_next_effective_health_milestone(
            effective_health,
            enemy_damage_per_hit,
        );

    let next_effective_health_milestone_with_wither_shield =
        calculate_next_effective_health_milestone(
            effective_health_with_wither_shield,
            enemy_damage_per_hit,
        );

    let difference = next_effective_health_milestone - effective_health;
    let difference_with_wither_shield =
        next_effective_health_milestone_with_wither_shield
            - effective_health_with_wither_shield;

    println!();
    println!("To afford to take another hit, you need {next_effective_health_milestone} ({difference} more than your current) Effective Health, or {next_effective_health_milestone_with_wither_shield} ({difference_with_wither_shield} more than your current) Effective Health with Wither Shield.");

    let needed_health =
        find_needed_health(health, defense, next_effective_health_milestone);
    let needed_defense =
        find_needed_defense(health, defense, next_effective_health_milestone);

    println!();
    println!("To afford to take another hit, you need {needed_health} more Health or {needed_defense} more Defense.");

    let needed_health_with_wither_shield = find_needed_health(
        total_health_with_wither_shield,
        defense,
        next_effective_health_milestone_with_wither_shield,
    );

    let needed_crit_damage = f64_to_i32(
        (f64::from(find_needed_health(
            total_health_with_wither_shield,
            defense,
            next_effective_health_milestone_with_wither_shield,
        )) / 1.5)
            .ceil(),
    );

    let needed_defense_with_wither_shield = find_needed_defense(
        total_health_with_wither_shield,
        defense,
        next_effective_health_milestone_with_wither_shield,
    );

    println!("To afford to take another hit with Wither Shield, you need {needed_health_with_wither_shield} more Health, {needed_crit_damage} more Crit Damage or {needed_defense_with_wither_shield} more Defense.");

    true
}

#[inline]
#[must_use]
fn find_needed_health(hp: i32, defense: i32, to_effective_health: i32) -> i32 {
    for new_hp in hp..=i32::MAX {
        let ehp = calculate_effective_health(new_hp, defense);

        if ehp >= to_effective_health {
            return new_hp - hp;
        }
    }

    -1
}

#[inline]
#[must_use]
fn find_needed_defense(
    hp: i32,
    defense: i32,
    to_effective_health: i32,
) -> i32 {
    for new_defense in defense..=i32::MAX {
        let ehp = calculate_effective_health(hp, new_defense);

        if ehp >= to_effective_health {
            return new_defense - defense;
        }
    }

    -1
}

#[inline]
#[must_use]
fn calculate_next_effective_health_milestone(
    effective_health: i32,
    enemy_damage_per_hit: i32,
) -> i32 {
    let original_hits_to_die =
        calculate_hits_to_die(effective_health, enemy_damage_per_hit);
    let mut hits_to_die;

    for ehp in effective_health..=i32::MAX {
        hits_to_die = calculate_hits_to_die(ehp, enemy_damage_per_hit);

        if !compare_f64(original_hits_to_die, original_hits_to_die.ceil())
            && compare_f64(hits_to_die, original_hits_to_die.ceil())
        {
            return ehp;
        }

        if compare_f64(hits_to_die, original_hits_to_die + 1.0) {
            return ehp;
        }
    }

    0
}

#[inline]
#[must_use]
fn calculate_hits_to_die(
    effective_health: i32,
    enemy_damage_per_hit: i32,
) -> f64 {
    1.0 + f64::max(
        1.0,
        f64::from(effective_health) / f64::from(enemy_damage_per_hit),
    )
}

#[inline]
#[must_use]
const fn calculate_effective_health(health: i32, defense: i32) -> i32 {
    health * (1 + defense / 100)
}

#[inline]
#[must_use]
fn get_enemy_damage_per_hit(selection: i32) -> i32 {
    match selection {
        1 | 2 => {
            let start = if selection == 1 { 3 } else { 5 };

            let mut total_index = 0;

            for dungeon_type in 1..=2 {
                let dungeon_type_as_str = if dungeon_type == 1 {
                    "F"
                } else {
                    println!();

                    "M"
                };

                for floor in start..=7 {
                    total_index += 1;

                    let as_str = floor.to_string();

                    println!(
                        " {}. {}{}",
                        total_index.to_string().bright_blue(),
                        dungeon_type_as_str,
                        as_str
                    );
                }
            }

            let floor_selection =
                ask_int_input("Select floor: ", Some(1), Some(total_index));

            match (selection, floor_selection) {
                (1, 1) => F3_SHADOW_ASSASSIN_DAMAGE,
                (1, 2) => F4_SHADOW_ASSASSIN_DAMAGE,
                (1, 3) => F5_SHADOW_ASSASSIN_DAMAGE,
                (1, 4) => F6_SHADOW_ASSASSIN_DAMAGE,
                (1, 5) => F7_SHADOW_ASSASSIN_DAMAGE,

                (1, 6) => M3_SHADOW_ASSASSIN_DAMAGE,
                (1, 7) => M4_SHADOW_ASSASSIN_DAMAGE,
                (1, 8) => M5_SHADOW_ASSASSIN_DAMAGE,
                (1, 9) => M6_SHADOW_ASSASSIN_DAMAGE,
                (1, 10) => M7_SHADOW_ASSASSIN_DAMAGE,

                (2, 1) => F5_FELS_DAMAGE,
                (2, 2) => F6_FELS_DAMAGE,
                (2, 3) => F7_FELS_DAMAGE,

                (2, 4) => M5_FELS_DAMAGE,
                (2, 5) => M6_FELS_DAMAGE,
                (2, 6) => M7_FELS_DAMAGE,

                (..) => {
                    eprintln!(
                        "{}({selection}, {floor_selection})",
                        "error: invalid selections: ".red()
                    );

                    0
                },
            }
        },

        3 => {
            println!();
            println!("Select tier: ");
            println!(" {}. Tier 1", "1".bright_blue(),);
            println!(" {}. Tier 2", "2".bright_blue(),);
            println!(" {}. Tier 3", "3".bright_blue(),);
            println!(" {}. Tier 4", "4".bright_blue(),);

            let tier_selection =
                ask_int_input("Enter a number to select: ", Some(1), Some(4));

            match tier_selection {
                1 => VOIDGLOOM_SERAPH_TIER_1_TOTAL_DAMAGE,
                2 => VOIDGLOOM_SERAPH_TIER_2_TOTAL_DAMAGE,
                3 => {
                    println!();
                    println!("{} This assumes there are no heads, and the hit shield DPS increase is also not factored in.", "Note:".red());

                    VOIDGLOOM_SERAPH_TIER_3_TOTAL_DAMAGE
                },
                4 => {
                    println!();
                    println!("{} This assumes there are no heads, you didn't get hit by any lasers, and the hit shield DPS increase is also not factored in.", "Note:".red());

                    VOIDGLOOM_SERAPH_TIER_4_TOTAL_DAMAGE
                },

                _ => {
                    eprintln!(
                        "{}{tier_selection}",
                        "error: invalid selection: ".red()
                    );

                    0
                },
            }
        },

        4 => ask_int_input("Enter your enemy's damage: ", Some(0), None),

        _ => {
            eprintln!("{}{selection}", "error: invalid selection: ".red());

            0
        },
    }
}
