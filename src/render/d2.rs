use super::{Vertex, View, WINDOW};
use crate::object::d2::{Obj2d, DrawUpdate};

use glam::{vec2, vec3, Mat4, Vec2, Vec3};
use std::f32::consts::TAU;

pub(crate) static mut RENDERS: Vec<Option<(Vec<Vertex>, Vec<u16>, Option<usize>, DrawUpdate)>> = Vec::new();

pub(super) static mut PROJ: Mat4 = Mat4::IDENTITY;
pub(crate) static mut MOUSE_PROJ: Vec2 = Vec2::ZERO;

pub(crate) static mut CANVAS: Vec2 = Vec2::new(1280., 720.);
pub(crate) static mut CANVAS_UPDATE: bool = false;
pub(crate) static mut CANVAS_PROJ: Vec2 = Vec2::new(1280. / 2., 720. / 2.);

pub(crate) static mut VIEW_WIDTH: View = View::KeepHeight;
pub(crate) static mut VIEW_HEIGHT: View = View::KeepWidth;

pub(crate) static mut CAMERA2D: Vec2 = Vec2::ZERO;

pub(crate) static mut ZOOM: f32 = 1.;

pub struct Camera2d;
impl Camera2d {
    pub fn set(self, pos: Vec2) -> Self {
        unsafe {
            CAMERA2D = pos;
            CANVAS_UPDATE = true;
        }
        self
    }

    pub fn get(&self) -> Vec2 {
        unsafe {
            CAMERA2D
        }
    }

    pub fn set_zoom(self, n: f32) -> Self {
        unsafe {
            ZOOM = n;
            CANVAS_UPDATE = true;
        }
        self
    }

    pub fn get_zoom(&self) -> f32 {
        unsafe {
            ZOOM
        }
    }

    pub fn view(self, width: View, height: View) -> Self {
        unsafe {
            VIEW_WIDTH = width;
            VIEW_HEIGHT = height;
        }
        self
    }
}

#[inline(always)]
pub(crate) fn draw(
    id: usize,
    pos: Vec2,
    obj: &Obj2d,
    scale: Vec2,
    rotation: f32,
    offset: Vec2,
    color: [f32; 4],
) {
    match obj {
        Obj2d::None => {}
        Obj2d::Circle(r) => {
            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            let segments = 40;

            vertices.push(Vertex {
                pos: vec3(pos.x, pos.y, 0.),
                color,
                uv: Vec2::new(0., 0.),
            });

            let offset = if offset == vec2(0., 0.) {
                offset
            } else {
                offset * r * scale
            };

            for i in 0..segments {
                let theta = i as f32 / segments as f32 * TAU;
                let x = r * theta.cos() * scale.x + offset.x;
                let y = r * theta.sin() * scale.y + offset.y;
                let p = rotate(vec2(x, y), pos, rotation);
                vertices.push(Vertex {
                    pos: p,
                    color: color,
                    uv: Vec2::new(0., 0.),
                });
            }

            for i in 1..segments {
                indices.extend([0, i as u16, (i + 1) as u16]);
            }
            indices.extend([0, segments, 1]);

            render(id, vertices, indices);
        }
        Obj2d::Rect(w, h, r) => {
            let w = (w * scale.x) / 2.;
            let h = (h * scale.y) / 2.;

            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            let offset = if offset == vec2(0., 0.) {
                offset
            } else {
                offset * vec2(w, h) * scale
            };

            if *r <= 1. {
                vertices.extend([
                    Vertex {
                        pos: rotate(vec2(-w + offset.x, -h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w + offset.x, -h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w + offset.x, h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(-w + offset.x, h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                ]);
                indices.extend(vec![0, 1, 3, 1, 2, 3]);
            } else {
                let segments = 40;

                let half_segments = segments / 4;

                let corner_centers = [
                    vec2(w - r, h - r),   // bottom-right
                    vec2(-w + r, h - r),  // bottom-left
                    vec2(-w + r, -h + r), // top-left
                    vec2(w - r, -h + r),  // top-right
                ];

                vertices.push(Vertex {
                    pos: vec3(pos.x, pos.y, 0.),
                    color,
                    uv: Vec2::new(0., 0.),
                });

                for (corner_index, &center) in corner_centers.iter().enumerate() {
                    for i in 0..half_segments {
                        let theta =
                            (corner_index * half_segments + i) as f32 / segments as f32 * TAU;
                        let x = center.x + r * theta.cos() * scale.x + offset.x;
                        let y = center.y + r * theta.sin() * scale.y + offset.y;
                        let p = rotate(vec2(x, y), pos, rotation);
                        vertices.push(Vertex {
                            pos: p,
                            color: color,
                            uv: Vec2::new(0., 0.),
                        });
                    }
                }

                for i in 1..segments {
                    indices.extend([0, i as u16, (i + 1) as u16]);
                }
                indices.extend([0, segments as u16, 1]);
            }

            render(id, vertices, indices);
        }
        Obj2d::Texture(t) | Obj2d::Text(_, _, _, t) => {
            let w = (t.width as f32 * scale.x) / 2.;
            let h = (t.height as f32 * scale.y) / 2.;

            let offset = offset * vec2(w, h) * scale;

            render(id, 
                vec![
                    Vertex {
                        pos: rotate(vec2(-w + offset.x, -h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w + offset.x, -h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(1., 0.),
                    },
                    Vertex {
                        pos: rotate(vec2(w + offset.x, h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(1., 1.),
                    },
                    Vertex {
                        pos: rotate(vec2(-w + offset.x, h + offset.y), pos, rotation),
                        color: color,
                        uv: Vec2::new(0., 1.),
                    },
                ],
                vec![0, 1, 3, 1, 2, 3],
            );
        }
    }
}

#[inline(always)]
fn rotate(p: Vec2, center: Vec2, rotation: f32) -> Vec3 {
    if rotation != 0. {
        let s = rotation.sin();
        let c = rotation.cos();

        vec3(
            (p.x * c - p.y * s) + center.x,
            (p.x * s + p.y * c) + center.y,
            0.,
        )
    } else {
        vec3(p.x + center.x, p.y + center.y, 0.)
    }
}

#[inline(always)]
fn render(id: usize, mut vert: Vec<Vertex>, mut indi: Vec<u16>) {
    unsafe {
        RENDERS[id].as_mut().unwrap().0 = vert;
        RENDERS[id].as_mut().unwrap().1 = indi;
        /*let needs_new_batch = match RENDERS.last() {
            Some((_, _, last_img)) => *last_img != img,
            None => true,
        };

        if needs_new_batch {
            RENDERS.push((vert, indi, img));
        } else {
            let last = RENDERS.last_mut().unwrap();
            let base_index = last.0.len() as u16;

            // Смещаем индексы на количество уже имеющихся вершин
            for index in &mut indi {
                *index += base_index;
            }

            last.0.extend(vert);
            last.1.extend(indi);
        }*/
    }
}

#[inline(always)]
pub(crate) fn new_render() -> usize {
    unsafe {
        RENDERS.push(Some((vec![], vec![], None, DrawUpdate::Create)));
        RENDERS.len() - 1
    }
}

#[inline(always)]
pub(crate) fn del_render(id: usize) {
    unsafe {
        RENDERS[id] = None;
    }
}

pub(crate) fn upd_proj() {
    unsafe {
        let aspect_window = WINDOW.x / WINDOW.y;
        let aspect_canvas = CANVAS.x / CANVAS.y;

        let canvas = CANVAS / 2. * ZOOM;
        let window = WINDOW / 2.;

        let view: &View = if aspect_window > aspect_canvas {
            &VIEW_WIDTH
        } else {
            &VIEW_HEIGHT
        };

        let proj = match view {
            View::KeepWidth => {
                let scale = canvas.y / (aspect_window / aspect_canvas);
                MOUSE_PROJ = vec2(canvas.x / window.x, scale / window.y);
                CANVAS_PROJ = vec2(canvas.x, scale);
                Mat4::orthographic_rh_gl(
                    -canvas.x + CAMERA2D.x,
                    canvas.x + CAMERA2D.x,
                    scale + CAMERA2D.y,
                    -scale + CAMERA2D.y,
                    -1.0,
                    1.0,
                )
            }
            View::KeepHeight => {
                let scale = canvas.x / (aspect_canvas / aspect_window);
                MOUSE_PROJ = vec2(scale / window.x, canvas.y / window.y);
                CANVAS_PROJ = vec2(scale, canvas.y);
                Mat4::orthographic_rh_gl(
                    -scale + CAMERA2D.x,
                    scale + CAMERA2D.x,
                    canvas.y + CAMERA2D.y,
                    -canvas.y + CAMERA2D.y,
                    -1.0,
                    1.0,
                )
            }
            View::Scale => {
                MOUSE_PROJ = vec2(canvas.x / window.x, canvas.y / window.y);
                CANVAS_PROJ = vec2(canvas.x, canvas.y);
                Mat4::orthographic_rh_gl(
                    -canvas.x + CAMERA2D.x,
                    canvas.x + CAMERA2D.x,
                    canvas.y + CAMERA2D.y,
                    -canvas.y + CAMERA2D.y,
                    -1.0,
                    1.0,
                )
            }
            View::Window => {
                let window = window * ZOOM;
                MOUSE_PROJ = vec2(ZOOM, ZOOM);
                CANVAS_PROJ = vec2(window.x, window.y);
                Mat4::orthographic_rh_gl(
                    -window.x + CAMERA2D.x,
                    window.x + CAMERA2D.x,
                    window.y + CAMERA2D.y,
                    -window.y + CAMERA2D.y,
                    -1.0,
                    1.0,
                )
            }
        };

        PROJ = proj
    }
}
