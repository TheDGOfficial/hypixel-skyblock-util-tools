#![allow(clippy::unwrap_used)]
#![allow(clippy::unreachable)]
#![allow(clippy::panic)]

use crate::master_skull_upgrade_helper::get_total_required_amount;
use std::env;
use std::path::Path;
use std::path::PathBuf;

use crate::rng_simulator;
use crate::rng_simulator::drop_rate_with_magic_find_and_looting;
use crate::rng_simulator::get_minimum_magic_find_needed_to_succeed;
use crate::rng_simulator::passes;
use crate::utils::cap;
use crate::utils::compare_f64;
use crate::utils::conditional_value_or_default;
use crate::utils::convert_i32_option_to_f64_option;
use crate::utils::f64_to_i32;
use crate::utils::get_odds;
use crate::utils::has_unique_elements;
use crate::utils::i64_to_f64;
use crate::utils::lines_from_file_from_end;
use crate::utils::mean;
use crate::utils::median;
use crate::utils::mode;
use crate::utils::percent_of;
use crate::utils::percentage_change;
use crate::utils::range;
use crate::utils::return_first_elem_if_only_one_elem;
use crate::utils::usize_to_f64;
use crate::utils::with_comma_separators;
use crate::utils::FunctionResult;

fn get_workspace_path() -> PathBuf {
    Path::new(".idea").join("workspace.xml")
}

fn should_test_intellij_clippy_args() -> bool {
    return get_workspace_path().exists() && env::var("CLIPPY_ARGS").is_ok();
}

#[test]
fn test_intellij_clippy_args() {
    if should_test_intellij_clippy_args() {
        if let Ok(clippy_args) = env::var("CLIPPY_ARGS") {
            test_intellij_clippy_args0(&clippy_args);
        }
    }
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_intellij_clippy_args_should_fail() {
    if should_test_intellij_clippy_args() {
        test_intellij_clippy_args0("whatever");
    } else {
        panic!("assertion `left == right` failed"); // workaround to not
                                                       // fail the test
    }
}

fn test_intellij_clippy_args0(args: &str) {
    let workspace_file_path = &get_workspace_path();

    let workspace_file_contents =
        lines_from_file_from_end(workspace_file_path, usize::MAX, false);

    assert!(
        !workspace_file_contents.is_empty(),
        "workspace file path is empty or can't get its contents"
    );

    for line in workspace_file_contents {
        if line
            .trim()
            .starts_with(r#"<option name="externalLinterArguments" value=""#)
        {
            assert_eq!(
                line.trim().replace('\t', "").replace(" />", "/>"),
                format!(
                    r#"<option name="externalLinterArguments" value="{args}"/>"#
                )
            );
        }
    }
}

#[test]
fn test_compare_f64() {
    assert!(compare_f64(1.0, 1.0));
    assert!(compare_f64(-1.0, -1.0));

    assert!(!compare_f64(1.1, 1.0));
    assert!(!compare_f64(-1.1, -1.0));
}

#[test]
fn test_mean() {
    let mean = mean(&vec![1, 2, 3, 4, 5]).unwrap();
    let expected_result = 3.0;

    assert!(
        compare_f64(mean, expected_result),
        "{} != {}",
        mean,
        expected_result
    );
}

#[test]
fn test_mean_overflow_i32() {
    let mean = mean(&vec![i32::MAX / 2; 3]).unwrap();
    let expected_result = f64::from(i32::MAX / 2);

    assert!(
        compare_f64(mean, expected_result),
        "{} != {}",
        mean,
        expected_result
    );
}

#[test]
fn test_convert_i32_option_to_f64_option() {
    let optional = Some(42);

    assert_eq!(
        optional.unwrap(),
        f64_to_i32(convert_i32_option_to_f64_option(optional).unwrap())
    );
}

#[test]
fn test_with_comma_separators() {
    assert_eq!(with_comma_separators("100000").unwrap(), "100,000");
}

#[test]
fn test_percentage_change() {
    let value = percentage_change(1.0, 2.0);
    let expected_result = 100.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_percent_of() {
    let value = percent_of(50.0, 25.0);
    let expected_result = 12.5;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_conditional_value_or_default() {
    assert_eq!(
        conditional_value_or_default(false, || { unreachable!() }, 100),
        100
    );

    assert_eq!(conditional_value_or_default(true, || { 100 }, 50), 100);
}

#[test]
fn test_range() {
    assert_eq!(range(&[1, 2, 3, 4, 5]).unwrap(), 4);
}

#[test]
fn test_mode() {
    assert_eq!(mode(&vec![1, 2, 3, 4, 5, 2, 2, 3]).unwrap(), 2);
}

#[test]
fn test_median() {
    let value = median(&mut vec![1, 2, 3, 4, 5]).unwrap();
    let expected_result = 3.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_median_2() {
    let value = median(&mut vec![1, 2, 3, 4, 5, 6]).unwrap();
    let expected_result = 3.5;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_i64_to_f64() {
    let value = i64_to_f64(10);
    let expected_result = 10.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_usize_to_f64() {
    let value = usize_to_f64(10);
    let expected_result = 10.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_f64_to_i32() {
    assert_eq!(f64_to_i32(10.0), 10);
}

#[test]
fn test_cap() {
    let value = cap(11.0, 10.0);
    let expected_result = 10.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_has_unique_elements() {
    assert!(has_unique_elements(&[1, 2, 3, 4, 5]));
    assert!(!has_unique_elements(&[1, 2, 3, 4, 5, 2]));
}

#[test]
fn test_get_odds() {
    let value = get_odds(0.5);
    let expected_result = 200.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_drop_rate_with_magic_find_and_looting() {
    let value = drop_rate_with_magic_find_and_looting(1.0, 50, 50.0);
    let expected_result = 2.25;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );
}

#[test]
fn test_passes() {
    let drop_chance = 6.0;
    let magic_number = rng_simulator::rand_f64(&mut rng_simulator::new_rng());

    let magic_find = 900;
    let looting_extra_chance = 75.0;

    let drop_rate = drop_rate_with_magic_find_and_looting(
        drop_chance,
        magic_find,
        looting_extra_chance,
    );

    let value = drop_rate;
    let expected_result = 105.0;

    assert!(
        compare_f64(value, expected_result),
        "{} != {}",
        value,
        expected_result
    );

    assert!(
        passes(magic_number, drop_chance, magic_find, looting_extra_chance),
        "{magic_number} > {}",
        drop_rate / 100.0
    );
}

#[test]
fn test_get_minimum_magic_find_needed_to_succeed() {
    assert_eq!(
        get_minimum_magic_find_needed_to_succeed(
            rng_simulator::rand_f64(&mut rng_simulator::new_rng()),
            100.0,
            0.0,
            None
        ),
        0
    );

    assert_eq!(
        // r/oddlyspecific

        // too lazy so got those values from running the method smh
        // we know it works now (so the values we got are correct), this test
        // is to ensure it keeps working in the future
        get_minimum_magic_find_needed_to_succeed(
            0.174_911_835_457_161_56,
            12.0,
            15.0,
            Some(26)
        ),
        27
    );
}

#[test]
fn test_get_total_required_amount() {
    assert_eq!(get_total_required_amount(1, 7), 4096);
}

#[test]
fn test_return_first_elem_if_only_elem() {
    test_return_first_elem_if_only_elem0(
        &vec![15],
        Some(15.0),
        &FunctionResult::Success,
    );
    test_return_first_elem_if_only_elem0(
        &vec![],
        None,
        &FunctionResult::Failure,
    );
    test_return_first_elem_if_only_elem0(
        &vec![1, 2],
        None,
        &FunctionResult::Failure,
    );
}

fn test_return_first_elem_if_only_elem0(
    vec: &Vec<i32>,
    expected_value: Option<f64>,
    expected_function_result: &FunctionResult,
) {
    let result = return_first_elem_if_only_one_elem(vec);

    if let Some(expected_result) = expected_value {
        let value = result.0.unwrap();

        assert!(
            compare_f64(value, expected_result),
            "{} != {}",
            value,
            expected_result
        );
    } else {
        assert!(result.0.is_none());
    }

    match result.1 {
        FunctionResult::Success => {
            if matches!(*expected_function_result, FunctionResult::Failure) {
                panic!("expected Success, got Failure");
            }
        },
        FunctionResult::Failure => {
            if matches!(*expected_function_result, FunctionResult::Success) {
                panic!("expected Failure, got Success");
            }
        },
    }
}
