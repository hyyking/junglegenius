use lyon_geom::{Angle, Point, Vector};
use lyon_path::ArcFlags;
use nom::{
    bytes::{complete::is_not, complete::tag},
    character::complete::{one_of, space0},
    multi::many1,
    number::complete::float,
    sequence::{delimited, tuple},
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

#[derive(Debug)]
pub enum Operation {
    M(Point<f32>), // moveto
    L(Point<f32>), // lineto
    Q {
        ctrl: Point<f32>,
        to: Point<f32>,
    }, // quadratic
    A {
        radii: Vector<f32>,
        x_rotation: Angle<f32>,
        flags: ArcFlags,
        to: Point<f32>,
    },
    Close,
}

pub fn path_to_operations(svg: &str) -> Vec<Operation> {
    many1(parse_op)(svg).unwrap().1
}

pub(crate) fn parse_op(s: &str) -> nom::IResult<&str, Operation> {
    let (s, op) = one_of("MLHVCSQTAZ")(s)?;
    let (s, _) = space0(s)?;

    let (s, operation) = match op {
        'M' => {
            let (s, point) = read_point(s)?;
            (s, Operation::M(point))
        }
        'L' => {
            let (s, point) = read_point(s)?;
            (s, Operation::L(point))
        }
        'Z' => (s, Operation::Close),
        'Q' => {
            let (s, ctrl) = read_point(s)?;
            let (s, _) = space0(s)?;
            let (s, to) = read_point(s)?;
            (s, Operation::Q { ctrl, to })
        }
        'A' => {
            let (s, radii) = read_vector(s)?;
            let (s, _) = space0(s)?;
            let (s, x_rotation) = read_angle(s)?;
            let (s, _) = space0(s)?;
            let (s, flags) = read_arcflags(s)?;
            let (s, _) = space0(s)?;
            let (s, to) = read_point(s)?;
            (
                s,
                Operation::A {
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

fn read_vector(s: &str) -> IResult<&str, Vector<f32>> {
    let (s, x) = float(s)?;
    let (s, _) = space0(s)?;
    let (s, y) = float(s)?;
    Ok((s, Vector::new(x, y)))
}

fn read_angle(s: &str) -> IResult<&str, Angle<f32>> {
    let (s, radians) = float(s)?;
    Ok((s, Angle { radians }))
}

fn read_arcflags(s: &str) -> IResult<&str, ArcFlags> {
    let (s, large) = nom::character::complete::u8(s)?;
    let (s, _) = space0(s)?;
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
    let (s, x) = float(s)?;
    let (s, _) = space0(s)?;
    let (s, y) = float(s)?;
    Ok((s, Point::new(x, y)))
}
