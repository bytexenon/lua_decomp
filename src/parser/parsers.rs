use super::constants::{Constant, Endianness};
use super::header::Header;
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take},
    combinator::{map, map_res},
    error::ErrorKind,
    number::complete::{be_u32, be_u64, le_u32, le_u64, u8},
};

/// Parses a 32-bit integer with specified endianness
pub fn parse_integer<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], i32> {
    match header.endianness {
        Endianness::Big => be_u32.map(|v| v as i32).parse(input),
        Endianness::Little => le_u32.map(|v| v as i32).parse(input),
    }
}

/// Parses a size_t value according to header specifications
pub fn parse_size_t<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], u64> {
    match (header.size_size_t, header.endianness) {
        (4, Endianness::Big) => be_u32.map(|v| v as u64).parse(input),
        (4, Endianness::Little) => le_u32.map(|v| v as u64).parse(input),
        (8, Endianness::Big) => be_u64.parse(input),
        (8, Endianness::Little) => le_u64.parse(input),
        _ => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            ErrorKind::Verify,
        ))),
    }
}

/// Parses a length-prefixed string with null terminator
pub fn parse_string<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], String> {
    let (input, len) = parse_size_t(input, header)?;
    if len == 0 {
        return Ok((input, String::new()));
    }

    let len_minus_1 = len
        .checked_sub(1)
        .ok_or_else(|| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::Verify)))?;

    let len_usize = usize::try_from(len_minus_1)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;

    let (input, bytes) = take(len_usize)(input)?;
    let (input, _) = tag(&b"\x00"[..])(input)?;

    Ok((input, String::from_utf8_lossy(bytes).into_owned()))
}

/// Parses a single instruction (4 bytes) with specified endianness
pub fn parse_instruction<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], u32> {
    match header.endianness {
        Endianness::Big => map(be_u32, |v| v as u32).parse(input),
        Endianness::Little => map(le_u32, |v| v as u32).parse(input),
    }
}

/// Parses a constant number according to header's integral flag
pub fn parse_number<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], f64> {
    let parse_bytes = |bytes: &'a [u8]| -> Result<f64, nom::error::Error<&[u8]>> {
        let arr = bytes
            .try_into()
            .map_err(|_| nom::error::Error::new(bytes, ErrorKind::LengthValue))?;
        Ok(if header.integral_flag {
            match header.endianness {
                Endianness::Big => i64::from_be_bytes(arr) as f64,
                Endianness::Little => i64::from_le_bytes(arr) as f64,
            }
        } else {
            match header.endianness {
                Endianness::Big => f64::from_be_bytes(arr),
                Endianness::Little => f64::from_le_bytes(arr),
            }
        })
    };

    map_res(take(header.size_number), parse_bytes).parse(input)
}

/// Parses a constant value from the bytecode
pub fn parse_constant<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Constant> {
    let (input, tag_byte) = u8(input)?;
    match tag_byte {
        0x00 => Ok((input, Constant::Nil)),
        0x01 => map(u8, |v| Constant::Boolean(v != 0)).parse(input),
        0x03 => map(|i| parse_number(i, header), Constant::Number).parse(input),
        0x04 => map(|i| parse_string(i, header), Constant::String).parse(input),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::Tag,
        ))),
    }
}
