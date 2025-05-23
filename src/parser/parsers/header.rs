use super::super::bytecode::{Endianness, Header};
use log::debug;
use nom::{
    bytes::complete::tag,
    combinator::{map, verify},
    error::context,
    IResult, Parser,
};

// Constants for validation
const MAGIC_NUMBER: &[u8] = b"\x1BLua";
const EXPECTED_VERSION: u8 = 0x51;
const EXPECTED_FORMAT: u8 = 0;
const EXPECTED_SIZE_INT: u8 = 4;
const EXPECTED_SIZE_SIZE_T: u8 = 8;
const EXPECTED_SIZE_INSTRUCTION: u8 = 4;
const EXPECTED_SIZE_NUMBER: u8 = 8;

/// Parsing functions module
mod parsers {
    use super::*;

    pub fn parse_magic_number(input: &[u8]) -> IResult<&[u8], &[u8]> {
        context("invalid magic number", tag(MAGIC_NUMBER)).parse(input)
    }

    pub fn parse_version(input: &[u8]) -> IResult<&[u8], u8> {
        context(
            "invalid Lua version (must be 0x51)",
            verify(nom::number::complete::u8, |&v| v == EXPECTED_VERSION),
        )
        .parse(input)
    }

    pub fn parse_format(input: &[u8]) -> IResult<&[u8], u8> {
        context(
            "unsupported format (must be 0 (official))",
            verify(nom::number::complete::u8, |&f| f == EXPECTED_FORMAT),
        )
        .parse(input)
    }

    pub fn parse_endianness(input: &[u8]) -> IResult<&[u8], Endianness> {
        map(nom::number::complete::u8, |b| match b {
            1 => Endianness::Little,
            _ => Endianness::Big,
        })
        .parse(input)
    }

    pub fn parse_size<'a>(
        name: &'static str,
        expected: u8,
        input: &'a [u8],
    ) -> IResult<&'a [u8], u8> {
        context(
            name,
            verify(nom::number::complete::u8, move |&v| v == expected),
        )
        .parse(input)
    }

    pub fn parse_integral_flag(input: &[u8]) -> IResult<&[u8], bool> {
        map(nom::number::complete::u8, |b| b != 0).parse(input)
    }
}

use parsers::*;

/// Parse the header of the Lua bytecode
pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, _) = parse_magic_number(input)?;
    let (input, version) = parse_version(input)?;
    let (input, format) = parse_format(input)?;
    let (input, endianness) = parse_endianness(input)?;

    let (input, size_int) = parse_size("invalid int size", EXPECTED_SIZE_INT, input)?;
    let (input, size_size_t) = parse_size("invalid size_t size", EXPECTED_SIZE_SIZE_T, input)?;
    let (input, size_instruction) =
        parse_size("invalid instruction size", EXPECTED_SIZE_INSTRUCTION, input)?;
    let (input, size_number) = parse_size("invalid number size", EXPECTED_SIZE_NUMBER, input)?;
    let (input, integral_flag) = parse_integral_flag(input)?;

    let header = Header {
        version,
        format,
        endianness,
        size_int,
        size_size_t,
        size_instruction,
        size_number,
        integral_flag,
    };

    debug!("Parsed header: {:#?}", header);

    Ok((input, header))
}
