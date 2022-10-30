use core::cmp::min;
use core::hash::BuildHasherDefault;
use core::time::Duration;
use std::time::Instant;

use colored::Colorize;
use futures::stream::FuturesOrdered;
use futures::StreamExt;
use nohash_hasher::IntMap;
use reqwest::Error;
use reqwest::Response;
use serde_json::Value;

use crate::utils::ask_int_input;
use crate::utils::with_comma_separators;

pub(crate) async fn upgrade_calculator_for_master_skulls(
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

    let mut prices =
        IntMap::with_capacity_and_hasher(7, BuildHasherDefault::default());

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

            for _ in 0..previous_tier {
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
                println!("Master Skull - Tier {tier} is priced {}, equals to {} coins per 4x of Tier {previous_tier} skulls, or {} coins per {tier_ones_required_to_craft_this_tier}x of Tier 1 skulls", with_comma_separators(&price.to_string()).unwrap_or_else(|| price.to_string()).yellow(), with_comma_separators(&(price / 4).to_string()).unwrap_or_else(|| (price / 4).to_string()).yellow(), with_comma_separators(&price_per_tier_one.to_string()).unwrap_or_else(|| price_per_tier_one.to_string()).yellow());
            }
        });
    }

    let mut total_required_amount =
        get_total_required_amount(best_tier_to_buy_and_combine, target_tier);
    let total_required_amount_for_current =
        get_total_required_amount(best_tier_to_buy_and_combine, current_tier);

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

fn get_total_required_amount(starting_tier: usize, ending_tier: i32) -> i64 {
    let mut total_required_amount = 1;

    match usize::try_from(ending_tier) {
        Ok(tier) =>
            for _ in starting_tier..tier {
                total_required_amount *= 4;
            },

        Err(e) => {
            println!("{}{e}", "Error converting i32 to usize: ".red());
        },
    }

    total_required_amount
}

async fn do_requests_and_extract_prices(prices: &mut IntMap<usize, i64>) -> bool {
    let mut requests = Vec::with_capacity(7);

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
                        .query(&[
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
                if !parse_request_and_insert_prices(prices, i, result_of_request)
                    .await
                {
                    return false;
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

async fn parse_request_and_insert_prices(
    prices: &mut IntMap<usize, i64>, i: usize,
    result_of_request: Result<Response, Error>,
) -> bool {
    match result_of_request {
        Ok(response) => {
            match response.text().await {
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
            }
        },

        Err(e) => {
            println!("{}{e}", "Error when getting response: ".red());

            return false;
        },
    }

    true
}
