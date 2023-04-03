use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character,
    combinator::{map, map_res, opt, value},
    multi::fold_many0,
    number,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

use crate::{
    rule::{Neighborhood, Rule},
    Init,
};

fn bitmask(input: &str) -> IResult<&str, u32> {
    alt((
        preceded(
            tag("0b"),
            map_res(take_while1(|c| c == '0' || c == '1'), |s| {
                u32::from_str_radix(s, 2)
            }),
        ),
        fold_many0(
            terminated(
                alt((
                    // Range of bits
                    map(
                        separated_pair(
                            character::complete::u8,
                            tag("-"),
                            character::complete::u8,
                        ),
                        |(l, r)| ((1 << l) - 1) ^ ((1 << (r + 1)) - 1),
                    ),
                    // Single bit
                    map(character::complete::u8, |n| 1 << n),
                )),
                // Not quite correct, better would be a `separated_fold0`, like
                // `separated_list0`, but that doesn't exist :(
                opt(tag(",")),
            ),
            || 0,
            |acc, m| (acc | m),
        ),
    ))(input)
}

pub fn rule_and_init(input: &str) -> IResult<&str, (Rule, Init)> {
    map(
        tuple((
            bitmask,
            tag("/"),
            bitmask,
            tag("/"),
            character::complete::u8,
            tag("/"),
            alt((
                value(Neighborhood::Moore, tag("M")),
                value(Neighborhood::MooreNonWrapping, tag("MN")),
                value(Neighborhood::VonNeumann, tag("N")),
                value(Neighborhood::VonNeumannNonWrapping, tag("NN")),
            )),
            opt(preceded(
                tag("/"),
                map(character::complete::u64, |n| n as usize),
            )),
            opt(preceded(tag("/"), number::complete::double)),
        )),
        |(
            survive_mask,
            _,
            born_mask,
            _,
            max_state,
            _,
            neighborhood,
            init_size,
            init_density,
        )| {
            (
                Rule {
                    survive_mask,
                    born_mask,
                    max_state,
                    neighborhood,
                },
                Init {
                    size: init_size.unwrap_or(2),
                    density: init_density.unwrap_or(1.),
                },
            )
        },
    )(input)
}
