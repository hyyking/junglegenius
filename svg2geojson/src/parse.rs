use lyon_geom::{Angle, Point, Vector};
use lyon_path::ArcFlags;
use nom::{
    branch::alt,
    bytes::{
        complete::is_not,
        complete::{tag, take_while_m_n},
    },
    character::{
        self,
        complete::{one_of, space0, space1},
    },
    combinator::{map, map_res},
    multi::many1,
    number::complete::float,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

#[derive(Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn parse_rgb(s: &str) -> IResult<&str, RGB> {
    let (s, _) = tag("rgb")(s)?;
    let (s, content) = delimited(
        nom::character::complete::char('('),
        is_not(")"),
        nom::character::complete::char(')'),
    )(s)?;
    let (_, (r, _, g, _, b)) = tuple((
        nom::character::complete::u8,
        nom::character::complete::char(','),
        nom::character::complete::u8,
        nom::character::complete::char(','),
        nom::character::complete::u8,
    ))(content)?;
    Ok((s, RGB { r, g, b }))
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub fn parse_hex_rgb(input: &str) -> IResult<&str, RGB> {
    let (input, _) = tag("#")(input)?;
    let (input, (r, g, b)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;
    Ok((input, RGB { r, g, b }))
}

#[derive(Debug)]
pub enum Operation {
    RelMoveTo(Vector<f32>),
    MoveTo(Point<f32>), // moveto
    RelLineTo(Vector<f32>),
    LineTo(Point<f32>), // lineto
    RelQuadBezierTo {
        ctrl: Vector<f32>,
        to: Vector<f32>,
    },
    QuadBezierTo {
        ctrl: Point<f32>,
        to: Point<f32>,
    }, // quadratic
    RelArcTo {
        radii: Vector<f32>,
        x_rotation: Angle<f32>,
        flags: ArcFlags,
        to: Vector<f32>,
    },
    ArcTo {
        radii: Vector<f32>,
        x_rotation: Angle<f32>,
        flags: ArcFlags,
        to: Point<f32>,
    },
    Close,
}

pub fn path_to_operations(svg: &str) -> IResult<&str, Vec<Operation>> {
    many1(parse_op)(svg)
}

pub(crate) fn parse_op(s: &str) -> nom::IResult<&str, Operation> {
    let (s, op) = one_of("MLHVCSQTAZmqla")(s)?;
    let (s, _) = space0(s)?;

    let (s, operation) = match op {
        'm' => {
            let (s, mv) = read_vector(s)?;
            (s, Operation::RelMoveTo(mv))
        }

        'M' => {
            let (s, point) = read_point(s)?;
            (s, Operation::MoveTo(point))
        }
        'l' => {
            let (s, mv) = read_vector(s)?;
            (s, Operation::RelLineTo(mv))
        }
        'L' => {
            let (s, point) = read_point(s)?;
            (s, Operation::LineTo(point))
        }
        'Z' => (s, Operation::Close),
        'q' => {
            let (s, ctrl) = read_vector(s)?;
            let (s, _) = space1(s)?;
            let (s, to) = read_vector(s)?;
            (s, Operation::RelQuadBezierTo { ctrl, to })
        }
        'Q' => {
            let (s, ctrl) = read_point(s)?;
            let (s, _) = space1(s)?;
            let (s, to) = read_point(s)?;
            (s, Operation::QuadBezierTo { ctrl, to })
        }
        'a' => {
            let (s, radii) = read_vector(s)?;
            let (s, _) = space1(s)?;
            let (s, x_rotation) = read_angle(s)?;
            let (s, _) = space1(s)?;
            let (s, flags) = read_arcflags(s)?;
            let (s, _) = space1(s)?;
            let (s, to) = read_vector(s)?;
            (
                s,
                Operation::RelArcTo {
                    radii,
                    x_rotation,
                    flags,
                    to,
                },
            )
        }
        'A' => {
            let (s, radii) = read_vector(s)?;
            let (s, _) = space1(s)?;
            let (s, x_rotation) = read_angle(s)?;
            let (s, _) = space1(s)?;
            let (s, flags) = read_arcflags(s)?;
            let (s, _) = space1(s)?;
            let (s, to) = read_point(s)?;
            (
                s,
                Operation::ArcTo {
                    radii,
                    x_rotation,
                    flags,
                    to,
                },
            )
        }
        a => todo!("'{a}' not implemented, rest: {s}"),
    };
    let (s, _) = space0(s)?;
    Ok((s, operation))
}

fn readf32(s: &str) -> IResult<&str, f32> {
    alt((float, map(nom::character::complete::u32, |n| n as f32)))(s)
}

fn read_vector(s: &str) -> IResult<&str, Vector<f32>> {
    let (s, (x, y)) = separated_pair(readf32, alt((tag(","), space0)), readf32)(s)?;
    Ok((s, Vector::new(x, y)))
}

fn read_angle(s: &str) -> IResult<&str, Angle<f32>> {
    let (s, radians) = readf32(s)?;
    Ok((s, Angle { radians }))
}

fn read_arcflags(s: &str) -> IResult<&str, ArcFlags> {
    let (s, large) = nom::character::complete::u8(s)?;
    let (s, _) = alt((tag(","), space0))(s)?;
    let (s, sweep) = nom::character::complete::u8(s)?;
    Ok((
        s,
        ArcFlags {
            large_arc: large == 1,
            sweep: sweep == 1,
        },
    ))
}

fn read_point(s: &str) -> IResult<&str, Point<f32>> {
    let (s, (x, y)) = separated_pair(readf32, alt((tag(","), space0)), readf32)(s)?;
    Ok((s, Point::new(x, y)))
}
