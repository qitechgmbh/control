#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RequestFunction {
    Read = b'R',
    Write = b'W',
    Execute = b'E',
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ResponseFunction {
    Read = b'r',
    Write = b'w',
    Execute = b'e',
}

#[derive(Debug, Clone, Copy)]
pub enum Function {
    Request(RequestFunction),
    Response(ResponseFunction),
}

impl Function {
    pub fn from_u8(value: u8) -> Option<Self> {
        use Function::*;

        match value {
            b'R' => Some(Request(RequestFunction::Read)),
            b'W' => Some(Request(RequestFunction::Write)),
            b'E' => Some(Request(RequestFunction::Execute)),

            b'r' => Some(Response(ResponseFunction::Read)),
            b'w' => Some(Response(ResponseFunction::Write)),
            b'e' => Some(Response(ResponseFunction::Execute)),

            _ => None,
        }
    }
}