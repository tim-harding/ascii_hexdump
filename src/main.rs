use std::{borrow::Cow, str::Utf8Error};

use clap::Clap;
use nom::{
    branch::alt,
    bytes::complete::{take_till, take_while},
    combinator::{eof, map, map_res},
    error::VerboseError,
    multi::many_till,
};
use thiserror::Error;

#[derive(Debug, Clone, Clap)]
struct Opts {
    #[clap(short, long)]
    input: String,
    #[clap(short, long)]
    output: String,
}

enum Fragment<'a> {
    Ascii(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> Into<Cow<'a, str>> for Fragment<'a> {
    fn into(self) -> Cow<'a, str> {
        match self {
            Fragment::Ascii(s) => Cow::Borrowed(s),
            Fragment::Bytes(bytes) => {
                let mut out = String::with_capacity(bytes.len() * 3);
                for byte in bytes {
                    let s = format!("{:X} ", byte);
                    out.push_str(&s);
                }
                Cow::Owned(out)
            }
        }
    }
}

type IResult<'a, T> = nom::IResult<&'a [u8], T, VerboseError<&'a [u8]>>;

#[derive(Debug, Error)]
enum AhError {
    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Parse error")]
    Parse,
}

fn main() -> Result<(), AhError> {
    let opts = Opts::parse();
    let input = std::fs::read(opts.input)?;
    let parts = parse(&input)?;
    let output = stringify(parts);
    std::fs::write(opts.output, output)?;
    Ok(())
}

fn stringify(fragments: Vec<Fragment>) -> String {
    todo!()
}

fn parse(b: &[u8]) -> Result<Vec<Fragment>, AhError> {
    let (_i, parts) = match fragments(b) {
        Ok(parts) => Ok(parts),
        Err(err) => {
            use nom::Err::*;
            match err {
                Incomplete(needed) => {
                    println!("Needed more bytes: {:?}", needed);
                }
                Error(err) | Failure(err) => {
                    println!("Error while parsing\n{:?}", err);
                }
            }
            Err(AhError::Parse)
        }
    }?;
    Ok(parts)
}

fn fragments(b: &[u8]) -> IResult<Vec<Fragment>> {
    map(many_till(fragment, eof), |(fragment, _)| fragment)(b)
}

fn fragment(b: &[u8]) -> IResult<Fragment> {
    alt((ascii, bytes))(b)
}

fn bytes(b: &[u8]) -> IResult<Fragment> {
    map(take_till(is_ascii), Fragment::Bytes)(b)
}

fn ascii(b: &[u8]) -> IResult<Fragment> {
    map_res(take_while(is_ascii), ascii_to_fragment)(b)
}

fn ascii_to_fragment(b: &[u8]) -> Result<Fragment, Utf8Error> {
    let s = std::str::from_utf8(b)?;
    Ok(Fragment::Ascii(s))
}

fn is_ascii(b: u8) -> bool {
    let c = b as char;
    c.is_ascii()
}
