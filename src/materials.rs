use crate::hittables::Hit;
use crate::render::Ray;
use crate::{color3, Color3};
use glam::{vec3, Vec3};
use rand::Rng;

#[derive(Copy, Clone)]
pub enum Material {
    Lambertian { albedo: Color3 },
    Metal { albedo: Color3, fuzz: f32 },
    Dielectric { refract_idx: f32 },
    DiffuseLight { emit: Vec3 },
}

impl Material {
    pub fn new_lambertian(r: f32, g: f32, b: f32) -> Material {
        Material::Lambertian {
            albedo: color3(r, g, b),
        }
    }

    pub fn new_metal(r: f32, g: f32, b: f32, fuzz: f32) -> Material {
        Material::Metal {
            albedo: color3(r, g, b),
            fuzz,
        }
    }

    pub fn new_dielectric(refract_idx: f32) -> Material {
        Material::Dielectric { refract_idx }
    }

    pub fn new_light(r: f32, g: f32, b: f32) -> Material {
        Material::DiffuseLight {
            emit: color3(r, g, b),
        }
    }

    pub fn scatter(ray: &Ray, hit: &Hit) -> Option<Scattered> {
        match hit.material {
            Material::Lambertian { albedo } => {
                let mut scatter_dir = hit.normal + random_sphere_vec3();
                if is_near_zero(scatter_dir) {
                    scatter_dir = hit.normal;
                }

                Some(Scattered {
                    ray: Ray::new(hit.p, scatter_dir),
                    attenuation: albedo,
                })
            }
            Material::Metal { albedo, fuzz } => {
                let fuzz = if fuzz < 1.0 { fuzz } else { 1.0 };
                let reflected = reflect(ray.dir().normalize(), hit.normal);
                let scattered = Ray::new(hit.p, reflected + fuzz * random_sphere_vec3());
                if scattered.dir().dot(hit.normal) > 0.0 {
                    Some(Scattered {
                        ray: scattered,
                        attenuation: albedo,
                    })
                } else {
                    None
                }
            }
            Material::Dielectric { refract_idx } => {
                let refract_ratio = if hit.front_face {
                    1.0 / refract_idx
                } else {
                    refract_idx
                };
                let unit_dir = ray.dir().normalize();
                let cos_theta = (-unit_dir).dot(hit.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let reflectance = {
                    let mut r0 = (1.0 - refract_ratio) / (1.0 + refract_ratio);
                    r0 = r0 * r0;
                    r0 + (1.0 - r0) * (1.0 - cos_theta).powi(5)
                };

                let cannot_refract = refract_ratio * sin_theta > 1.0;
                let dir = if cannot_refract || reflectance > rand::random::<f32>() {
                    reflect(unit_dir, hit.normal)
                } else {
                    refract(unit_dir, hit.normal, refract_ratio)
                };

                Some(Scattered {
                    ray: Ray::new(hit.p, dir),
                    attenuation: color3(1.0, 1.0, 1.0),
                })
            }
            Material::DiffuseLight { .. } => None,
        }
    }

    pub fn emitted(&self) -> Color3 {
        match self {
            Material::DiffuseLight { emit } => *emit,
            _ => color3(0.0, 0.0, 0.0),
        }
    }
}

pub struct Scattered {
    pub ray: Ray,
    pub attenuation: Color3,
}

fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
    v - 2.0 * v.dot(normal) * normal
}

fn refract(uv: Vec3, normal: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(normal).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * normal);
    let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * normal;
    r_out_perp + r_out_parallel
}

fn is_near_zero(v: Vec3) -> bool {
    const EPSILON: f32 = 1e-8;
    v.x.abs() < EPSILON && v.y.abs() < EPSILON && v.z.abs() < EPSILON
}

fn random_sphere_vec3() -> Vec3 {
    loop {
        let v = vec3(
            rand::thread_rng().gen_range(-1.0..1.0),
            rand::thread_rng().gen_range(-1.0..1.0),
            rand::thread_rng().gen_range(-1.0..1.0),
        );
        if v.length_squared() < 1.0 {
            return v.normalize();
        }
    }
}
