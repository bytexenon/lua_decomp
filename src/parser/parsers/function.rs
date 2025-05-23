use super::super::bytecode::FunctionPrototype;
use super::super::bytecode::Header;
use super::super::bytecode::{DebugInfo, LocalVariable};
use super::parsers::{parse_constant, parse_instruction, parse_integer, parse_string};
use log::debug;
use nom::{error::ErrorKind, multi::count, number::complete::u8, IResult, Parser};

/// Parsing functions module
mod parsers {
    use super::*;

    pub fn parse_section<'a, T, F>(
        input: &'a [u8],
        header: &Header,
        parser: F,
    ) -> IResult<&'a [u8], Vec<T>>
    where
        F: Fn(&'a [u8]) -> IResult<&'a [u8], T>,
    {
        let (input, len) = parse_integer(input, header)?;
        let len = usize::try_from(len)
            .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::TooLarge)))?;
        count(parser, len).parse(input)
    }

    pub fn parse_local_variable<'a>(
        input: &'a [u8],
        header: &Header,
    ) -> IResult<&'a [u8], LocalVariable> {
        let (input, varname) = parse_string(input, header)?;
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
    }

    pub fn parse_debug_info<'a>(input: &'a [u8], header: &Header) -> IResult<&'a [u8], DebugInfo> {
        let (input, lineinfo) = parse_section(input, header, |i| parse_integer(i, header))?;
        let (input, locals) = parse_section(input, header, |i| parse_local_variable(i, header))?;
        let (input, upvalues) = parse_section(input, header, |i| parse_string(i, header))?;

        let debug_info = DebugInfo {
            lineinfo: lineinfo.into_iter().map(|v| v as u32).collect(),
            locals,
            upvalues,
        };

        Ok((input, debug_info))
    }
}

use parsers::*;

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

    let (input, code) = parse_section(input, header, |i| parse_instruction(i, header))?;
    let (input, constants) = parse_section(input, header, |i| parse_constant(i, header))?;
    let (input, prototypes) = parse_section(input, header, |i| parse_function(i, header))?;
    let (input, debug_info) = parse_debug_info(input, header)?;

    let proto = FunctionPrototype {
        source_name,
        line_defined,
        last_line_defined,
        num_upvalues,
        num_params,
        is_vararg,
        max_stack_size,
        code,
        constants,
        prototypes,
        debug_info,
    };

    debug!("Parsed function prototype: {:#?}", proto);

    Ok((input, proto))
}
