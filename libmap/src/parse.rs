use lyon_geom::{Angle, Point, Vector};
use lyon_path::ArcFlags;
use nom::{
    branch::alt,
    bytes::{
        complete::is_not,
        complete::{tag, take_while_m_n},
    },
    character::complete::{one_of, space0, space1},
    combinator::{map, map_res},
    multi::{many1, separated_list1},
    number::complete::float,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    RelVerticalLineTo(f32),
    VerticalLineTo(f32),
    HorizontalLineTo(f32),
    RelHorizontalLineTo(f32),
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
    CubicBezierTo {
        ctrl1: Point<f32>,
        ctrl2: Point<f32>,
        to: Point<f32>,
    },
    RelCubicBezierTo {
        ctrl1: Vector<f32>,
        ctrl2: Vector<f32>,
        to: Vector<f32>,
    },
    Close,
}

pub fn path_to_operations(svg: &str) -> IResult<&str, Vec<Vec<Operation>>> {
    many1(parse_op)(svg)
}

pub(crate) fn parse_op(s: &str) -> nom::IResult<&str, Vec<Operation>> {
    let (s, op) = one_of("vVhHmMlLzZqQaAcC")(s)?;
    let (s, _) = space0(s)?;

    let (s, operation) = match op {
        'v' => separated_list1(space1, map(readf32, Operation::RelVerticalLineTo))(s)?,
        'V' => separated_list1(space1, map(readf32, Operation::VerticalLineTo))(s)?,
        'h' => many1(map(readf32, Operation::RelHorizontalLineTo))(s)?,
        'H' => many1(map(readf32, Operation::HorizontalLineTo))(s)?,
        'm' => separated_list1(space1, map(read_vector, Operation::RelMoveTo))(s)?,

        'M' => separated_list1(space1, map(read_point, Operation::MoveTo))(s)?,
        'l' => separated_list1(space1, map(read_vector, Operation::RelLineTo))(s)?,
        'L' => separated_list1(space1, map(read_point, Operation::LineTo))(s)?,
        'Z' | 'z' => (s, vec![Operation::Close]),
        'q' => separated_list1(
            space1,
            map(
                separated_pair(read_vector, space1, read_vector),
                |(ctrl, to)| Operation::RelQuadBezierTo { ctrl, to },
            ),
        )(s)?,
        'Q' => separated_list1(
            space1,
            map(
                separated_pair(read_point, space1, read_point),
                |(ctrl, to)| Operation::QuadBezierTo { ctrl, to },
            ),
        )(s)?,
        'a' => separated_list1(
            space1,
            map(
                tuple((
                    read_vector,
                    space1,
                    read_angle,
                    space1,
                    read_arcflags,
                    space1,
                    read_vector,
                )),
                |(radii, _, x_rotation, _, flags, _, to)| Operation::RelArcTo {
                    radii,
                    x_rotation,
                    flags,
                    to,
                },
            ),
        )(s)?,
        'A' => separated_list1(
            space1,
            map(
                tuple((
                    read_vector,
                    space1,
                    read_angle,
                    space1,
                    read_arcflags,
                    space1,
                    read_point,
                )),
                |(radii, _, x_rotation, _, flags, _, to)| Operation::ArcTo {
                    radii,
                    x_rotation,
                    flags,
                    to,
                },
            ),
        )(s)?,
        'C' => separated_list1(
            space1,
            map(
                tuple((read_point, space1, read_point, space1, read_point)),
                |(ctrl1, _, ctrl2, _, to)| Operation::CubicBezierTo { ctrl1, ctrl2, to },
            ),
        )(s)?,
        'c' => separated_list1(
            space1,
            map(
                tuple((read_vector, space1, read_vector, space1, read_vector)),
                |(ctrl1, _, ctrl2, _, to)| Operation::RelCubicBezierTo { ctrl1, ctrl2, to },
            ),
        )(s)?,
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
