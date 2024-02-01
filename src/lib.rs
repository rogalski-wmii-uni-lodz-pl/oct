use std::{cmp::Reverse, collections::HashSet};

use bitvec::prelude::*;

use itertools::Itertools;

pub type Nimber = usize;
pub type Nimpos = usize;

pub fn to_nimpos(x: Nimber, p: usize) -> Nimpos {
    (x << 1) | (p & 1)
}

pub fn from_nimpos(x: Nimpos) -> (Nimber, usize) {
    (x >> 1, x & 1)
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
    pub divide_all: Vec<usize>,
    pub divide: [Vec<usize>; 2],
    pub len: usize,
}

/// Transform a game string like "0.034" into Rules
pub fn rules_from_str(game: &str) -> Rules {
    let vals = game
        .chars()
        .filter(|&x| x != '.')
        .map(|x| x.to_digit(10).unwrap() as usize)
        .collect_vec();

    let divide_all = extract_bit(&vals, 2);
    let (even, odd): (Vec<usize>, Vec<usize>) =
        divide_all.iter().partition(|&x| x & 1 == 0);

    Rules {
        all: extract_bit(&vals, 0),
        some: extract_bit(&vals, 1),
        divide_all,
        divide: [even, odd],
        len: vals.len() - 1,
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

fn can_add_to_common(common: &HashSet<usize>, np: Nimpos, parity: usize) -> bool {
    // 0.142
    // if np == 3 || np == 257 || np == 512 {
    //     return false;
    // }
    // 0.104
    // if np == 8 {
    //     return false;
    // }
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
pub struct Octal {
    pub common: [HashSet<usize>; 2],
    pub both_common: Bin,
    pub rares: [Vec<(usize, Nimber)>; 2],
    pub largest: usize,
    pub last_divide: usize,
    pub even: bool,
    pub odd: bool,
    pub g: Vec<Nimber>,
    pub rules: Rules,
    pub counts: Vec<usize>,
    pub seen: Bin,
}

const CHECK: bool = false;

impl Octal {
    pub fn new(g_size: usize, largest: usize, rules: Rules) -> Self {
        Self {
            both_common: Bin::make(2 * largest),
            common: [HashSet::new(), HashSet::new()],
            rares: [vec![], vec![]],
            largest,
            last_divide: *rules.divide_all.last().unwrap(),
            even: rules.divide[0].len() != 0,
            odd: rules.divide[1].len() != 0,
            rules,
            g: vec![0; g_size],
            counts: vec![0; largest << 2 + 1],
            seen: Bin::make(largest),
        }
    }

    pub fn calc(self: &mut Self, n: usize) {
        let gn = if n > self.rules.len {
            if CHECK {
                let check = self.def(n);
                let gn = self.rc(n);
                dbg!(n, gn, check);
                assert!(gn == check);
                gn
            } else {
                self.rc(n)
            }
        } else {
            self.def(n)
        };

        if self.largest < gn {
            self.largest = gn;
            self.resize(self.largest);
        }

        self.counts[to_nimpos(gn, n)] += 1;

        self.add_to_both_common_or_rare(n, gn);

        self.g[n] = gn;

        if n.is_power_of_two() {
            self.redo_common(n);
        }

        println!(
            "n: {n} val: {gn} {} {}",
            &self.rares[0].len(),
            &self.rares[1].len()
        );
        if CHECK {
            dbg!(&self.common);
            dbg!(&self.rares);
        }
    }

    pub fn def(self: &mut Self, n: usize) -> usize {
        let mut seen = Bin::make(self.largest);
        for &d in self.rules.all.iter() {
            if n == d {
                seen.set_bit(0);
            }
        }

        for &d in self.rules.some.iter() {
            if n > d {
                seen.set_bit(self.g[n - d]);
            }
        }

        for &d in self.rules.divide_all.iter() {
            if n > d {
                for i in 1..=(n - d) / 2 {
                    seen.set_bit(self.g[i] ^ self.g[n - d - i]);
                }
            }
        }

        seen.lowest_unset()
    }

    pub fn resize(self: &mut Self, largest: usize) {
        let mut common2 = Bin::make(2 * largest);
        common2.set_all_bits_from(&self.both_common);
        self.both_common = common2;
        self.largest = largest;
        self.counts.resize(largest << 2 + 1, 0);
        self.seen = Bin::make(largest);
        // self.redo_common();
    }

    pub fn redo_common(self: &mut Self, n: usize) {
        self.both_common = Bin::make(2 * self.largest);
        self.common[0].clear();
        self.common[1].clear();
        self.rares[0].clear();
        self.rares[1].clear();

        // this clone here is probably bad :(
        for (np, count) in self.counts.clone().iter().enumerate().sorted_by_key(|&(n, count)| (Reverse(count), n)) {
            if *count > 0 {
                let (n, p) = from_nimpos(np);
                println!("{n} {p} {count}");
                self.add_to_both_common(np);
            }
        }

        for i in 1..=n {
            let np = to_nimpos(self.g[i], i);
            if self.even && !self.common[0].contains(&np) {
                self.rares[0].push((i, self.g[i]));
            }
            if self.odd && !self.common[1].contains(&np) {
                self.rares[1].push((i, self.g[i]));
            }
        }
        self.rares[0].sort();
        self.rares[1].sort();

    }

    pub fn add_to_common(self: &mut Self, np: Nimpos, parity: usize) -> bool {
        if self.common[parity].contains(&np) {
            return true;
        }

        // let not_zero = match (self.even, self.odd) {
        //     (true, true) => np != parity,
        //     (true, false) => np != 0,
        //     (false, true) => np != 1,
        //     (false, false) => false, // who cares?
        // };
        //
        let not_zero = np != parity;

        if not_zero && can_add_to_common(&self.common[parity], np, parity) {
            self.common[parity].insert(np);
            true
        } else {
            false
        }
    }

    pub fn add_to_common_or_rare(self: &mut Self, n: usize, gn: Nimber, parity: usize) -> bool {
        let np = to_nimpos(gn, n);
        if self.add_to_common(np, parity) {
            true
        } else {
            self.rares[parity].push((n, gn));
            false
        }
    }

    pub fn add_to_both_common(self: &mut Self, np: Nimpos) {
        let ec = if self.even {
            self.add_to_common(np, 0)
        } else {
            true
        };
        let oc = if self.odd {
            self.add_to_common(np, 1)
        } else {
            true
        };

        if ec && oc {
            self.both_common.set_bit(np);
        }
    }


    pub fn add_to_both_common_or_rare(self: &mut Self, n: usize, gn: Nimber) {
        let ec = if self.even {
            self.add_to_common_or_rare(n, gn, 0)
        } else {
            true
        };
        let oc = if self.odd {
            self.add_to_common_or_rare(n, gn, 1)
        } else {
            true
        };

        if ec && oc {
            let np = to_nimpos(gn, n);
            self.both_common.set_bit(np);
        }
    }

    pub fn rc(self: &mut Self, n: usize) -> usize {
        self.seen.zero_bits();

        self.set_some(n);
        self.set_0_if_divisible_into_same_size(n);
        self.set_rare(n);
        self.prove(n)
    }

    fn prove(self: &mut Self, n: usize) -> usize {
        let mut m = self.seen.lowest_unset();

        let mp = to_nimpos(m, n);

        if self.both_common.get(mp) {
            return m;
        }

        for i in 1..(n - self.last_divide) {
            for d in self.rules.divide_all.iter() {
                let gndi = self.g[n - d - i];
                let gi = self.g[i];

                let x = gi ^ gndi;

                self.seen.set_bit(x);
            }

            m = self.seen.lowest_unset();

            let mp = to_nimpos(m, n);
            if self.both_common.get(mp) {
                return m;
            }
        }
        for i in n - self.last_divide..n {
            for &d in self.rules.divide_all.iter() {
                if n - d > i {
                    let gndi = self.g[n - d - i];
                    let gi = self.g[i];

                    let x = gi ^ gndi;

                    self.seen.set_bit(x);
                }
            }
        }

        self.seen.lowest_unset()
    }

    fn set_rare(self: &mut Self, n: usize) {
        // set rare
        for parity in [0, 1] {
            for &d in self.rules.divide[parity].iter() {
                for &(i, r) in self.rares[parity].iter() {
                    if n - d > i {
                        let gndi = self.g[n - d - i];
                        let x = r ^ gndi;
                        self.seen.set_bit(x);
                    }
                }
            }
        }
    }

    fn set_0_if_divisible_into_same_size(self: &mut Self, n: usize) {
        if (n & 1 == 0 && self.even) || (n & 1 == 1 && self.odd) {
            self.seen.set_bit(0);
        }
    }

    fn set_some(self: &mut Self, n: usize) {
        for d in self.rules.some.iter() {
            let x = self.g[n - d];
            self.seen.set_bit(x);
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
