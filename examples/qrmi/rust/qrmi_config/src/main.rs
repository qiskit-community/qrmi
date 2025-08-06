// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use clap::Parser;
use qrmi::models::Config;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "Parsing qrmi_config.json file")]
struct Args {
    /// qrmi_config.json file
    #[arg(short, long)]
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::load(&args.file).unwrap();

    println!("{:#?}", config.resource_map);

    Ok(())
}
