use std::{
    collections::BTreeMap as Map,
    env::args,
    io::{self, stdout, Write},
};

use rand::{thread_rng, Rng};

fn main() -> io::Result<()> {
    let mut args = args().skip(1);
    let amount = args
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(255);
    let tries = args
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1000000);
    eprintln!("Outputting {amount} random bytes...");
    eprintln!("Aiming for highest distribution and trying {tries} times");

    let mut out = stdout();
    out.write_all(&amount.to_ne_bytes())?;
    out.flush()?;

    let mut final_bytes = Vec::new();
    let mut final_distribution = Map::new();

    for _ in 0..tries {
        let mut distribution = Map::new();
        let mut bytes = Vec::new();

        let mut rng = thread_rng();
        for _ in 0..amount {
            let rand = rng.gen::<u8>();
            bytes.push(rand);
            if let Some(current) = distribution.get_mut(&rand) {
                *current += 1;
            } else {
                distribution.insert(rand, 1);
            }
        }

        if distribution.len() > final_distribution.len() {
            final_distribution = distribution;
            final_bytes = bytes;
        }
    }

    eprintln!(
        "Distribution of values, in total {} different values:",
        final_distribution.len()
    );
    for vals in final_distribution.into_iter().collect::<Vec<_>>().chunks(9) {
        for (val, rate) in vals {
            eprint!("{: >4}: {: <4} | ", val, rate);
        }
        eprintln!()
    }

    for num in final_bytes {
        out.write_all(&num.to_ne_bytes())?;
    }
    stdout().flush()?;

    Ok(())
}
