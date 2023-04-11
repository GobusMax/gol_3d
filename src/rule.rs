use std::{fmt::Display, ops::RangeInclusive, str::FromStr};

use ndarray::Array3;
use ndarray_rand::{
    rand::{self, Rng},
    rand_distr::{Distribution, Standard},
};
use nom::Finish;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::rule_parse;

#[derive(Debug, Clone, Copy)]
pub enum Neighborhood {
    Moore,
    MooreNonWrapping,
    VonNeumann,
    VonNeumannNonWrapping,
}

impl Display for Neighborhood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Neighborhood::Moore => write!(f, "M"),
            Neighborhood::MooreNonWrapping => write!(f, "MN"),
            Neighborhood::VonNeumann => write!(f, "N"),
            Neighborhood::VonNeumannNonWrapping => write!(f, "NN"),
        }
    }
}

impl Distribution<Neighborhood> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Neighborhood {
        match rng.gen_range(0..=3) {
            1 => Neighborhood::MooreNonWrapping,
            2 => Neighborhood::VonNeumann,
            3 => Neighborhood::VonNeumannNonWrapping,
            _ => Neighborhood::Moore,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub survive_mask: u32,
    pub born_mask: u32,
    pub max_state: u8,
    pub neighborhood: Neighborhood,
}

impl Rule {
    pub fn new<T: ToBitMask, U: ToBitMask>(
        survive: T,
        born: U,
        max_state: u8,
        neighborhood: Neighborhood,
    ) -> Self {
        Self {
            survive_mask: survive.to_bit_mask(),
            born_mask: born.to_bit_mask(),
            max_state,
            neighborhood,
        }
    }
    pub fn new_random() -> Self {
        Self {
            survive_mask: rand::random::<u32>() & (u32::MAX - 1),
            born_mask: rand::random::<u32>() & (u32::MAX - 1),
            max_state: rand::random::<u8>() / 64 + 1,
            neighborhood: rand::thread_rng()
                .sample(rand::distributions::Standard),
        }
    }
    pub fn as_buffer(&self, device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Rule Buffer"),
            contents: &bytemuck::cast::<_, [u8; 16]>(RuleRaw::from(self)),
            usage: BufferUsages::UNIFORM,
        })
    }
    pub fn survive(&self, count: u8) -> bool {
        self.survive_mask & (1 << count) != 0
    }
    pub fn born(&self, count: u8) -> bool {
        self.born_mask & (1 << count) != 0
    }
    pub fn count_neighbors(
        &self,
        cells: &Array3<u8>,
        idx: (usize, usize, usize),
    ) -> u8 {
        match self.neighborhood {
            Neighborhood::Moore => self.moore_neighborhood(cells, idx),
            Neighborhood::MooreNonWrapping => {
                self.moore_neighborhood_non_wrapping(cells, idx)
            }
            Neighborhood::VonNeumann => {
                self.von_neumann_neigborhood(cells, idx)
            }

            Neighborhood::VonNeumannNonWrapping => {
                self.von_neumann_neigborhood_non_wrapping(cells, idx)
            }
        }
    }
    fn moore_neighborhood_non_wrapping(
        &self,
        cells: &Array3<u8>,
        index: (usize, usize, usize),
    ) -> u8 {
        let dim = cells.dim();
        let mut sum = 0;
        for x in -1..=1 {
            if index.0.checked_add_signed(x).unwrap_or(dim.0) < dim.0 {
                for y in -1..=1 {
                    if index.1.checked_add_signed(y).unwrap_or(dim.1) < dim.1 {
                        for z in -1..=1 {
                            if index.2.checked_add_signed(z).unwrap_or(dim.2)
                                < dim.2
                            {
                                let new_index = (
                                    (index.0).wrapping_add_signed(x),
                                    (index.1).wrapping_add_signed(y),
                                    (index.2).wrapping_add_signed(z),
                                );
                                if (x, y, z) != (0, 0, 0)
                                    && cells[new_index] == self.max_state
                                {
                                    sum += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        sum
    }

    fn moore_neighborhood(
        &self,
        cells: &Array3<u8>,
        index: (usize, usize, usize),
    ) -> u8 {
        let dim = cells.dim();
        let mut sum = 0;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let new_index = (
                        (index.0 + dim.0).wrapping_add_signed(x) % dim.0,
                        (index.1 + dim.1).wrapping_add_signed(y) % dim.1,
                        (index.2 + dim.2).wrapping_add_signed(z) % dim.2,
                    );
                    if (x, y, z) != (0, 0, 0)
                        && cells[new_index] == self.max_state
                    {
                        sum += 1;
                    }
                }
            }
        }
        sum
    }

    fn von_neumann_neigborhood_non_wrapping(
        &self,
        cells: &Array3<u8>,
        index: (usize, usize, usize),
    ) -> u8 {
        let dim = cells.dim();
        let mut sum = 0;

        if index.0 + 1 < dim.0
            && cells[(index.0 + 1, index.1, index.2)] == self.max_state
        {
            sum += 1;
        }
        if index.1 + 1 < dim.1
            && cells[(index.0, index.1 + 1, index.2)] == self.max_state
        {
            sum += 1;
        }
        if index.2 + 1 < dim.2
            && cells[(index.0, index.1, index.2 + 1)] == self.max_state
        {
            sum += 1;
        }
        if index.0 > 0
            && cells[(index.0 - 1, index.1, index.2)] == self.max_state
        {
            sum += 1;
        }
        if index.1 > 0
            && cells[(index.0, index.1 - 1, index.2)] == self.max_state
        {
            sum += 1;
        }
        if index.2 > 0
            && cells[(index.0, index.1, index.2 - 1)] == self.max_state
        {
            sum += 1;
        }
        sum
    }
    fn von_neumann_neigborhood(
        &self,
        cells: &Array3<u8>,
        idx: (usize, usize, usize),
    ) -> u8 {
        let dim = cells.dim();
        let mut sum = 0;

        if cells[((idx.0 + 1) % dim.0, idx.1, idx.2)] == self.max_state {
            sum += 1;
        }
        if cells[(idx.0, (idx.1 + 1) % dim.1, idx.2)] == self.max_state {
            sum += 1;
        }
        if cells[(idx.0, idx.1, (idx.2 + 1) % dim.2)] == self.max_state {
            sum += 1;
        }
        if cells[((idx.0 + dim.0 - 1) % dim.0, idx.1, idx.2)] == self.max_state
        {
            sum += 1;
        }
        if cells[(idx.0, (idx.1 + dim.1 - 1) % dim.1, idx.2)] == self.max_state
        {
            sum += 1;
        }
        if cells[(idx.0, idx.1, (idx.2 + dim.2 - 1) % dim.2)] == self.max_state
        {
            sum += 1;
        }
        sum
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "Rule{{survive_mask: {:#034b}, born_mask: {:#034b}, max_state:{}, neighborhood: rule::Neighborhood::{:?}}};",
        //     self.survive_mask,
        //     self.born_mask,
        //     self.max_state,
        //     self.neighborhood)
        // write!(
        //     f,
        //     "0b{:b}/0b{:b}/{}/{}",
        //     self.survive_mask,
        //     self.born_mask,
        //     self.max_state+1,
        //     self.neighborhood
        // )
        write!(
            f,
            "{}/{}/{}/{}",
            bit_run_string(self.survive_mask as u64),
            bit_run_string(self.born_mask as u64),
            self.max_state + 1,
            self.neighborhood
        )
    }
}

impl FromStr for Rule {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match rule_parse::rule_and_init(s).finish() {
            Ok((_, (r, _))) => Ok(r),
            Err(nom::error::Error { input, code }) => Err(nom::error::Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}

pub trait ToBitMask {
    fn to_bit_mask(self) -> u32;
}
impl ToBitMask for RangeInclusive<u8> {
    fn to_bit_mask(self) -> u32 {
        let mut mask = 0;
        for i in self {
            mask |= 1 << i;
        }
        mask
    }
}
impl<F> ToBitMask for F
where
    F: Fn(u8) -> bool,
{
    fn to_bit_mask(self) -> u32 {
        let mut mask = 0;
        for i in 0..=27 {
            if self(i) {
                mask |= 1 << i;
            }
        }
        mask
    }
}
impl ToBitMask for u8 {
    fn to_bit_mask(self) -> u32 {
        1 << self
    }
}
impl ToBitMask for &str {
    fn to_bit_mask(self) -> u32 {
        let mut mask = 0;
        for p in self.split(',') {
            if p.contains('-') {
                let mut split = p.split('-');
                let range = (split.next().unwrap().parse::<u8>().unwrap()
                    ..=split.next().unwrap().parse::<u8>().unwrap())
                    .to_bit_mask();
                mask |= range;
            } else if !p.is_empty() {
                mask |= 1 << p.parse::<u8>().unwrap();
            }
        }
        mask
    }
}

fn bit_run_list(mut n: u64) -> Vec<(u8, u8)> {
    let mut res = Vec::new();
    let mut run_start = None;
    let mut i = 0;
    while n != 0 || run_start.is_some() {
        if n & 1 != 0 && run_start.is_none() {
            run_start = Some(i);
        } else if n & 1 == 0 {
            if let Some(start) = run_start {
                let end = i - 1;
                res.push((start, end));
                run_start = None;
            }
        }

        n >>= 1;
        i += 1;
    }

    res
}

fn bit_run_string(n: u64) -> String {
    let bit_runs = bit_run_list(n);
    let mut res = String::new();
    for (start, end) in bit_runs {
        if !res.is_empty() {
            res += ",";
        }
        if start == end {
            res += &format!("{}", start);
        } else {
            res += &format!("{}-{}", start, end);
        }
    }

    res
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RuleRaw {
    pub survive_mask: u32,
    pub born_mask: u32,
    pub max_state: u32,
    pub neighborhood: u32,
}
impl From<&Rule> for RuleRaw {
    fn from(rule: &Rule) -> Self {
        let neighborhood = match rule.neighborhood {
            Neighborhood::Moore => 0,
            Neighborhood::MooreNonWrapping => 1,
            Neighborhood::VonNeumann => 2,
            Neighborhood::VonNeumannNonWrapping => 3,
        };
        Self {
            survive_mask: rule.survive_mask,
            born_mask: rule.born_mask,
            max_state: rule.max_state as u32,
            neighborhood,
        }
    }
}
