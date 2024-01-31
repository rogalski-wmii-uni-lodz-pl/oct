use std::collections::HashSet;

use bitvec::prelude::*;

use itertools::Itertools;

pub type Nimber = usize;
pub type Nimpos = usize;

pub fn to_nimpos(x: Nimber, p: usize) -> Nimpos {
    (x << 1) | (p & 1)
}

pub fn from_nimpos(x: Nimpos) -> (Nimber, bool) {
    (x >> 1, x & 1 == 1)
}

pub fn xor(x: Nimpos, y: Nimpos, d: usize) -> Nimpos {
    x ^ y ^ (d & 1)
}

#[derive(Clone, Debug)]
pub struct Bin {
    bits: BitVec<u64, Msb0>,
}

#[cfg(any(
    feature = "bits_bitvec",
    not(any(feature = "bits_u32", feature = "bits_u64", feature = "bits_u128"))
))]
impl Bin {
    pub fn set_bit(&mut self, x: usize) {
        self.bits.set(x, true);
    }

    pub fn zero_bits(&mut self) {
        self.bits.set_elements(0);
    }

    pub fn get(&self, x: usize) -> bool {
        self.bits[x]
    }

    pub fn lowest_unset(&self) -> usize {
        self.bits.first_zero().unwrap()
    }

    pub fn make(largest: Nimber) -> Self {
        let bs = 2 * (largest as usize).next_power_of_two() + 2;
        Self {
            bits: bitvec!(u64, Msb0; 0; bs),
        }
    }

    pub fn count_unset(&self) -> usize {
        self.bits.count_zeros()
    }

    pub fn set_all_bits_from(&mut self, other: &Self) {
        self.bits |= &other.bits
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rules {
    pub all: Vec<usize>,
    pub some: Vec<usize>,
    pub divide: Vec<usize>,
}

/// Transform a game string like "0.034" into Rules
pub fn rules_from_str(game: &str) -> Rules {
    let vals = game
        .chars()
        .filter(|&x| x != '.')
        .map(|x| x.to_digit(10).unwrap() as usize)
        .collect_vec();

    Rules {
        all: extract_bit(&vals, 0),
        some: extract_bit(&vals, 1),
        divide: extract_bit(&vals, 2),
    }
}

fn extract_bit(vals: &Vec<usize>, b: usize) -> Vec<usize> {
    let bit = 1 << b;
    vals.iter()
        .enumerate()
        .filter_map(|(i, v)| {
            if v & bit == bit && i != 0 {
                Some(i)
            } else {
                None
            }
        })
        .collect_vec()
}

pub fn def(n: usize, rules: &Rules, g: &Vec<Nimber>, largest: usize) -> usize {
    let mut seen = Bin::make(largest);
    for &d in rules.all.iter() {
        if n == d {
            seen.set_bit(0);
        }
    }

    for &d in rules.some.iter() {
        if n > d {
            seen.set_bit(g[n - d]);
        }
    }

    for &d in rules.divide.iter() {
        if n > d {
            for i in 1..=(n - d) / 2 {
                seen.set_bit(g[i] ^ g[n - d - i]);
            }
        }
    }

    seen.lowest_unset()
}

fn can_add_to_common(common: &HashSet<usize>, np: Nimpos, parity: usize) -> bool {
    // 0.142
    // if np == 3 || np == 257 || np == 512 {
    //     return false;
    // }
    // 0.104
    if np == 8 {
        return false;
    }
    let with_itself = xor(np, np, parity);
    if with_itself == np || common.contains(&with_itself) {
        return false;
    }
    for &i in common {
        let x = xor(np, i, parity);
        if common.contains(&x) {
            return false;
        }
    }

    !common.contains(&parity)
}

#[derive(Debug)]
pub struct Data {
    pub common: [HashSet<usize>; 2],
    pub both_common: Bin,
    pub rares: [Vec<(usize, Nimber)>; 2],
    pub largest: usize,
    pub even: bool,
    pub odd: bool,
}

impl Data {
    pub fn new(largest: usize, rules: &Rules) -> Self {
        Self {
            both_common: Bin::make(2 * largest),
            common: [HashSet::new(), HashSet::new()],
            rares: [vec![], vec![]],
            largest,
            even: rules.divide.iter().any(|x| x & 1 == 0),
            odd: rules.divide.iter().any(|x| x & 1 == 1),
        }
    }

    pub fn resize(self: &mut Self, largest: usize) {
        let mut common2 = Bin::make(2 * largest);
        common2.set_all_bits_from(&self.both_common);
        self.both_common = common2;
        self.largest = largest;
    }

    pub fn add_to_common(self: &mut Self, n: usize, gn: Nimber, parity: usize) -> bool {
        let np = to_nimpos(gn, n);
        if self.common[parity].contains(&np) {
            return true;
        }

        let not_zero = match (self.even, self.odd) {
            (true, true) => np != parity,
            (true, false) => np != 0,
            (false, true) => np != 1,
            (false, false) => false, // who cares?
        };

        if not_zero && can_add_to_common(&self.common[parity], np, parity) {
            self.common[parity].insert(np);
            // self.both_common.set_bit(np);
            true
        } else {
            self.rares[parity].push((n, gn));
            false
        }
    }

    pub fn add_to_common2(self: &mut Self, n: usize, gn: Nimber) {
        let ec = if self.even {
            self.add_to_common(n, gn, 0)
        } else {
            true
        };
        let oc = if self.odd {
            self.add_to_common(n, gn, 1)
        } else {
            true
        };

        if ec && oc {
            let np = to_nimpos(gn, n);
            self.both_common.set_bit(np);
        }
    }
}

pub fn rc(n: usize, rules: &Rules, g: &Vec<Nimber>, data: &Data) -> usize {
    let mut seen = Bin::make(data.largest);

    // set some
    for d in rules.some.iter() {
        let x = g[n - d];
        seen.set_bit(x);
    }

    if (n & 1 == 0 && data.even) || (n & 1 == 1 && data.odd) {
        seen.set_bit(0);
    }

    // set rare
    for &d in rules.divide.iter() {
        for &(i, r) in data.rares[d & 1].iter() {
            if n - d > i {
                let gndi = g[n - d - i];
                let x = r ^ gndi;
                seen.set_bit(x);
            }
        }
    }

    let mut m = seen.lowest_unset();

    let mp = to_nimpos(m, n);

    if data.both_common.get(mp) {
        return m;
    }

    for i in 1..(n - rules.divide.last().unwrap()) {
        for d in rules.divide.iter() {
            let gndi = g[n - d - i];
            let gi = g[i];

            let x = gi ^ gndi;

            seen.set_bit(x);
        }

        m = seen.lowest_unset();

        let mp = to_nimpos(m, n);
        if data.both_common.get(mp) {
            return m;
        }
    }
    for i in n - rules.divide.last().unwrap()..n {
        for &d in rules.divide.iter() {
            if n - d > i {
                let gndi = g[n - d - i];
                let gi = g[i];

                let x = gi ^ gndi;

                seen.set_bit(x);
            }
        }
    }

    seen.lowest_unset()

    // if data.even && data.odd {
    //     assert!(m[0] == m[1]);
    //     return m[0];
    // } else if data.even {
    //     return m[0];
    // } else if data.odd {
    //     return m[1];
    // }
    // return seen.;
}

// pub fn rc2(
//     n: usize,
//     rules: &Rules,
//     g: &Vec<Nimber>,
//     rares: &Vec<(usize, Nimber)>,
//     rare: &Bin,
//     largest: Nimber,
// ) {
//     let mut seen = Bin::make(largest);

//     // set some
//     for d in rules.some.iter() {
//         seen.set_bit(g[n - d]);
//     }

//     // set rare
//     for (i, r) in rares.iter() {
//         for d in rules.divide.iter() {
//             seen.set_bit(r ^ g[n - d - i]);
//         }
//     }

//     let first_common = seen.find_first_unset_also_unset_in(&rare);

//     let mut mex = seen.copy_up_to_inclusive(first_common + 1);

//     let mut remaining_unset = mex.count_unset() - 1; // -1 for mex[first_common]
// }

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
