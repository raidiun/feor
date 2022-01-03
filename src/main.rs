use std::io;

extern crate bmp;
use bmp::{Image,Pixel,px};

use rand::prelude::*;

extern crate nalgebra as na;

type Fpr = f64;

type Vector3 = na::Vector3<Fpr>;
type Colour = Vector3;

struct PixelColour(Pixel);

impl From<Colour> for PixelColour {
    fn from(vec: Colour) -> Self {
        let r = (vec[0].sqrt()*255.999).round();
        let g = (vec[1].sqrt()*255.999).round();
        let b = (vec[2].sqrt()*255.999).round();
        PixelColour {
            0: px!(r,g,b)
        }
    }
}


impl Into<Pixel> for PixelColour {
    fn into(self: Self) -> Pixel {
        self.0
    }
}

fn random_in_sphere() -> Vector3 {
    let mut rng = rand::thread_rng();
    loop {
        let x = rng.gen_range(-1.0..=1.0);
        let y = rng.gen_range(-1.0..=1.0);
        let z = rng.gen_range(-1.0..=1.0);
        let vec = Vector3::new(x,y,z);
        if vec.norm() <= 1.0 {
            return vec
        }
    }
}

struct Ray {
    origin: Vector3,
    direction: Vector3,
}

impl Ray {
    fn new(origin: Vector3, direction: Vector3) -> Self {
        Self { origin, direction }
    }

    fn at(&self, t: Fpr) -> Vector3 {
        self.origin + t * self.direction
    }
}

struct Hit<'a> {
    t: Fpr,
    position: Vector3,
    normal: Vector3,
    material: &'a Box<dyn Material + 'a>,
}

trait Material {
    fn response(&self, ray: &Ray, hit: &Hit) -> (Colour,Option<Ray>);
}

struct Diffuse {
    colour: Colour,
}

impl Diffuse {
    fn new(colour: Colour) -> Self {
        Diffuse {
            colour
        }
    }
}

impl Material for Diffuse {
    fn response(&self, _ray: &Ray, hit: &Hit) -> (Colour,Option<Ray>) {
        let target = hit.position + hit.normal + random_in_sphere();
        (self.colour, Some(Ray::new(hit.position,target-hit.position)))
    }
}

struct Metal {
    colour: Colour,
}

impl Metal {
    fn new(colour: Colour) -> Self {
        Metal {
            colour
        }
    }
}

impl Material for Metal {
    fn response(&self, ray: &Ray, hit: &Hit) -> (Colour,Option<Ray>) {
        let raynorm = ray.direction.normalize();
        let reflected = raynorm - 2.0 * raynorm.dot(&hit.normal) * hit.normal;
        (self.colour, Some(Ray::new(hit.position,reflected)))
    }
}

struct Dielectric {
    colour: Colour,
    refractive_index: Fpr,
}

impl Dielectric {
    fn new(colour: Colour, refractive_index: Fpr) -> Self {
        Dielectric {
            colour,
            refractive_index,
        }
    }

    fn reflectance(cos_theta: Fpr, refrac_ratio: Fpr) -> Fpr {
        let r02 = ((1.0-refrac_ratio) / (1.0+refrac_ratio)).powi(2);
        r02 + (1.0-r02)*(1.0-cos_theta).powi(5)
    }
}

impl Material for Dielectric {
    fn response(&self, ray: &Ray, hit: &Hit) -> (Colour,Option<Ray>) {

        let raynorm = ray.direction.normalize();

        let mut hnormal = hit.normal;
        let mut raydotnorm = raynorm.dot(&hit.normal);

        let refrac_ratio;
        if raydotnorm > 0.0 { // Norm and ray in same direction
            // Inside
            hnormal = -hnormal;
            raydotnorm = -raydotnorm;
            refrac_ratio = self.refractive_index;
        } else {
            // Outside
            refrac_ratio = self.refractive_index.recip();
        };

        let cos_theta = Fpr::min(-raydotnorm,1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = refrac_ratio * sin_theta > 1.0;
        if  cannot_refract || Self::reflectance(cos_theta,refrac_ratio) > rand::random() {
            let raynorm = ray.direction.normalize();
            let reflected = raynorm - 2.0 * raynorm.dot(&hit.normal) * hit.normal;
            (self.colour, Some(Ray::new(hit.position,reflected)))
        }
        else {
            let out_tang = refrac_ratio * (raynorm + cos_theta*hnormal);
            let out_norm = -(1.0-out_tang.norm_squared()).abs().sqrt() * hnormal;
    
            let refracted = out_tang + out_norm;
    
            (self.colour, Some(Ray::new(hit.position,refracted)))
        }
        
    }
}

trait Hittable<'a> {
    fn get_hit(&self, ray: &Ray, tmin: Option<Fpr>, tmax: Option<Fpr>) -> Option<Hit>;
    fn get_material(&self) -> &'a Box<dyn Material + 'a>;
}

struct Sphere<'a> {
    centre: Vector3,
    radius: Fpr,
    material: &'a Box<dyn Material + 'a>,
}

impl<'a> Hittable<'a> for Sphere<'a> {
    fn get_hit(&self, ray: &Ray, tmin_opt: Option<Fpr>, tmax_opt: Option<Fpr>) -> Option<Hit> {
        let oc = ray.origin - self.centre;
        let a = ray.direction.dot(&ray.direction);
        let half_b = oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius.powi(2);
        let discriminant = half_b.powi(2) - a*c;

        if discriminant < 0.0 {
            return None;
        }

        let tmin = tmin_opt.unwrap_or(0.0001);
        let tmax = tmax_opt.unwrap_or(Fpr::INFINITY);

        let dsqrt = discriminant.sqrt();
        let mut t = (-half_b - dsqrt) / a;
        if t < tmin || t > tmax {
            // Try other root
            t = (-half_b + dsqrt) / a;
            if t < tmin || t > tmax { 
                return None
            }
        }

        let position = ray.at(t);
        let normal = (position - self.centre).normalize();
        Some( Hit {
            t,
            position,
            normal,
            material: self.get_material(),
        })
        
    }

    fn get_material(&self) -> &'a Box<dyn Material + 'a> {
        &self.material
    }

}

struct HittableList<'a> {
    hittables: Vec<&'a dyn Hittable<'a>>,
}

impl HittableList<'_> {
    fn get_hit(&self, ray: &Ray, tmin: Option<Fpr>, tmax: Option<Fpr>) -> Option<Hit> {
        let mut closest = tmax.unwrap_or(Fpr::INFINITY);
        let mut hit = None;

        for hittable in &self.hittables {
            if let Some(h) = hittable.get_hit(ray,tmin,Some(closest)) {
                closest = h.t;
                hit = Some(h);
            }
        }

        hit
    }
}

struct ViewPort {
    aspect_ratio: Fpr,
    width: Fpr,
    height: Fpr,
}

impl ViewPort {
    fn new(aspect_ratio: Fpr, height: Fpr) -> Self {
        let width = aspect_ratio * height;
        
        ViewPort {
            aspect_ratio,
            width,
            height,
        }
    }
}

struct Camera {
    origin: Vector3,
    focal_length: Fpr,
    viewport: ViewPort,
    horizontal: Vector3,
    vertical: Vector3,
    image_origin: Vector3,
}

impl Camera {
    fn new(origin: Vector3, focal_length: Fpr, viewport: ViewPort, horizontal: Vector3, vertical: Vector3) -> Self {
        let horizontal = horizontal.normalize() * viewport.width;
        let vertical = vertical.normalize() * viewport.height;
        let image_origin = origin - horizontal/2.0 - vertical/2.0 - Vector3::new(0.0,0.0,focal_length);

        Camera {
            origin,
            focal_length,
            viewport,
            horizontal,
            vertical,
            image_origin,
        }
    }

    fn get_ray(&self, u: Fpr, v: Fpr) -> Ray {
        Ray::new(self.origin,self.image_origin + u*self.horizontal + v*self.vertical - self.origin)
    }
}

fn background_colour(ray: &Ray) -> Colour {
    let dirnorm = ray.direction.normalize();
    let t = 0.5 * (dirnorm.y + 1.0);
    (1.0-t)*Colour::new(1.0,1.0,1.0) + t*Colour::new(0.5,0.7,1.0)
}

fn get_ray_colour(ray: &Ray, world: &HittableList, depth: u32) -> Colour {

    if depth <= 0 {
        return Colour::zeros();
    }

    if let Some(hit) = world.get_hit(ray,None,None) {

        let (attenuation,scattered) = hit.material.response(ray,&hit);

        if let Some(scattered) = scattered {
            get_ray_colour(&scattered, world, depth-1).component_mul(&attenuation)
        }
        else {
            Colour::zeros()
        }

    }
    else {
        background_colour(ray)
    }

}

fn main() -> io::Result<()> {
    // Image configuration
    const ASPECT_RATIO: Fpr = 16.0 / 9.0;
    const IMAGE_WIDTH: u32 = 400;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as Fpr / ASPECT_RATIO) as u32;

    // World configuration
    let diffuse_red: Box<dyn Material> = Box::new(Diffuse::new(Colour::new(1.0,0.0,0.0)));
    let diffuse_green: Box<dyn Material> = Box::new(Diffuse::new(Colour::new(0.0,1.0,0.0)));
    let diffuse_blue: Box<dyn Material> = Box::new(Diffuse::new(Colour::new(0.0,0.0,1.0)));
    
    let metal: Box<dyn Material> = Box::new(Metal::new(Colour::new(0.8,0.8,0.8)));
    let glass: Box<dyn Material> = Box::new(Dielectric::new(Colour::new(0.9,0.8,0.9),1.5));

    let centre_sph = Sphere { centre: Vector3::new(0.0,0.0,-1.0), radius: 0.5, material: &glass};
    let right_sph = Sphere { centre: Vector3::new(1.0,0.0,-1.0), radius: 0.2, material: &diffuse_blue};
    let left_sph = Sphere { centre: Vector3::new(-1.0,0.0,-1.0), radius: 0.4, material: &diffuse_red};
    let world_sph = Sphere { centre: Vector3::new(0.0,-100.5,-1.0), radius: 100.0, material: &diffuse_green};

    let world = HittableList {
        hittables: vec![
            &centre_sph, &right_sph, &left_sph, &world_sph
        ]
    };

    // Camera configuration
    const VIEWPORT_HEIGHT: Fpr = 2.0;
    let viewport = ViewPort::new(ASPECT_RATIO,VIEWPORT_HEIGHT);
    
    const FOCAL_LENGTH: Fpr = 1.0;
    const CAMERA_ORIGIN: Vector3 = Vector3::new(0.0,0.0,0.0);
    const CAMERA_HORIZ: Vector3 = Vector3::new(1.0,0.0,0.0);
    const CAMERA_VERT: Vector3 = Vector3::new(0.0,1.0,0.0);
    let camera = Camera::new(CAMERA_ORIGIN,FOCAL_LENGTH,viewport,CAMERA_HORIZ,CAMERA_VERT);


    // Actual rendering
    let mut img = Image::new(IMAGE_WIDTH,IMAGE_HEIGHT);

    const PIXEL_SAMPLES: u32 = 500;
    const MAX_DEPTH: u32 = 150;

    let mut rng = rand::thread_rng();

    for j in 0..IMAGE_HEIGHT {
        for i in 0..IMAGE_WIDTH {
            
            let mut colour = Colour::zeros();
            for _ in 0..PIXEL_SAMPLES {
                let image_x = i as Fpr + rng.gen::<Fpr>();
                let image_y = (IMAGE_HEIGHT-1-j) as Fpr + rng.gen::<Fpr>();

                let u = image_x / (IMAGE_WIDTH-1) as Fpr;
                let v = image_y / (IMAGE_HEIGHT-1) as Fpr;

                let ray = camera.get_ray(u, v);
                colour += get_ray_colour(&ray,&world,MAX_DEPTH);
            }
            
            img.set_pixel(i, j, PixelColour::from(colour/PIXEL_SAMPLES as Fpr).into() );
        }
    }

    img.save("img.bmp")
}
