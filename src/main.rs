pub mod macros;

mod parser;
use parser::parse_lua_bytecode;

fn main() {
    let bytecode = include_bytes!("test.luac");
    match parse_lua_bytecode(bytecode) {
        Ok((_remaining, (header, prototype))) => {
            println!("Header: {:#?}", header);
            println!("Function Prototype: {:#?}", prototype);
        }
        Err(err) => {
            eprintln!("Error parsing Lua bytecode: {:?}", err);
        }
    }
}
