use clap::Clap;
use nom::{
    branch::alt,
    bytes::complete::{take_till1, take_while1},
    combinator::{eof, map, map_res},
    error::{context, VerboseError},
    multi::many_till,
};
use std::{borrow::Cow, fs, str::Utf8Error};
use thiserror::Error;

type IResult<'a, T> = nom::IResult<&'a [u8], T, VerboseError<&'a [u8]>>;

#[derive(Debug, Clone, Clap)]
struct Opts {
    #[clap(short, long)]
    input: String,
    #[clap(short, long)]
    output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Fragment<'a> {
    Ascii(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> From<Fragment<'a>> for Cow<'a, str> {
    fn from(fragment: Fragment<'a>) -> Self {
        match fragment {
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

#[derive(Debug, Error)]
enum AhError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Parse error")]
    Parse,
}

fn main() -> Result<(), AhError> {
    let opts = Opts::parse();
    let input = fs::read(opts.input)?;
    let output = parse(&input)?;
    fs::write(opts.output, output)?;
    Ok(())
}

fn parse(b: &[u8]) -> Result<String, AhError> {
    let (_i, s) = match combine(b) {
        Ok(parts) => Ok(parts),
        Err(err) => {
            use nom::Err::*;
            match err {
                Incomplete(needed) => {
                    eprintln!("Needed more bytes: {:?}", needed);
                }
                Error(err) | Failure(err) => {
                    eprintln!("Errors while parsing:");
                    for err in err.errors {
                        eprintln!("{}, {:?}", err.0.len(), err.1);
                    }
                }
            }
            Err(AhError::Parse)
        }
    }?;
    Ok(s)
}

fn combine(b: &[u8]) -> IResult<String> {
    context(
        "combine",
        map(fragments, |fragments| {
            let mut s = String::new();
            for fragment in fragments {
                let cow: Cow<_> = fragment.into();
                s.push_str(&cow);
            }
            s
        }),
    )(b)
}

fn fragments(b: &[u8]) -> IResult<Vec<Fragment>> {
    context(
        "fragments",
        // Todo: It'd be cooler to fold the results in this step,
        // but many_till seems to be the only combinator that
        // doesn't break when it reaches EOF.
        map(many_till(fragment, eof), |(fragment, _)| fragment),
    )(b)
}

fn fragment(b: &[u8]) -> IResult<Fragment> {
    context("fragment", alt((ascii, bytes)))(b)
}

fn bytes(b: &[u8]) -> IResult<Fragment> {
    context("bytes", map(take_till1(is_ascii), Fragment::Bytes))(b)
}

fn ascii(b: &[u8]) -> IResult<Fragment> {
    context("ascii", map_res(take_while1(is_ascii), ascii_to_fragment))(b)
}

fn ascii_to_fragment(b: &[u8]) -> Result<Fragment, Utf8Error> {
    let s = std::str::from_utf8(b)?;
    Ok(Fragment::Ascii(s))
}

fn is_ascii(b: u8) -> bool {
    let c = b as char;
    c.is_ascii()
}
