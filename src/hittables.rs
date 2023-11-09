use crate::materials::Material;
use crate::render::Ray;
use crate::{point3, Point3};
use glam::{vec3, Vec3};

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<Hit>;
}

pub struct Hit {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
    pub material: Material,
}

impl Hit {
    fn new(p: Point3, outward_normal: Vec3, ray: &Ray, t: f32, material: Material) -> Self {
        let front_face = ray.dir().dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            p,
            normal,
            t,
            front_face,
            material,
        }
    }
}

pub struct Sphere {
    center: Point3,
    radius: f32,
    mat: Material,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32, mat: Material) -> Self {
        Self {
            center,
            radius,
            mat,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<Hit> {
        let oc = ray.origin() - self.center;
        let a = ray.dir().length_squared();
        let half_b = oc.dot(ray.dir());
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (-half_b + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }

        let t = root;
        let p = ray.at(t);
        let outward_normal = (p - self.center) / self.radius;
        Some(Hit::new(p, outward_normal, ray, t, self.mat))
    }
}

pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    mat: Material,

    normal: Vec3,
    d: f32,
    w: Vec3,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Material) -> Self {
        let n = u.cross(v);
        let normal = n.normalize();
        let d = normal.dot(q);
        let w = n / n.dot(n);
        Quad {
            q,
            u,
            v,
            mat,
            normal,
            d,
            w,
        }
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<Hit> {
        const EPSILON: f32 = 1e-8;

        let denom = self.normal.dot(ray.dir());
        if denom.abs() < EPSILON {
            return None;
        }

        let t = (self.d - self.normal.dot(ray.origin())) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        let intersection = ray.at(t);
        let planar_hit_vector = intersection - self.q;
        let alpha = self.w.dot(planar_hit_vector.cross(self.v));
        let beta = self.w.dot(self.u.cross(planar_hit_vector));

        if alpha < 0.0 || alpha > 1.0 || beta < 0.0 || beta > 1.0 {
            return None;
        }

        Some(Hit::new(intersection, self.normal, ray, t, self.mat))
    }
}

pub type HittableVec = Vec<Box<dyn Hittable>>;

impl Hittable for HittableVec {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<Hit> {
        let mut closest_hit = None;
        let mut closest_t = ray_t.max;

        for obj in self {
            if let Some(hit) = obj.hit(ray, Interval::new(ray_t.min, closest_t)) {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }
        closest_hit
    }
}

pub struct Translate {
    offset: Vec3,
    object: Box<dyn Hittable>,
}

impl Translate {
    pub fn new(offset: Vec3, object: Box<dyn Hittable>) -> Self {
        Self { offset, object }
    }
}

impl Hittable for Translate {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<Hit> {
        let offset_r = Ray::new(ray.origin() - self.offset, ray.dir());

        match self.object.hit(&offset_r, ray_t) {
            Some(mut hit) => {
                hit.p += self.offset;
                Some(hit)
            }
            None => None,
        }
    }
}

pub struct RotateY {
    sin_theta: f32,
    cos_theta: f32,
    object: Box<dyn Hittable>,
}

impl RotateY {
    pub fn new(angle: f32, object: Box<dyn Hittable>) -> Self {
        let angle = angle.to_radians();
        RotateY {
            sin_theta: angle.sin(),
            cos_theta: angle.cos(),
            object,
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<Hit> {
        let origin = vec3(
            self.cos_theta * ray.origin().x - self.sin_theta * ray.origin().z,
            ray.origin().y,
            self.sin_theta * ray.origin().x + self.cos_theta * ray.origin().z,
        );
        let dir = vec3(
            self.cos_theta * ray.dir().x - self.sin_theta * ray.dir().z,
            ray.dir().y,
            self.sin_theta * ray.dir().x + self.cos_theta * ray.dir().z,
        );
        let rotated_r = Ray::new(origin, dir);

        let mut hit = match self.object.hit(&rotated_r, ray_t) {
            Some(hit) => hit,
            None => {
                return None;
            }
        };

        hit.p = point3(
            self.cos_theta * hit.p.x + self.sin_theta * hit.p.z,
            hit.p.y,
            -self.sin_theta * hit.p.x + self.cos_theta * hit.p.z,
        );
        hit.normal = vec3(
            self.cos_theta * hit.normal.x + self.sin_theta * hit.normal.z,
            hit.normal.y,
            hit.normal.y - self.sin_theta * hit.normal.x + self.cos_theta * hit.normal.z,
        );
        Some(hit)
    }
}

pub fn make_box(a: Point3, b: Point3, mat: Material) -> HittableVec {
    let min = point3(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = point3(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    let dx = vec3(max.x - min.x, 0.0, 0.0);
    let dy = vec3(0.0, max.y - min.y, 0.0);
    let dz = vec3(0.0, 0.0, max.z - min.z);

    vec![
        Box::new(Quad::new(point3(min.x, min.y, max.z), dx, dy, mat)), // front
        Box::new(Quad::new(point3(max.x, min.y, max.z), -dz, dy, mat)), // right
        Box::new(Quad::new(point3(max.x, min.y, min.z), -dx, dy, mat)), // back
        Box::new(Quad::new(point3(min.x, min.y, min.z), dz, dy, mat)), // left
        Box::new(Quad::new(point3(min.x, max.y, max.z), dx, -dz, mat)), // top
        Box::new(Quad::new(point3(min.x, min.y, min.z), dx, dz, mat)), // bottom
    ]
}

#[derive(Copy, Clone)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    const _EMPTY: Interval = Interval {
        min: f32::INFINITY,
        max: f32::NEG_INFINITY,
    };

    const _UNIVERSE: Interval = Interval {
        min: f32::NEG_INFINITY,
        max: f32::INFINITY,
    };

    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    fn contains(&self, val: f32) -> bool {
        self.min <= val && val <= self.max
    }

    fn surrounds(&self, val: f32) -> bool {
        self.min < val && val < self.max
    }
}
