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

// Constants for errors
const ERROR_INVALID_MAGIC_NUMBER: &str = "invalid magic number";
const ERROR_INVALID_VERSION: &str = "invalid Lua version (must be 0x51)";
const ERROR_INVALID_FORMAT: &str = "unsupported format (must be 0 (official))";
const ERROR_INVALID_SIZE_INT: &str = "invalid int size";
const ERROR_INVALID_SIZE_SIZE_T: &str = "invalid size_t size";
const ERROR_INVALID_SIZE_INSTRUCTION: &str = "invalid instruction size";
const ERROR_INVALID_SIZE_NUMBER: &str = "invalid number size";

/// Parsing functions module
mod parsers {
    use super::*;

    pub fn parse_magic_number(input: &[u8]) -> IResult<&[u8], &[u8]> {
        context(ERROR_INVALID_MAGIC_NUMBER, tag(MAGIC_NUMBER)).parse(input)
    }

    pub fn parse_version(input: &[u8]) -> IResult<&[u8], u8> {
        context(
            ERROR_INVALID_VERSION,
            verify(nom::number::complete::u8, |&v| v == EXPECTED_VERSION),
        )
        .parse(input)
    }

    pub fn parse_format(input: &[u8]) -> IResult<&[u8], u8> {
        context(
            ERROR_INVALID_FORMAT,
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

    let (input, size_int) = parse_size(ERROR_INVALID_SIZE_INT, EXPECTED_SIZE_INT, input)?;
    let (input, size_size_t) = parse_size(ERROR_INVALID_SIZE_SIZE_T, EXPECTED_SIZE_SIZE_T, input)?;
    let (input, size_instruction) = parse_size(
        ERROR_INVALID_SIZE_INSTRUCTION,
        EXPECTED_SIZE_INSTRUCTION,
        input,
    )?;
    let (input, size_number) = parse_size(ERROR_INVALID_SIZE_NUMBER, EXPECTED_SIZE_NUMBER, input)?;
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
