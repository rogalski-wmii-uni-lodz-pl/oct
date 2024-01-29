use bitvec::prelude::*;
use std::collections::HashSet;

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

    pub fn find_first_unset_also_unset_in(&self, other: &Self) -> usize {
        for i in 0..other.bits.len() {
            if !self.get(i) && !other.get(i) {
                return i;
            }
        }

        self.bits.len() - 1
    }

    pub fn copy_up_to_inclusive(&self, x: usize) -> Self {
        Self {
            bits: self.bits[0..x].to_owned(),
        }
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

fn can_add_to_common(common: &HashSet<Nimpos>, np: Nimpos, parity: usize) -> bool {
    for &i in common.iter() {
        if common.contains(&xor(np, i, parity)) {
            return false;
        }
    }

    !common.contains(&0)
}

#[derive(Debug)]
pub struct Data {
    pub common: [HashSet<Nimpos>; 2],
    pub rares: [Vec<(usize, Nimber)>; 2],
    pub rare: [Bin; 2],
    pub largest: usize,
    pub even: bool,
    pub odd: bool,
}

impl Data {
    pub fn new(largest: usize, rules: &Rules) -> Self {
        let mut x = Self {
            common: [HashSet::new(), HashSet::new()],
            rares: [vec![], vec![]],
            rare: [Bin::make(largest), Bin::make(largest)],
            largest,
            even: rules.divide.iter().any(|x| x & 1 == 0),
            odd: rules.divide.iter().any(|x| x & 1 == 1),
        };

        if x.even {
            x.rare[0].set_bit(0);
        }
        if x.odd {
            x.rare[1].set_bit(0);
        }
        x
    }


    pub fn resize(self: &mut Self, largest: usize) {
        let mut rare2 = [Bin::make(largest), Bin::make(largest)];
        rare2[0].set_all_bits_from(&self.rare[0]);
        rare2[1].set_all_bits_from(&self.rare[1]);
        self.rare = rare2;
        self.largest = largest;
    }

    pub fn add_to_common(self: &mut Self, n: usize, gn: Nimber, parity: usize) {
        let np = to_nimpos(gn,n);

        if self.common[parity].contains(&np) {
            return;
        }

        if np != 0 && can_add_to_common(&self.common[parity], np, parity) {
            self.common[parity].insert(np);
            for &c in self.common[parity].iter() {
                let cc = from_nimpos(c).0;
                let x = cc ^ gn;

                self.rare[parity].set_bit(x);
            }
        } else {
            self.rare[parity].set_bit(gn);
            self.rares[parity].push((n, gn));
        }
    }

    pub fn add_to_common2(self: &mut Self, n: usize, gn: Nimber) {
        if self.even {
            self.add_to_common(n, gn, 0);
        }
        if self.odd {
            self.add_to_common(n, gn, 1);
        }
    }
}

pub fn rc(n: usize, rules: &Rules, g: &Vec<Nimber>, data: &Data) -> usize {
    let mut seen = [Bin::make(data.largest), Bin::make(data.largest)];

    // set some
    for d in rules.some.iter() {
        let x = g[n - d];
        seen[0].set_bit(x);
        seen[1].set_bit(x);
    }

    // set rare
    for &d in rules.divide.iter() {
        let parity = d & 1;
        for &(i, r) in data.rares[d & 1].iter() {
            if n - d > i {
                let gndi = g[n - d - i];
                let x = r ^ gndi;
                seen[parity].set_bit(x);
            }
        }
    }

    let first_common = [
        seen[0].find_first_unset_also_unset_in(&data.rare[0]),
        seen[1].find_first_unset_also_unset_in(&data.rare[1]),
    ];

    let mut mex = [
        seen[0].copy_up_to_inclusive(first_common[0] + 1),
        seen[1].copy_up_to_inclusive(first_common[1] + 1),
    ];

    let mut remaining_unset = [mex[0].count_unset() - 1, mex[1].count_unset() - 1]; // -1 for mex[first_common]

    if remaining_unset[0] != 0 || remaining_unset[1] != 0 {
        for i in 1..(n - rules.divide.last().unwrap()) {
            for d in rules.divide.iter() {
                let gndi = g[n - d - i];
                let gi = g[i];

                let x = gi ^ gndi;

                let parity = d & 1;
                if x < first_common[parity] && !mex[parity].get(x) {
                    mex[parity].set_bit(x);
                    remaining_unset[parity] -= 1;
                }
            }
            if remaining_unset[0] == 0 && remaining_unset[1] == 0 {
                break;
            }
        }
        if remaining_unset[0] != 0 || remaining_unset[1] != 0 {
            for i in n - rules.divide.last().unwrap()..n {
                for &d in rules.divide.iter() {
                    if n - d > i {
                        let gndi = g[n - d - i];
                        let gi = g[i];

                        let x = gi ^ gndi;

                        if x < first_common[d & 1] {
                            mex[d & 1].set_bit(x);
                        }
                    }
                }
            }
        }
    }

    if data.even && data.odd {
        mex[0].lowest_unset().min(mex[1].lowest_unset()) // ?
    } else if data.even {
        mex[0].lowest_unset()
    } else {
        mex[1].lowest_unset()
    }
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
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
