#[allow(dead_code)]
#[derive(Debug)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Endianness {
    Big,
    Little,
}
