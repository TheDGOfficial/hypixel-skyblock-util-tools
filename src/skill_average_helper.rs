use std::time::Instant;

use crate::utils;

#[inline]
pub(crate) fn skill_average_helper(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    let mut skills: Vec<Skill> = Vec::new();

    for skill in Skill::ALL_SKILLS {
        let current_level = utils::ask_int_input(
            format!("What is your {} level?: ", skill.name).as_str(),
            Some(0),
            Some(skill.max_level),
        );

        skills.push(Skill { level: current_level, ..skill });
    }

    let max_skill_average = utils::mean(
        &Skill::ALL_SKILLS
            .clone()
            .iter()
            .map(|skill| skill.max_level)
            .collect(),
    )
    .unwrap_or(0.0);
    let target_skill_average = utils::ask_float_input(
        "What is your target Skill Average?: ",
        Some(0.0),
        Some(max_skill_average),
    );

    *start_without_user_input = Some(Instant::now());

    let mut skill_average =
        utils::mean(&skills.iter().map(|skill| skill.level).collect())
            .unwrap_or(0.0);

    println!(
        "Your Skill Average is {skill_average}. Max Skill Average is {max_skill_average}."
    );
    println!();

    let skill_levels_required_for_target_average =
        get_skill_levels_required_for_target_average(
            &skills,
            target_skill_average,
        );

    println!(
        "To get your target Skill Average, you need to level one of the Skills below to specified levels (or level multiple to get in total {skill_levels_required_for_target_average} Skill Levels):"
    );
    println!();

    let mut target_skills: Vec<Skill> = skills.clone();

    for skill in &skills {
        if skill.level < skill.max_level {
            let old_skill_average = skill_average;

            while skill_average < target_skill_average {
                if let Some(index) = target_skills
                    .iter()
                    .position(|&target_skill| target_skill.name == skill.name)
                {
                    target_skills[index].level += 1;

                    skill_average = utils::mean(
                        &target_skills
                            .iter()
                            .map(|target_skill| target_skill.level)
                            .collect(),
                    )
                    .unwrap_or(0.0);
                }
            }

            if let Some(index) = target_skills
                .iter()
                .position(|&target_skill| target_skill.name == skill.name)
            {
                if target_skills[index].level > target_skills[index].max_level
                {
                    println!(
                        "{} {} AND {} levels from other skills (from ones below or above)",
                        skill.name,
                        target_skills[index].max_level,
                        target_skills[index].level
                            - target_skills[index].max_level
                    );
                } else {
                    println!("{} {}", skill.name, target_skills[index].level);
                }
            }

            skill_average = old_skill_average;
            target_skills = skills.clone();
        }
    }

    let mut nonmaxed_skills = Vec::new();

    for skill in &skills {
        if skill.level < skill.max_level {
            nonmaxed_skills.push(skill);
        }
    }

    println!(
        "If Required Skill Levels needed fairly split between non maxed skills (Recommended):"
    );
    println!();

    for skill in &nonmaxed_skills {
        println!(
            "{} {}",
            skill.name,
            skill.level
                + skill_levels_required_for_target_average
                    / utils::f64_to_i32(utils::usize_to_f64(
                        nonmaxed_skills.len()
                    ))
        );
    }

    true
}

#[inline]
#[must_use]
fn get_skill_levels_required_for_target_average(
    skills: &[Skill],
    target_skill_average: f64,
) -> i32 {
    let mut target_skills = skills.to_owned();
    let mut skill_average =
        utils::mean(&target_skills.iter().map(|skill| skill.level).collect())
            .unwrap_or(0.0);

    let mut skill_levels_needed = 0;

    while skill_average < target_skill_average {
        target_skills[0].level += 1;
        skill_average = utils::mean(
            &target_skills.iter().map(|skill| skill.level).collect(),
        )
        .unwrap_or(0.0);

        skill_levels_needed += 1;
    }

    skill_levels_needed
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
struct Skill {
    name: &'static str,

    level: i32,
    max_level: i32,
}

impl Skill {
    const ALCHEMY: Self = Self::new("Alchemy", 0, 50);
    const ALL_SKILLS: [Self; 9] = [
        Self::FARMING,
        Self::MINING,
        Self::COMBAT,
        Self::FORAGING,
        Self::FISHING,
        Self::ENCHANTING,
        Self::ALCHEMY,
        Self::CARPENTRY,
        Self::TAMING,
    ];
    const CARPENTRY: Self = Self::new("Carpentry", 0, 50);
    const COMBAT: Self = Self::new("Combat", 0, 60);
    const ENCHANTING: Self = Self::new("Enchanting", 0, 60);
    const FARMING: Self = Self::new("Farming", 0, 60);
    const FISHING: Self = Self::new("Fishing", 0, 50);
    const FORAGING: Self = Self::new("Foraging", 0, 50);
    const MINING: Self = Self::new("Mining", 0, 60);
    const TAMING: Self = Self::new("Taming", 0, 60);

    #[inline]
    #[must_use]
    const fn new(name: &'static str, level: i32, max_level: i32) -> Self {
        Self { name, level, max_level }
    }
}
