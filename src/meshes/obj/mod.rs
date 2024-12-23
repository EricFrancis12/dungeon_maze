pub mod macros;

use crate::error::Error;
use std::{fmt, str::FromStr};

enum ObjToken {
    O,
    V,
    Vn,
    Vt,
    S,
    F,
}

impl fmt::Display for ObjToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token_str = match self {
            ObjToken::O => "o",
            ObjToken::V => "v",
            ObjToken::Vn => "vn",
            ObjToken::Vt => "vt",
            ObjToken::S => "s",
            ObjToken::F => "f",
        };
        write!(f, "{}", token_str)
    }
}

impl FromStr for ObjToken {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "o" => Ok(ObjToken::O),
            "v" => Ok(ObjToken::V),
            "vn" => Ok(ObjToken::Vn),
            "vt" => Ok(ObjToken::Vt),
            "s" => Ok(ObjToken::S),
            "f" => Ok(ObjToken::F),
            _ => Err(Error::Parsing(
                "error parsing ObjToken from string".to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_arr(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.9}, {:.9}, {:.9}]", self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Normal {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Normal {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_arr(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.9}, {:.9}, {:.9}]", self.x, self.y, self.z)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UVTexture {
    pub h: f32,
    pub v: f32,
}

impl UVTexture {
    fn new(h: f32, v: f32) -> Self {
        Self { h, v }
    }

    pub fn to_arr(&self) -> [f32; 2] {
        [self.h, self.v]
    }
}

impl fmt::Display for UVTexture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:.9}, {:.9}]", self.h, self.v)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FaceDefinition {
    pub vertex_index: usize,
    pub normal_index: usize,
    pub uv_texture_index: usize,
}

impl FaceDefinition {
    fn new(vertex_index: usize, normal_index: usize, uv_texture_index: usize) -> Self {
        Self {
            vertex_index,
            normal_index,
            uv_texture_index,
        }
    }
}

impl FromStr for FaceDefinition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("/").collect();
        if let [v_str, vt_str, vn_str] = &parts[0..] {
            // Subtracting 1 is necessary because .obj indexing starts at 1:
            let v = v_str.parse::<usize>()? - 1;
            let vt = vt_str.parse::<usize>()? - 1;
            let vn = vn_str.parse::<usize>()? - 1;

            return Ok(Self::new(v, vn, vt));
        }

        Err(Error::Parsing(
            "error parsing FaceDefinition from string".to_owned(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Face {
    pub face_defs: Vec<FaceDefinition>,
}

impl Face {
    fn new(face_defs: Vec<FaceDefinition>) -> Self {
        Self { face_defs }
    }
}

impl fmt::Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self
            .face_defs
            .iter()
            .map(|fd| {
                format!(
                    "{},{},{}",
                    fd.vertex_index, fd.uv_texture_index, fd.normal_index,
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Obj3D {
    pub vertecies: Vec<Vertex>,
    pub normals: Vec<Normal>,
    pub uv_textures: Vec<UVTexture>,
    pub smoothing: u8,
    pub faces: Vec<Face>,
}

impl Obj3D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_string(s: impl Into<String>) -> Result<Vec<Self>, Error> {
        let content = s.into();
        let lines: Vec<&str> = content.split("\n").collect();

        let mut objs: Vec<Self> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let n = i + 1;
            let tokens: Vec<&str> = line.split(" ").map(|s| s.trim()).collect();

            if let Some(s) = tokens.get(0) {
                if let Ok(obj_token) = ObjToken::from_str(s) {
                    match obj_token {
                        ObjToken::O => {
                            objs.push(Self::new());
                        }
                        ObjToken::V => {
                            let xyz = &tokens[1..];
                            if let [x_str, y_str, z_str] = xyz {
                                if let Some(obj) = objs.last_mut() {
                                    let x = x_str.parse::<f32>()?;
                                    let y = y_str.parse::<f32>()?;
                                    let z = z_str.parse::<f32>()?;

                                    obj.vertecies.push(Vertex::new(x, y, z));
                                } else {
                                    return Err(Error::Obj(format!(
                                        "line {}: expected object declaration",
                                        n
                                    )));
                                }
                            } else {
                                return Err(Error::Obj(format!(
                                    "line {}: expected 3 values, but got {}",
                                    n,
                                    xyz.len()
                                )));
                            };
                        }
                        ObjToken::Vn => {
                            let xyz = &tokens[1..];
                            if let [x_str, y_str, z_str] = xyz {
                                if let Some(obj) = objs.last_mut() {
                                    let x = x_str.parse::<f32>()?;
                                    let y = y_str.parse::<f32>()?;
                                    let z = z_str.parse::<f32>()?;

                                    obj.normals.push(Normal::new(x, y, z));
                                } else {
                                    return Err(Error::Obj(format!(
                                        "line {}: expected object declaration",
                                        n
                                    )));
                                }
                            } else {
                                return Err(Error::Obj(format!(
                                    "line {}: expected 3 values, but got {}",
                                    n,
                                    xyz.len()
                                )));
                            };
                        }
                        ObjToken::Vt => {
                            let vh = &tokens[1..];
                            if let [v_str, h_str] = vh {
                                if let Some(obj) = objs.last_mut() {
                                    let v = v_str.parse::<f32>()?;
                                    let h = h_str.parse::<f32>()?;

                                    obj.uv_textures.push(UVTexture::new(v, h));
                                } else {
                                    return Err(Error::Obj(format!(
                                        "line {}: expected object declaration",
                                        n
                                    )));
                                }
                            } else {
                                return Err(Error::Obj(format!(
                                    "line {}: expected 2 values, but got {}",
                                    n,
                                    vh.len()
                                )));
                            };
                        }
                        ObjToken::S => {
                            let smoothing_slice = &tokens[1..];
                            if let [smoothing] = smoothing_slice {
                                if let Some(obj) = objs.last_mut() {
                                    // TODO: handle possibility where there could be
                                    // multiple s tokens in a single object
                                    if smoothing.to_lowercase() == "off" {
                                        obj.smoothing = 0;
                                    } else {
                                        obj.smoothing = smoothing.parse::<u8>()?;
                                    }
                                } else {
                                    return Err(Error::Obj(format!(
                                        "line {}: expected object declaration",
                                        n
                                    )));
                                }
                            } else {
                                return Err(Error::Obj(format!(
                                    "line {}: expected 1 value, but got {}",
                                    n,
                                    smoothing_slice.len()
                                )));
                            };
                        }
                        ObjToken::F => {
                            if let Some(obj) = objs.last_mut() {
                                let faces_slice = &tokens[1..];

                                let mut face_defs = Vec::new();
                                for face_str in faces_slice {
                                    face_defs.push(FaceDefinition::from_str(face_str)?);
                                }

                                obj.faces.push(Face::new(face_defs));
                            } else {
                                return Err(Error::Obj(format!(
                                    "line {}: expected object declaration",
                                    n
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(objs)
    }

    pub fn parse_string_single(s: impl Into<String>) -> Result<Self, Error> {
        let content = s.into();
        match Self::parse_string(content) {
            Ok(objs) => match objs.get(0) {
                Some(obj) => Ok(obj.clone()),
                None => Err(Error::Obj("no objects found".to_owned())),
            },
            Err(err) => Err(err),
        }
    }
}
