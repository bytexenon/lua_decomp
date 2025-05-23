pub mod bytecode;
pub mod parsers;

use bytecode::{FunctionPrototype, Header};

pub use parsers::function::parse_function;
pub use parsers::header::parse_header;

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
