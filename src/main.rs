use oct;

fn main() {
    // let rules = oct::rules_from_str("0.142");
    // let rules = oct::rules_from_str("0.104"); // breaks down for 12: (4, 0)
    // let rules = oct::rules_from_str("0.051");
    //
    // let game = "0.205";
    // let game = "0.142";
    // let game = "0.051";
    // let game = "0.104";
    // let game = "0.106";
    // let game = "0.051";
    let game = "0.166";
    let rules = oct::rules_from_str(game);
    dbg!(&rules);

    // let mut g = vec![0; 1_000_000];
    let mut octal = oct::Octal::new(1_000_000, 1, rules);

    for n in 1..octal.g.len() {
        octal.calc(n);
    }
    // dbg!(&octal);

    for (n, &count) in octal.counts.iter().enumerate() {
        if count != 0 {
            println!(
                "{n}: {count} {} {}",
                octal.rares[0].contains(&oct::from_nimpos(n)),
                octal.rares[1].contains(&oct::from_nimpos(n))
            );
        }
    }

    // dbg!(&data.common_bitset.len());
}
