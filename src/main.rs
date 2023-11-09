mod canvas;
mod hittables;
mod materials;
mod render;

use anyhow::Result;
use canvas::Canvas;
use glam::{vec3, Vec3};
use hittables::{make_box, HittableVec, Quad, RotateY, Sphere, Translate};
use indicatif::ProgressBar;
use materials::Material;
use rayon::prelude::*;
use render::{Camera, CameraBuilder};
use std::path::Path;

fn main() -> Result<()> {
    const WIDTH: u32 = 800;
    const ASPECT: f32 = 1.0;
    const HEIGHT: u32 = (WIDTH as f32 / ASPECT) as u32;

    let mut canvas = Canvas::new(WIDTH, HEIGHT);
    let mut world: HittableVec = vec![];
    let camera = Camera::builder(WIDTH, HEIGHT).samples(50).max_depth(50);
    let camera = cornell_box(&mut world, camera);

    let bar = ProgressBar::new(HEIGHT as u64);
    let start = std::time::Instant::now();
    let mut scanline: Vec<Color3> = vec![];
    for y in 0..HEIGHT {
        (0..WIDTH)
            .into_par_iter()
            .map(|x| camera.render(x, y, &world))
            .collect_into_vec(&mut scanline);

        for (x, color) in scanline.iter().enumerate() {
            canvas.draw(x as u32, y, *color);
        }

        bar.inc(1);
    }
    bar.finish();
    println!("Rendered in {:?}", start.elapsed());

    canvas.save(Path::new(r"output.png"))?;
    Ok(())
}

#[allow(dead_code)]
fn spheres_scene(world: &mut HittableVec, cam_builder: CameraBuilder) -> Camera {
    let mat_ground = Material::new_lambertian(0.8, 0.8, 0.0);
    let mat_center = Material::new_lambertian(0.1, 0.2, 0.5);
    let mat_left = Material::new_dielectric(1.5);
    let mat_right = Material::new_metal(0.8, 0.6, 0.2, 0.0);

    world.append(&mut vec![
        Box::new(Sphere::new(point3(0.0, -100.5, -1.0), 100.0, mat_ground)),
        Box::new(Sphere::new(point3(0.0, 0.0, -1.0), 0.5, mat_center)),
        Box::new(Sphere::new(point3(-1.0, 0.0, -1.0), 0.5, mat_left)),
        Box::new(Sphere::new(point3(-1.0, 0.0, -1.0), -0.4, mat_left)),
        Box::new(Sphere::new(point3(1.0, 0.0, -1.0), 0.5, mat_right)),
    ]);

    cam_builder
        .background(color3(0.7, 0.8, 1.0))
        .vert_fov(20.0)
        .look_from(point3(-2.0, 2.0, 1.0))
        .look_at(point3(0.0, 0.0, -1.0))
        .look_up(vec3(0.0, 1.0, 0.0))
        .defocus_angle(3.0)
        .focus_dist(3.4)
        .build()
}

#[allow(dead_code)]
fn cornell_box(world: &mut HittableVec, cam_builder: CameraBuilder) -> Camera {
    let red = Material::new_lambertian(0.65, 0.05, 0.05);
    let white = Material::new_lambertian(0.73, 0.73, 0.73);
    let green = Material::new_lambertian(0.12, 0.45, 0.15);
    let light = Material::new_light(15.0, 15.0, 15.0);

    world.append(&mut vec![
        Box::new(Quad::new(
            point3(555.0, 0.0, 0.0),
            vec3(0.0, 555.0, 0.0),
            vec3(0.0, 0.0, 555.0),
            green,
        )),
        Box::new(Quad::new(
            point3(0.0, 0.0, 0.0),
            vec3(0.0, 555.0, 0.0),
            vec3(0.0, 0.0, 555.0),
            red,
        )),
        Box::new(Quad::new(
            point3(343.0, 554.0, 332.0),
            vec3(-130.0, 0.0, 0.0),
            vec3(0.0, 0.0, -105.0),
            light,
        )),
        Box::new(Quad::new(
            point3(0.0, 0.0, 0.0),
            vec3(555.0, 0.0, 0.0),
            vec3(0.0, 0.0, 555.0),
            white,
        )),
        Box::new(Quad::new(
            point3(555.0, 555.0, 555.0),
            vec3(-555.0, 0.0, 0.0),
            vec3(0.0, 0.0, -555.0),
            white,
        )),
        Box::new(Quad::new(
            point3(0.0, 0.0, 555.0),
            vec3(555.0, 0.0, 0.0),
            vec3(0.0, 555.0, 0.0),
            white,
        )),
        Box::new(Translate::new(
            vec3(265.0, 0.0, 295.0),
            Box::new(RotateY::new(
                15.0,
                Box::new(make_box(
                    point3(0.0, 0.0, 0.0),
                    point3(165.0, 330.0, 165.0),
                    white,
                )),
            )),
        )),
        Box::new(Translate::new(
            vec3(130.0, 0.0, 65.0),
            Box::new(RotateY::new(
                -18.0,
                Box::new(make_box(
                    point3(0.0, 0.0, 0.0),
                    point3(165.0, 165.0, 165.0),
                    white,
                )),
            )),
        )),
    ]);

    cam_builder
        .background(color3(0.0, 0.0, 0.0))
        .vert_fov(40.0)
        .look_from(point3(278.0, 278.0, -800.0))
        .look_at(point3(278.0, 278.0, 0.0))
        .look_up(vec3(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .build()
}

type Color3 = Vec3;
type Point3 = Vec3;

fn point3(x: f32, y: f32, z: f32) -> Point3 {
    Point3::new(x, y, z)
}

fn color3(x: f32, y: f32, z: f32) -> Color3 {
    Color3::new(x, y, z)
}
