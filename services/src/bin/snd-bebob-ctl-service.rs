// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::env;
use bebob::runtime::BebobRuntime;
use core::RuntimeOperation;

fn main() {
    // Check arguments in command line.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("At least, one argument is required for: ");
        println!("  The numerical ID of sound card.");
        std::process::exit(1);
    }

    let card_id = match args[1].parse::<u32>() {
        Ok(card_id) => card_id,
        Err(err) => {
            println!("{:?}", err);
            std::process::exit(1);
        }
    };

    let err = match BebobRuntime::new(card_id) {
        Err(err) => {
            println!("The {} is not for BeBoB device: {}", card_id, err);
            Err(err)
        }
        Ok(mut unit) => {
            if let Err(err) = unit.listen() {
                println!("Fail to listen events: {}", err);
                Err(err)
            } else {
                unit.run()
            }
        }
    };

    if err.is_err() {
        std::process::exit(1)
    }

    std::process::exit(0)
}
