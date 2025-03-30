use super::constants::Endianness;
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    combinator::{map, verify},
    error::context,
};

/// Header metadata describing the bytecode format and target architecture
#[derive(Debug)]
#[allow(dead_code)]
pub struct Header {
    pub version: u8,            // Lua version (e.g., 0x51 for Lua 5.1)
    pub format: u8,             // Bytecode format (0 for official Lua bytecode)
    pub endianness: Endianness, // Byte order (Big or Little Endian)
    pub size_int: u8,           // Size of an integer in bytes
    pub size_size_t: u8,        // Size of a size_t value in bytes
    pub size_instruction: u8,   // Size of an instruction in bytes
    pub size_number: u8,        // Size of a number in bytes

    /// Whether numbers (constants) are stored as integers (`true`) or floats (`false`)
    pub integral_flag: bool,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, _) = context("invalid magic number", tag(&b"\x1BLua"[..])).parse(input)?;

    let (input, version) = context(
        "invalid Lua version (must be 0x51)",
        verify(nom::number::complete::u8, |&v| v == 0x51),
    )
    .parse(input)?;
    let (input, format) = context(
        "unsupported format (must be 0 (official))",
        verify(nom::number::complete::u8, |&f| f == 0),
    )
    .parse(input)?;

    let (input, endianness) = map(nom::number::complete::u8, |b| match b {
        1 => Endianness::Little,
        _ => Endianness::Big,
    })
    .parse(input)?;

    let parse_size = |name, expected| {
        context(
            name,
            verify(nom::number::complete::u8, move |&v| v == expected),
        )
    };
    let (input, size_int) = parse_size("invalid int size", 4).parse(input)?;
    let (input, size_size_t) = parse_size("invalid size_t size", 8).parse(input)?;
    let (input, size_instruction) = parse_size("invalid instruction size", 4).parse(input)?;
    let (input, size_number) = parse_size("invalid number size", 8).parse(input)?;
    let (input, integral_flag) = map(nom::number::complete::u8, |b| b != 0).parse(input)?;

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

    crate::debug_println!("Parsed header: {:#?}", header);

    Ok((input, header))
}
