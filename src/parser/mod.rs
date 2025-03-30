pub mod constants;
pub mod debug;
pub mod function;
pub mod header;
pub mod parsers;

pub use function::parse_function;
pub use header::parse_header;

use function::FunctionPrototype;
use header::Header;

/// Main entry point for parsing Lua bytecode
pub fn parse_lua_bytecode(
    input: &[u8],
) -> Result<(Header, FunctionPrototype), nom::Err<nom::error::Error<&[u8]>>> {
    let (input, header) = parse_header(input)?;
    let (input, prototype) = parse_function(input, &header)?;

    // Check for any remaining bytes after parsing
    if !input.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )));
    };

    Ok((header, prototype))
}
