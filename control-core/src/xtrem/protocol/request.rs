use std::borrow::Cow;

use crate::xtrem::protocol::DataAddress;

#[derive(Debug, Clone)]
pub struct Request<'a> {
    address:  DataAddress,
    function: RequestPayload<'a>
}

#[derive(Debug, Clone)]
pub enum RequestPayload<'a> {
    Read,
    Write(Cow<'a, [u8]>),
    Execute,
}

#[derive(Debug, Clone, Copy)]
pub enum Response<'a> {
    Read(&'a [u8]),
    Write(u16),
    Execute(u16),
}