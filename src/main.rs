use oct;

const CHECK: bool = false;

fn main() {
    // let rules = oct::rules_from_str("0.142");
    // let rules = oct::rules_from_str("0.104"); // breaks down for 12: (4, 0)
    // let rules = oct::rules_from_str("0.051");
    //
    // let game = "0.205";
    // let game = "0.142";
    // let game = "0.051";
    let game = "0.104";
    // let game = "0.106";
    // let game = "0.051";
    // let game = "0.165";
    let rules = oct::rules_from_str(game); // breaks down for (2, 1) and (45, 8)

    // let mut g = vec![0; 1_000_000];
    let mut g = vec![0; 1_000_000];

    let mut largest = 1;

    let mut data = oct::Data::new(largest, &rules);
    dbg!(&rules);

    for n in 1..g.len() {
        let gn = if n > game.len() {
            if CHECK {
                let check = oct::def(n, &rules, &g, largest);
                let gn = oct::rc(n, &rules, &g, &data);
                dbg!(n, gn, check);
                assert!(gn == check);
                gn
            } else {
                oct::rc(n, &rules, &g, &data)
            }
        } else {
            oct::def(n, &rules, &g, largest)
        };

        if largest < gn {
            largest = gn;
            data.resize(largest);
        }

        data.add_to_common2(n, gn);

        g[n] = gn;

        println!(
            "n: {n} val: {gn} {} {}",
            &data.rares[0].len(),
            &data.rares[1].len()
        );
        if CHECK {
            dbg!(&data);
        }
    }
    dbg!(&data);

    // dbg!(&data.common_bitset.len());
}
