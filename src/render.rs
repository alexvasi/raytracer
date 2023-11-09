use crate::hittables::{Hittable, HittableVec, Interval};
use crate::materials::Material;
use crate::{color3, point3, Color3, Point3};
use glam::{vec3, Vec3};
use rand::Rng;

pub struct Ray {
    origin: Point3,
    dir: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn origin(&self) -> Point3 {
        self.origin
    }

    pub fn dir(&self) -> Vec3 {
        self.dir
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.dir * t
    }
}

pub struct Camera {
    samples_per_pixel: u32,
    max_depth: u32,
    background: Color3,

    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_angle: f32,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn builder(image_width: u32, image_height: u32) -> CameraBuilder {
        CameraBuilder {
            image_width,
            image_height,
            samples_per_pixel: 10,
            max_depth: 10,
            background: color3(1.0, 1.0, 1.0),
            v_fov: 90.0,
            look_from: point3(0.0, 0.0, -1.0),
            look_at: point3(0.0, 0.0, 0.0),
            vup: vec3(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
        }
    }

    fn new(builder: CameraBuilder) -> Self {
        let center = builder.look_from;

        let viewport_height = {
            let theta = builder.v_fov.to_radians();
            let h = (theta / 2.0).tan();
            2.0 * h * builder.focus_dist
        };
        let viewport_width =
            viewport_height * (builder.image_width as f32 / builder.image_height as f32);

        let w = (builder.look_from - builder.look_at).normalize();
        let u = builder.vup.cross(w).normalize();
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / builder.image_width as f32;
        let pixel_delta_v = viewport_v / builder.image_height as f32;

        let viewport_upper_left =
            center - (builder.focus_dist * w) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius = builder.focus_dist * (builder.defocus_angle.to_radians() / 2.0).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            samples_per_pixel: builder.samples_per_pixel,
            max_depth: builder.max_depth,
            background: builder.background,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_angle: builder.defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    pub fn render(&self, x: u32, y: u32, world: &HittableVec) -> Color3 {
        let mut color = Color3::ZERO;

        for _ in 0..self.samples_per_pixel {
            let ray = self.get_ray(x, y);
            color += self.ray_color(&ray, self.max_depth, world)
        }
        color /= self.samples_per_pixel as f32;

        color
    }

    fn ray_color(&self, ray: &Ray, depth: u32, world: &HittableVec) -> Color3 {
        const EPSILON: f32 = 0.001;

        if depth == 0 {
            return color3(0.0, 0.0, 0.0);
        }

        let hit = match world.hit(ray, Interval::new(EPSILON, f32::INFINITY)) {
            Some(hit) => hit,
            None => {
                return self.background;
            }
        };

        let emission_color = hit.material.emitted();
        let scatter_color = match Material::scatter(ray, &hit) {
            Some(scattered) => {
                scattered.attenuation * self.ray_color(&scattered.ray, depth - 1, world)
            }
            None => color3(0.0, 0.0, 0.0),
        };

        emission_color + scatter_color
    }

    fn get_ray(&self, x: u32, y: u32) -> Ray {
        let pixel_center =
            self.pixel00_loc + (x as f32 * self.pixel_delta_u) + (y as f32 * self.pixel_delta_v);
        let pixel_sample = pixel_center + self.random_pixel_sample();
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        Ray::new(ray_origin, pixel_sample - ray_origin)
    }

    fn random_pixel_sample(&self) -> Vec3 {
        let px = rand::random::<f32>() - 0.5;
        let py = rand::random::<f32>() - 0.5;
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = Self::random_in_unit_disk();
        self.center + (p[0] * self.defocus_disk_u) + (p[1] * self.defocus_disk_v)
    }

    fn random_in_unit_disk() -> Vec3 {
        loop {
            let v = vec3(
                rand::thread_rng().gen_range(-1.0..1.0),
                rand::thread_rng().gen_range(-1.0..1.0),
                0.0,
            );
            if v.length_squared() < 1.0 {
                return v;
            }
        }
    }
}

pub struct CameraBuilder {
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    max_depth: u32,
    background: Color3,
    v_fov: f32,
    look_from: Point3,
    look_at: Point3,
    vup: Vec3,
    defocus_angle: f32,
    focus_dist: f32,
}

impl CameraBuilder {
    pub fn build(self) -> Camera {
        Camera::new(self)
    }

    pub fn samples(mut self, samples_per_pixel: u32) -> Self {
        self.samples_per_pixel = samples_per_pixel;
        self
    }

    pub fn max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn background(mut self, background: Color3) -> Self {
        self.background = background;
        self
    }

    pub fn vert_fov(mut self, v_fov: f32) -> Self {
        self.v_fov = v_fov;
        self
    }

    pub fn look_from(mut self, from: Point3) -> Self {
        self.look_from = from;
        self
    }

    pub fn look_at(mut self, at: Point3) -> Self {
        self.look_at = at;
        self
    }

    pub fn look_up(mut self, vup: Vec3) -> Self {
        self.vup = vup;
        self
    }

    pub fn defocus_angle(mut self, angle: f32) -> Self {
        self.defocus_angle = angle;
        self
    }

    pub fn focus_dist(mut self, dist: f32) -> Self {
        self.focus_dist = dist;
        self
    }
}
