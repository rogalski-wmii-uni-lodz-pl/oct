use oct;


fn main() {
    let rules = oct::rules_from_str("0.142");
    // let rules = oct::rules_from_str("0.106");
    // let rules = oct::rules_from_str("0.051");

    let mut g = vec![0; 10000000];

    let mut largest = 1;

    let mut data = oct::Data::new(largest, &rules);
    dbg!(&rules);

    for n in 1..100000 {
        let gn = oct::def(n, &rules, &g, largest);

        if n > 4 {
            let gn2 = oct::rc(n, &rules, &g, &data);
            println!("{gn} {gn2}");
            assert!(gn == gn2);
        }

        if largest < gn {
            largest = gn;
            data.resize(largest);
        }

        data.add_to_common2(n, gn);

        g[n] = gn;

        println!("n: {n} val: {gn} {} {}", &data.rares[0].len(), &data.rares[1].len());
        // dbg!(&data);
    }

    dbg!(&data.rare.len());
}
