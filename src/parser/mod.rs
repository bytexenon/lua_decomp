pub mod constants;
pub mod function;
pub mod header;

pub use function::parse_function;
pub use header::parse_header;

use function::FunctionPrototype;
use header::Header;
use nom::IResult;

/// Main entry point for parsing Lua bytecode
pub fn parse_lua_bytecode(input: &[u8]) -> IResult<&[u8], (Header, FunctionPrototype)> {
    let (input, header) = parse_header(input)?;
    let (input, prototype) = parse_function(input, &header)?;

    // Check for any remaining bytes after parsing
    if !input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )));
    };

    Ok((input, (header, prototype)))
}
