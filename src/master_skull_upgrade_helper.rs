use core::cmp::min;
use core::time::Duration;
use std::time::Instant;

use colored::Colorize;
use futures::StreamExt;
use futures::stream::FuturesOrdered;
use nohash_hasher::BuildNoHashHasher;
use nohash_hasher::IntMap;
use reqwest::Error;
use reqwest::Response;
use reqwest::tls::Version;
use serde_json::Value;

use crate::utils::ask_int_input;
use crate::utils::with_comma_separators;

#[inline]
pub(crate) async fn upgrade_calculator_for_master_skulls(
    start_without_user_input: &mut Option<Instant>,
) -> bool {
    let current_tier = ask_int_input(
        "Enter your current Master Skull tier: ",
        Some(1),
        Some(7),
    );
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
            "You already have the Tier 7 Master Skull, exiting."
                .bright_green()
        );

        return true;
    }

    let mut prices =
        IntMap::with_capacity_and_hasher(7, BuildNoHashHasher::default());

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

    if total_required_amount != 1 || total_required_amount_for_current != 1 {
        total_required_amount -= total_required_amount_for_current;
    }

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
        eprintln!(
            "{}",
            "Critical error(s) occurred while running the program. Please read above for details.".red()
        );

        return false;
    }

    true
}

#[inline]
#[must_use]
pub(crate) fn get_total_required_amount(
    starting_tier: usize,
    ending_tier: i32,
) -> i64 {
    let mut total_required_amount = 1;

    match usize::try_from(ending_tier) {
        Ok(tier) =>
            for _ in starting_tier..tier {
                total_required_amount *= 4;
            },

        Err(e) => {
            eprintln!("{}{e}", "Error converting i32 to usize: ".red());
        },
    }

    total_required_amount
}

#[inline]
async fn do_requests_and_extract_prices(
    prices: &mut IntMap<usize, i64>,
) -> bool {
    let mut requests = Vec::with_capacity(7);
    let client = reqwest::ClientBuilder::new()
        .https_only(true)
        .http3_prior_knowledge()
        .timeout(Duration::from_secs(10))
        .min_tls_version(Version::TLS_1_3)
        .brotli(true)
        .build();

    match client {
        Ok(resulting_client) =>
            for i in 1..8 {
                let id = format!("MASTER_SKULL_TIER_{i}");

                requests.push(
                    resulting_client
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
                        .header("Alt-Used", "api.slothpixel.me")
                        .header("Connection", "keep-alive")
                        .header("DNT", "1")
                        .header("Host", "api.slothpixel.me")
                        .header("Sec-Fetch-Dest", "document")
                        .header("Sec-Fetch-Mode", "navigate")
                        .header("Sec-Fetch-Site", "none")
                        .header("Sec-Fetch-User", "?1")
                        .header("Sec-GPC", "1")
                        .header("TE", "trailers")
                        .header("Upgrade-Insecure-Requests", "1")
                        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:122.0) Gecko/20100101 Firefox/122.0")
                        .send(),
                );
            },

        Err(e) => {
            eprintln!("{}{e}", "Error when building http client: ".red());
        },
    }

    let mut completion_stream =
        requests.into_iter().map(tokio::spawn).collect::<FuturesOrdered<_>>();
    let mut i = 0;

    while let Some(result_of_task) = completion_stream.next().await {
        match result_of_task {
            Ok(result_of_request) => {
                if !parse_request_and_insert_prices(
                    prices,
                    i,
                    result_of_request,
                )
                .await
                {
                    return false;
                }
            },

            Err(e) => {
                eprintln!("{}{e}", "Error on task execution: ".red());

                return false;
            },
        }

        i += 1;
    }

    true
}

#[inline]
async fn parse_request_and_insert_prices(
    prices: &mut IntMap<usize, i64>,
    i: usize,
    result_of_request: Result<Response, Error>,
) -> bool {
    match result_of_request {
        Ok(response) => {
            match response.text().await {
                Ok(response_body) => {
                    // println!("Received response: {response_body}");
                    match serde_json::from_str::<Value>(&response_body) {
                        Ok(json) => {
                            json.get("matching_query").map_or_else(|| {
                                eprintln!("{}{response_body}", "error: can't find matching_query field in JSON: ".red());
                            }, |matching_query| {
                                matching_query.as_i64().map_or_else(|| {
                                    eprintln!("{}{matching_query}", "error: matching_query field value is not an i64: ".red());
                                }, |matches| {
                                    if matches >= 1 { // Available for sale
                                        json.get("auctions").map_or_else(|| {
                                            eprintln!("{}{response_body}", "error: can't find auctions field in JSON: ".red());
                                        }, |auctions| {
                                            auctions.as_array().map_or_else(|| {
                                                eprintln!("{}{auctions}", "error: auctions field is not an array: ".red());
                                            }, |auctions_array| {
                                                auctions_array.first().map_or_else(|| {
                                                    eprintln!("{}{response_body}", "error: can't find the first auction in the auctions list while matching_query was >= 1: ".red());
                                                }, |auction| {
                                                    auction.as_object().map_or_else(|| {
                                                        eprintln!("{}{auction}", "error: auction data is not a Map: ".red());
                                                    }, |auction_map| {
                                                        auction_map.get("starting_bid").map_or_else(|| {
                                                            eprintln!("{}{response_body}", "error: can't find starting_bid field in auction JSON: ".red());
                                                        }, |starting_bid| {
                                                            starting_bid.as_i64().map_or_else(|| {
                                                                eprintln!("{}{starting_bid}", "error: starting_bid field is not an i64: ".red());
                                                            }, |price| {
                                                                if prices.insert(i + 1, price).is_some() {
                                                                    eprintln!("error: duplicate value at index {}, updating the value and continuing", i + 1);
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
                            eprintln!(
                                "{}{e}: {response_body}",
                                "Error when parsing JSON: ".red()
                            );

                            return false;
                        },
                    }
                },

                Err(e) => {
                    eprintln!(
                        "{}{e}",
                        "Error when getting response body: ".red()
                    );

                    return false;
                },
            }
        },

        Err(e) => {
            eprintln!("{}{e}", "Error when getting response: ".red());

            return false;
        },
    }

    true
}
