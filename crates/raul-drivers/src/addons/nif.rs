// SPDX-License-Identifier: GPL-3.0-only
//! A glow mesh written from numbers: one low-poly sphere for Fallout 3 and New Vegas (NIF 20.2.0.7, user version 11,
//! user version 2 34), drawn unlit and additive. `BSShaderNoLightingProperty` with its view-angle falloff makes the
//! sphere look like one soft round halo from every side: full where the surface faces the camera, nothing at the
//! outline. Only front faces are drawn (no stencil property), so the halo never doubles. The texture is named by its
//! path in the game's own data and sampled at its centre (every UV is 0.5, 0.5); nothing of it is read or copied.

use serde::Deserialize;

/// The numbers of a glow sphere, as the add-on table gives them.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlowSphere {
    /// The radius in mesh units, before the placed reference's scale.
    pub radius: f64,
    /// The vertex alpha: how strong the halo is.
    pub alpha: f64,
    /// The vertex colour, red, green and blue from 0 to 1.
    pub rgb: [f64; 3],
    /// Latitude bands from pole to pole (at least 2).
    pub rings: u16,
    /// Points around each latitude row (at least 3).
    pub segments: u16,
    /// The texture's path in the game's data, for example `textures\effects\FXSoftGlowSpot01.dds`.
    pub texture: String,
}

/// The name of the root node and the shape.
const NAME: &str = "RaulSmallGlow";

/// The falloff: full opacity facing the camera, none from 75 degrees off.
const FALLOFF_END_DEGREES: f64 = 75.0;

/// The shader flags of the game's own soft glow effects: vertex colours, vertex alpha, the falloff, z test.
const SHADER_FLAGS_1: u32 = 0xa200_0148;

/// The unit normals and outward-wound triangles of a UV sphere with a vertex at each pole; a vertex is its normal
/// times the radius.
///
/// # Panics
///
/// When the sphere has more than 65 535 vertices, which the format cannot hold.
#[must_use]
pub fn sphere(rings: u16, segments: u16) -> (Vec<[f64; 3]>, Vec<[u16; 3]>) {
    let (rings, segments) = (rings.max(2), segments.max(3));
    let mut vertices = vec![[0.0, 0.0, 1.0]];
    for i in 1..rings {
        let theta = std::f64::consts::PI * f64::from(i) / f64::from(rings);
        for j in 0..segments {
            let phi = 2.0 * std::f64::consts::PI * f64::from(j) / f64::from(segments);
            vertices.push([
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            ]);
        }
    }
    vertices.push([0.0, 0.0, -1.0]);
    let last = u16::try_from(vertices.len() - 1).expect("a small sphere");
    let row = |i: u16, j: u16| 1 + (i - 1) * segments + (j % segments);
    let mut triangles = Vec::new();
    for j in 0..segments {
        triangles.push([0, row(1, j), row(1, j + 1)]);
    }
    for i in 1..rings - 1 {
        for j in 0..segments {
            let (p, q, r, s) = (row(i, j), row(i, j + 1), row(i + 1, j), row(i + 1, j + 1));
            triangles.push([p, r, q]);
            triangles.push([q, r, s]);
        }
    }
    for j in 0..segments {
        triangles.push([last, row(rings - 1, j + 1), row(rings - 1, j)]);
    }
    (vertices, triangles)
}

/// The points of `normals` on a sphere of `radius`.
#[must_use]
pub fn scaled(normals: &[[f64; 3]], radius: f64) -> Vec<[f64; 3]> {
    normals
        .iter()
        .map(|n| [radius * n[0], radius * n[1], radius * n[2]])
        .collect()
}

/// Little-endian writer of the NIF's fields.
#[derive(Default)]
struct Out(Vec<u8>);

impl Out {
    fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    fn u16(&mut self, v: u16) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    fn i32(&mut self, v: i32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    #[allow(clippy::cast_possible_truncation)] // the file stores single precision
    fn f(&mut self, v: f64) -> &mut Self {
        self.0.extend_from_slice(&(v as f32).to_le_bytes());
        self
    }
    fn fs(&mut self, vs: &[f64]) -> &mut Self {
        for v in vs {
            self.f(*v);
        }
        self
    }
    fn bytes(&mut self, b: &[u8]) -> &mut Self {
        self.0.extend_from_slice(b);
        self
    }
    fn len32(&mut self, n: usize) -> &mut Self {
        self.u32(u32::try_from(n).expect("a small mesh"))
    }
    /// `NiObjectNET`: the name (an index into the string table, or -1), extra data references, no controller.
    fn net(&mut self, name: i32, extra: &[i32]) -> &mut Self {
        self.i32(name).len32(extra.len());
        for e in extra {
            self.i32(*e);
        }
        self.i32(-1)
    }
    /// `NiAVObject`: flags, identity transform, property references, no collision object.
    fn av(&mut self, name: i32, extra: &[i32], properties: &[i32]) -> &mut Self {
        self.net(name, extra).u32(0x8_000e);
        self.fs(&[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0,
        ]);
        self.len32(properties.len());
        for p in properties {
            self.i32(*p);
        }
        self.i32(-1)
    }
}

/// The NIF file of `glow`; `creator` is written into the header's export information.
///
/// # Panics
///
/// Never for the numbers of the add-on table: a sphere of more than 65 535 vertices does not fit the format.
#[must_use]
#[allow(clippy::too_many_lines)] // one block after the other, in file order
pub fn glow_mesh(glow: &GlowSphere, creator: &str) -> Vec<u8> {
    let (normals, triangles) = sphere(glow.rings, glow.segments);
    let vertices = scaled(&normals, glow.radius);
    let strings = [
        NAME.to_owned(),
        "BSX".to_owned(),
        format!("{NAME}:0"),
        glow.texture.clone(),
    ];
    let mut blocks: Vec<(&str, Vec<u8>)> = Vec::new();
    // 0 root, 1 BSX, 2 shape, 3 alpha, 4 material, 5 shader, 6 data
    let mut b = Out::default();
    b.av(0, &[1], &[]).u32(1).i32(2).u32(0);
    blocks.push(("BSFadeNode", b.0));
    let mut b = Out::default();
    b.i32(1).u32(0);
    blocks.push(("BSXFlags", b.0));
    let mut b = Out::default();
    b.av(2, &[], &[3, 4, 5]).i32(6).i32(-1).u32(0).i32(0).u8(0);
    blocks.push(("NiTriShape", b.0));
    // Blending on, source SRC_ALPHA, destination ONE: additive.
    let mut b = Out::default();
    b.net(-1, &[]).u16(0x100d).u8(0);
    blocks.push(("NiAlphaProperty", b.0));
    // Specular 0, emissive 1, glossiness 10, alpha 1, emissive multiplier 1.
    let mut b = Out::default();
    b.net(-1, &[])
        .fs(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 10.0, 1.0, 1.0]);
    blocks.push(("NiMaterialProperty", b.0));
    let mut b = Out::default();
    b.net(-1, &[])
        .u16(0x1)
        .u32(33)
        .u32(SHADER_FLAGS_1)
        .u32(0)
        .f(1.0)
        .u32(3)
        .len32(glow.texture.len())
        .bytes(glow.texture.as_bytes())
        .fs(&[1.0, FALLOFF_END_DEGREES.to_radians().cos(), 1.0, 0.0]);
    blocks.push(("BSShaderNoLightingProperty", b.0));
    let nv = u16::try_from(vertices.len()).expect("a small sphere");
    let bound = vertices
        .iter()
        .map(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
        .fold(0.0, f64::max);
    let mut b = Out::default();
    b.i32(0).u16(nv).u8(0).u8(0).u8(1);
    for v in &vertices {
        b.fs(v);
    }
    b.u16(0x0001).u8(1);
    for n in &normals {
        b.fs(n);
    }
    b.fs(&[0.0, 0.0, 0.0, bound]).u8(1);
    for _ in &vertices {
        b.fs(&[glow.rgb[0], glow.rgb[1], glow.rgb[2], glow.alpha]);
    }
    for _ in &vertices {
        b.fs(&[0.5, 0.5]);
    }
    let nt = u16::try_from(triangles.len()).expect("a small sphere");
    b.u16(0x4000).i32(-1).u16(nt).u32(3 * u32::from(nt)).u8(1);
    for t in &triangles {
        b.u16(t[0]).u16(t[1]).u16(t[2]);
    }
    b.u16(0);
    blocks.push(("NiTriShapeData", b.0));

    let mut types: Vec<&str> = Vec::new();
    for (t, _) in &blocks {
        if !types.contains(t) {
            types.push(t);
        }
    }
    let mut h = Out::default();
    h.bytes(b"Gamebryo File Format, Version 20.2.0.7\n")
        .u32(0x1402_0007)
        .u8(1)
        .u32(11)
        .len32(blocks.len())
        .u32(34);
    for s in ["raul", creator, ""] {
        let n = u8::try_from(s.len() + 1).expect("a short export string");
        h.u8(n).bytes(s.as_bytes()).u8(0);
    }
    h.u16(u16::try_from(types.len()).expect("few block types"));
    for t in &types {
        h.len32(t.len()).bytes(t.as_bytes());
    }
    for (t, _) in &blocks {
        let index = types.iter().position(|x| x == t).expect("listed above");
        h.u16(u16::try_from(index).expect("few block types"));
    }
    for (_, data) in &blocks {
        h.len32(data.len());
    }
    h.len32(strings.len())
        .len32(strings.iter().map(String::len).max().unwrap_or(0));
    for s in &strings {
        h.len32(s.len()).bytes(s.as_bytes());
    }
    h.u32(0);
    for (_, data) in &blocks {
        h.bytes(data);
    }
    h.u32(1).i32(0);
    h.0
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn shipped() -> GlowSphere {
        GlowSphere {
            radius: 34.0,
            alpha: 0.3,
            rgb: [1.0, 0.92, 0.7],
            rings: 6,
            segments: 12,
            texture: "textures\\effects\\FXSoftGlowSpot01.dds".to_owned(),
        }
    }

    fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    #[test]
    #[allow(clippy::many_single_char_names)] // the names of the vector algebra
    fn every_face_of_the_sphere_is_wound_outward() {
        let (n, t) = sphere(6, 12);
        let v = scaled(&n, 34.0);
        assert_eq!((v.len(), t.len()), (62, 120));
        for tri in &t {
            let [a, b, c] = tri.map(|i| v[usize::from(i)]);
            let (u, w) = (sub(b, a), sub(c, a));
            let n = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            let m = [a[0] + b[0] + c[0], a[1] + b[1] + c[1], a[2] + b[2] + c[2]];
            assert!(n[0] * m[0] + n[1] * m[1] + n[2] * m[2] > 0.0, "{tri:?}");
        }
        for p in &v {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 34.0).abs() < 1e-9);
        }
    }

    /// Reads the header the way the game does and checks that each block's stated size ends where the next begins.
    fn blocks_of(nif: &[u8]) -> (Vec<String>, Vec<usize>, usize) {
        let mut at = nif.iter().position(|b| *b == b'\n').unwrap() + 1;
        let u32_at = |at: &mut usize| {
            let v = u32::from_le_bytes(nif[*at..*at + 4].try_into().unwrap());
            *at += 4;
            v as usize
        };
        let u16_at = |at: &mut usize| {
            let v = u16::from_le_bytes(nif[*at..*at + 2].try_into().unwrap());
            *at += 2;
            usize::from(v)
        };
        assert_eq!(u32_at(&mut at), 0x1402_0007);
        assert_eq!(nif[at], 1);
        at += 1;
        assert_eq!(u32_at(&mut at), 11);
        let n = u32_at(&mut at);
        assert_eq!(u32_at(&mut at), 34);
        for _ in 0..3 {
            at += 1 + usize::from(nif[at]);
        }
        let types: Vec<String> = (0..u16_at(&mut at))
            .map(|_| {
                let len = u32_at(&mut at);
                at += len;
                String::from_utf8(nif[at - len..at].to_vec()).unwrap()
            })
            .collect();
        let kinds: Vec<String> = (0..n).map(|_| types[u16_at(&mut at)].clone()).collect();
        let sizes: Vec<usize> = (0..n).map(|_| u32_at(&mut at)).collect();
        let strings = u32_at(&mut at);
        let _max = u32_at(&mut at);
        for _ in 0..strings {
            let len = u32_at(&mut at);
            at += len;
        }
        assert_eq!(u32_at(&mut at), 0, "no groups");
        (kinds, sizes, at)
    }

    #[test]
    fn the_file_has_seven_blocks_whose_sizes_add_up_to_the_file() {
        let nif = glow_mesh(&shipped(), "raul");
        let (kinds, sizes, start) = blocks_of(&nif);
        assert_eq!(
            kinds,
            [
                "BSFadeNode",
                "BSXFlags",
                "NiTriShape",
                "NiAlphaProperty",
                "NiMaterialProperty",
                "BSShaderNoLightingProperty",
                "NiTriShapeData"
            ]
        );
        // The footer: one root, block 0.
        assert_eq!(start + sizes.iter().sum::<usize>() + 8, nif.len());
        assert_eq!(&nif[nif.len() - 8..], &[1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn the_shader_names_the_texture_by_path_and_the_data_holds_the_numbers() {
        let glow = shipped();
        let nif = glow_mesh(&glow, "raul");
        let text = glow.texture.as_bytes();
        assert!(nif.windows(text.len()).any(|w| w == text));
        let (_, sizes, start) = blocks_of(&nif);
        let data = &nif[start + sizes[..6].iter().sum::<usize>()..][..sizes[6]];
        let f = |at: usize| f32::from_le_bytes(data[at..at + 4].try_into().unwrap());
        assert_eq!(u16::from_le_bytes([data[4], data[5]]), 62, "vertices");
        // The first vertex is the top pole.
        assert_eq!((f(9), f(13), f(17)), (0.0, 0.0, 34.0));
        // After vertices, normals and the bounding sphere: the first colour.
        let colours = 9 + 62 * 12 + 3 + 62 * 12 + 16 + 1;
        assert_eq!(f(colours + 12), 0.3);
        assert_eq!(f(colours), 1.0);
        assert_eq!(f(colours + 4), 0.92);
        assert_eq!(f(colours - 5), 34.0, "bounding radius");
    }

    /// The bytes of the mesh that was checked in the engine, at night, beside the mod's own glow: the same numbers
    /// must give the same file. Only on Linux, where it was made: another platform's sine may round one point apart.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_shipped_numbers_give_the_mesh_that_was_checked_in_game() {
        use sha2::{Digest, Sha256};
        let nif = glow_mesh(&shipped(), "raul");
        assert_eq!(nif.len(), 4446);
        let digest = Sha256::digest(&nif);
        let hex = digest.iter().fold(String::new(), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
            s
        });
        assert_eq!(
            hex,
            "d75537273d5267a9bec33852d101e37988b789fdb68d35015b019a766620aa4c"
        );
    }

    #[test]
    fn a_bigger_radius_changes_the_points_and_nothing_else_of_the_layout() {
        let small = glow_mesh(
            &GlowSphere {
                radius: 24.0,
                ..shipped()
            },
            "raul",
        );
        let big = glow_mesh(&shipped(), "raul");
        assert_eq!(small.len(), big.len());
        assert_ne!(small, big);
    }
}
