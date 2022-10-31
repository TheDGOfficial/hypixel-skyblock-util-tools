use core::hash::BuildHasherDefault;
use std::io;
use std::io::BufRead;
use std::io::Write;

use colored::Colorize;
use nohash_hasher::IntMap;
use nohash_hasher::IntSet;
use num::FromPrimitive;

use crate::utils::FunctionResult::Failure;
use crate::utils::FunctionResult::Success;

pub(crate) fn get_odds(percentage_chance: f64) -> f64 {
    100.0 / percentage_chance
}

pub(crate) fn compare_f64(f64: f64, compare_to: f64) -> bool {
    (f64 - compare_to).abs() < f64::EPSILON
}

pub(crate) fn has_unique_elements(vec: &[i32]) -> bool {
    let mut unique = IntSet::with_capacity_and_hasher(
        vec.len(),
        BuildHasherDefault::default(),
    );
    vec.iter().all(move |x| unique.insert(x.to_owned()))
}

pub(crate) fn cap(number: f64, cap: f64) -> f64 {
    if number > cap {
        return cap;
    }

    number
}

pub(crate) fn f64_to_i32(f64: f64) -> i32 {
    i32::from_f64(f64).map_or_else(|| {
        println!("{}{f64}", "warning: loss of precision while converting from f64 to i32, if this is intentional, call .trunc() on the value before calling this function. f64 value: ".yellow());

        // i32::from is not implemented for f64 so using as is the only option.
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::as_conversions)]
        {
            f64 as i32
        }
    }, |i32| i32)
}

pub(crate) fn usize_to_f64(usize: usize) -> f64 {
    f64::from_usize(usize).map_or_else(|| {
        println!("{}{usize}", "warning: loss of precision due to overflow of usize while converting to f64: ".yellow());

        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::as_conversions)]
        {
            usize as f64
        }
    }, |f64| f64)
}

pub(crate) fn i64_to_f64(i64: i64) -> f64 {
    f64::from_i64(i64).map_or_else(
        || {
            println!(
                "{}{i64}",
                "warning: loss of precision due while converting i64 to f64: "
                    .yellow()
            );

            #[allow(clippy::cast_precision_loss)]
            #[allow(clippy::as_conversions)]
            {
                i64 as f64
            }
        },
        |f64| f64,
    )
}

// Result<T, E> like enum but without the result and error.
// This useful if a function can fail without an error.
pub(crate) enum FunctionResult {
    Success,
    Failure,
}

// Returns first element on the array and Success FunctionResult if there's only
// one element in the array.

// If theres more than one or no elements, returns None and Failure.
// The failure here is like function returning, i.e false. It's not like an
// error.

// If the array size is not empty, but first value is None, returns None and
// Success.
pub(crate) fn return_first_elem_if_only_one_elem(
    array: &Vec<i32>,
) -> (Option<f64>, FunctionResult) {
    if array.len() == 1 {
        if let Some(first) = array.first() {
            return (Some(f64::from(*first)), Success);
        }

        return (None, Success);
    }

    (None, Failure)
}

pub(crate) fn mean(array: &Vec<i32>) -> Option<f64> {
    if array.is_empty() {
        return None;
    }

    let first_elem = return_first_elem_if_only_one_elem(array);

    if let Success = first_elem.1 {
        return first_elem.0;
    }

    // We must calculate sum manually because theres no checked_sum shortcut
    // method in standard library.
    let mut sum: i64 = 0;

    for value in array {
        if let Some(result) = sum.checked_add(i64::from(*value)) {
            sum = result;
        } else {
            // Overflow occurred
            return None;
        }
    }

    Some(i64_to_f64(sum) / usize_to_f64(array.len()))
}

// Returns the middle value in an array.
// This method sorts the array, and such, the array order will not be same after
// this method is called. Returns None if the array is empty, and if theres only
// one value in the array, returns that value.
pub(crate) fn median(array: &mut Vec<i32>) -> Option<f64> {
    if array.is_empty() {
        return None;
    }

    let first_elem = return_first_elem_if_only_one_elem(array);

    if let Success = first_elem.1 {
        return first_elem.0;
    }

    array.sort_unstable();

    if array.len() % 2 == 0 {
        if let Some(left) = array.get(array.len() / 2 - 1) {
            if let Some(right) = array.get(array.len() / 2) {
                return Some(f64::from(left + right) / 2.0);
            }
        }

        return None;
    }

    if let Some(value) = array.get(array.len() / 2) {
        return Some(f64::from(*value));
    }

    None
}

// Returns the most occurring value in an array.
// Returns None if the array is empty.
pub(crate) fn mode(array: &Vec<i32>) -> Option<i32> {
    let mut occurrences = IntMap::with_capacity_and_hasher(
        array.len(),
        BuildHasherDefault::default(),
    );

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
pub(crate) fn range(array: &[i32]) -> Option<i32> {
    if let Some(min) = array.iter().min() {
        if let Some(max) = array.iter().max() {
            return Some(max - min);
        }
    }

    None
}

pub(crate) fn conditional_value_or_default<T>(
    condition: bool, value: fn() -> T, default: T,
) -> T {
    if condition {
        return value();
    }

    default
}

pub(crate) fn percent_of(number: f64, percent: f64) -> f64 {
    (number / 100.0) * percent
}

pub(crate) fn percentage_change(starting_number: f64, ending_number: f64) -> f64 {
    ((ending_number - starting_number) / f64::abs(starting_number)) * 100.0
}

pub(crate) fn with_comma_separators(s: &str) -> Option<String> {
    let dot = s.bytes().position(|c| c == b'.').unwrap_or(s.len());
    let negative = s.bytes().next() == Some(b'-');

    let mut integer_digits_remaining = dot - usize::from(negative);
    let mut out = String::with_capacity(s.len() + integer_digits_remaining / 3);

    for (i, c) in s.bytes().enumerate() {
        match c {
            b'-' =>
                if i != 0 {
                    return None;
                },

            b'.' =>
                if i != dot {
                    return None;
                },

            b'0'..=b'9' =>
                if integer_digits_remaining > 0 {
                    if i != usize::from(negative) &&
                        integer_digits_remaining % 3 == 0
                    {
                        out.push(',');
                    }

                    integer_digits_remaining -= 1;
                },

            _ => {
                return None;
            },
        }

        out.push(char::from(c));
    }

    Some(out)
}

pub(crate) fn print(text: &str) {
    print!("{text}");
    if let Err(e) = io::stdout().flush() {
        println!("{}{e}", "Unable to flush stdout: ".red());
    }
}

pub(crate) fn ask_int_input(
    question: &str, min: Option<i32>, max: Option<i32>,
) -> i32 {
    f64_to_i32(
        ask_float_input(
            question,
            convert_i32_option_to_f64_option(min),
            convert_i32_option_to_f64_option(max),
        )
        .trunc(),
    )
}

pub(crate) fn convert_i32_option_to_f64_option(
    option: Option<i32>,
) -> Option<f64> {
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

pub(crate) fn ask_float_input(
    question: &str, min: Option<f64>, max: Option<f64>,
) -> f64 {
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
