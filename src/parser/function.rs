use super::constants::{Constant, Endianness};
use super::header::Header;
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take},
    combinator::{map, map_res},
    error::ErrorKind,
    multi::count,
    number::complete::{be_u32, be_u64, le_u32, le_u64, u8},
};

/// Represents a local variable debug information
#[derive(Debug)]
#[allow(dead_code)]
pub struct LocalVariable {
    varname: String, // Variable name
    startpc: u32,    // First instruction index where the variable is valid
    endpc: u32,      // Last instruction index where the variable is valid
}

/// Represents debug information for a function
#[derive(Debug)]
#[allow(dead_code)]
struct DebugInfo {
    lineinfo: Vec<u32>,         // Line numbers for each instruction
    locals: Vec<LocalVariable>, // Local variable information
    upvalues: Vec<String>,      // Upvalue names
}

/// Represents a Lua function prototype
#[derive(Debug)]
#[allow(dead_code)]
pub struct FunctionPrototype {
    source_name: String,                // Source file name
    line_defined: i32,                  // Line number where the function is defined
    last_line_defined: i32,             // Last line number where the function is defined
    num_upvalues: u8,                   // Number of upvalues
    num_params: u8,                     // Number of parameters
    is_vararg: u8,                      // Whether the function accepts variable arguments
    max_stack_size: u8,                 // Maximum stack size
    code: Vec<u32>,                     // Bytecode instructions
    constants: Vec<Constant>,           // Constants used in the function
    prototypes: Vec<FunctionPrototype>, // Nested function prototypes
    debug_info: DebugInfo,              // Debug information
}

/// Parses a 32-bit integer with specified endianness
fn parse_integer<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], i32> {
    match header.endianness {
        Endianness::Big => be_u32.map(|v| v as i32).parse(input),
        Endianness::Little => le_u32.map(|v| v as i32).parse(input),
    }
}

/// Parses a size_t value according to header specifications
fn parse_size_t<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], u64> {
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
fn parse_string<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], String> {
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
fn parse_instruction<'a>(header: &Header) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], u32> {
    match header.endianness {
        Endianness::Big => be_u32,
        Endianness::Little => le_u32,
    }
}

/// Parses a constant number according to header's integral flag
fn parse_number<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], f64> {
    let mut number_parser = map_res(
        take(header.size_number),
        |bytes: &[u8]| -> Result<f64, nom::error::Error<&[u8]>> {
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
        },
    );

    let (input, num) = number_parser.parse(input)?;
    Ok((input, num))
}

/// Parses a constant value from the bytecode
fn parse_constant<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], Constant> {
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

/// Parse a Lua function prototype
pub fn parse_function<'a>(
    input: &'a [u8],
    header: &Header,
) -> IResult<&'a [u8], FunctionPrototype> {
    let (input, source_name) = parse_string(input, header)?;
    let (input, line_defined) = parse_integer(input, header)?;
    let (input, last_line_defined) = parse_integer(input, header)?;
    let (input, num_upvalues) = u8(input)?;
    let (input, num_params) = u8(input)?;
    let (input, is_vararg) = u8(input)?;
    let (input, max_stack_size) = u8(input)?;

    let (input, code_len) = parse_integer(input, header)?;
    let code_len = usize::try_from(code_len)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
    let (input, code) = count(parse_instruction(header), code_len).parse(input)?;

    let (input, constants_len) = parse_integer(input, header)?;
    let constants_len = usize::try_from(constants_len)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
    let (input, constants) = count(|i| parse_constant(i, header), constants_len).parse(input)?;

    let (input, prototypes_len) = parse_integer(input, header)?;
    let prototypes_len = usize::try_from(prototypes_len)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
    let (input, prototypes) = count(|i| parse_function(i, header), prototypes_len).parse(input)?;

    // Debug information
    let (input, lineinfo_len) = parse_integer(input, header)?;
    let lineinfo_len = usize::try_from(lineinfo_len)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
    let (input, lineinfo) = count(|i| parse_integer(i, header), lineinfo_len).parse(input)?;
    let (input, locals_len) = parse_integer(input, header)?;
    let locals_len = usize::try_from(locals_len)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
    let (input, locals) = count(
        |i| {
            let (input, varname) = parse_string(i, header)?;
            let (input, startpc) = parse_integer(input, header)?;
            let (input, endpc) = parse_integer(input, header)?;
            Ok((
                input,
                LocalVariable {
                    varname,
                    startpc: startpc as u32,
                    endpc: endpc as u32,
                },
            ))
        },
        locals_len,
    )
    .parse(input)?;
    let (input, upvalues_len) = parse_integer(input, header)?;
    let upvalues_len = usize::try_from(upvalues_len)
        .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
    let (input, upvalues) = count(|i| parse_string(i, header), upvalues_len).parse(input)?;
    let debug_info = DebugInfo {
        lineinfo: lineinfo.into_iter().map(|v| v as u32).collect(),
        locals,
        upvalues,
    };

    let proto = FunctionPrototype {
        source_name: source_name.clone(),
        line_defined,
        last_line_defined,
        num_upvalues,
        num_params,
        is_vararg,
        max_stack_size,
        code: code.to_vec(),
        constants: constants,
        prototypes: prototypes,
        debug_info,
    };

    crate::debug_println!("Parsed function prototype: {:#?}", proto);

    Ok((input, proto))
}
