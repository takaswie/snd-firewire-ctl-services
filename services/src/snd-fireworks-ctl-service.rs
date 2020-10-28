// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) 2020 Takashi Sakamoto
use std::env;

fn print_help() {
    println!("
Usage:
  snd-fireworks-ctl-service CARD_ID

  where:
    CARD_ID: The numerical ID of sound card.
    ");
}

fn main() {
    // Check arguments in command line.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        std::process::exit(libc::EXIT_FAILURE);
    }

    let card_id = match args[1].parse::<u32>() {
        Ok(card_id) => card_id,
        Err(err) => {
            println!("{:?}", err);
            print_help();
            std::process::exit(libc::EXIT_FAILURE);
        }
    };

    let err = match efw::unit::EfwUnit::new(card_id) {
        Err(err) => {
            println!("The card {} is not for fireworks device: {}",
                     card_id, err);
            Err(err)
        }
        Ok(mut unit) => {
            if let Err(err) = unit.listen() {
                println!("Fail to listen events: {}", err);
                Err(err)
            } else {
                unit.run();
                Ok(())
            }
        }
    };

    if err.is_err() {
        std::process::exit(libc::EXIT_FAILURE)
    }

    std::process::exit(libc::EXIT_SUCCESS)
}
