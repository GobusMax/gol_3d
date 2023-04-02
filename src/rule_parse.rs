use nom::{
    branch::alt,
    bytes::complete::tag,
    character,
    combinator::{map, opt, value},
    multi::fold_many0,
    sequence::{separated_pair, terminated, tuple},
    IResult,
};

use crate::rule::{Neighborhood, Rule};

fn bitmask(input: &str) -> IResult<&str, u32> {
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
    )(input)
}

pub fn rule(input: &str) -> IResult<&str, Rule> {
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
        )),
        |(survive_mask, _, born_mask, _, max_state, _, neighborhood)| Rule {
            survive_mask,
            born_mask,
            max_state,
            neighborhood,
        },
    )(input)
}
