use std::io;

use rand::prelude::*;

extern crate nalgebra as na;

extern crate crossbeam;

type Fpr = f64;

type Vector3 = na::Vector3<Fpr>;
type Colour = Vector3;

mod output;

#[derive(Copy,Clone)]
enum Chroma {
    Red = 0,
    Green = 1,
    Blue = 2,
    White,
}

impl From<usize> for Chroma {
    fn from(c: usize) -> Self {
        match c {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            _ => { unreachable!() },
        }
    }
}

struct Ray {
    origin: Vector3,
    direction: Vector3,
    chroma: Chroma,
}

impl Ray {
    fn new(origin: Vector3, direction: Vector3) -> Self {
        Self { origin, direction, chroma: Chroma::White }
    }

    fn new_chroma(origin: Vector3, direction: Vector3, chroma: Chroma) -> Self {
        Self { origin, direction, chroma }
    }

    fn at(&self, t: Fpr) -> Vector3 {
        self.origin + t * self.direction
    }
}

struct Hit<'a> {
    t: Fpr,
    position: Vector3,
    normal: Vector3,
    material: &'a dyn Material,
}

mod materials;
use materials::{Material,Diffuse,Metal,Dielectric,DispersiveDielectric};

mod camera;
use camera::{Camera,ViewPort};

mod geometry;
use geometry::{Sphere,Plane,RenderedBody};

mod scene;
use scene::{Scene};

fn background_colour(ray: &Ray) -> Colour {
    let dirnorm = ray.direction.normalize();
    let t = 0.5 * (dirnorm.y + 1.0);
    (1.0-t)*Colour::new(1.0,1.0,1.0) + t*Colour::new(0.5,0.7,1.0)
}

fn get_ray_colour(ray: &Ray, scene: &Scene, depth: u32) -> Colour {

    if depth <= 0 {
        return Colour::zeros();
    }

    if let Some(hit) = scene.get_hit(ray,None,None) {
        let mut colour = Colour::zeros();
        
        let scattered_rays = hit.material.response(ray,&hit);
        for (attenuation,scattered_ray) in scattered_rays {
            colour += get_ray_colour(&scattered_ray, scene, depth-1).component_mul(&attenuation)
        }
        
        colour
    }
    else {
        background_colour(ray)
    }

}

fn main() -> io::Result<()> {
    // Image configuration
    const ASPECT_RATIO: Fpr = 16.0 / 9.0;
    const IMAGE_WIDTH: u32 = 1920/2;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as Fpr / ASPECT_RATIO) as u32;

    // World configuration

    // Materials
    let diffuse_red = Diffuse::new(Colour::new(1.0,0.0,0.0));
    let diffuse_ground = Diffuse::new(Colour::new(0.2,0.8,0.2));
    let diffuse_blue = Diffuse::new(Colour::new(0.0,0.0,1.0));
    let metal = Metal::new(Colour::new(0.8,0.8,0.8));
    let dirty_metal = Metal::new(Colour::new(0.5,0.5,0.5));
    let glass = Dielectric::new(Colour::new(0.9,0.8,0.9),1.5);
    const RED_REFRAC: Fpr = 1.5;
    let disp_glass = DispersiveDielectric::new(Colour::new(0.9,0.9,0.9),[RED_REFRAC,RED_REFRAC*0.97,RED_REFRAC*0.96]);

    // Bodies
    let centre_sph = Sphere::new(Vector3::new(0.0,0.0,-1.1), 0.2, &disp_glass);
    let right_sph = Sphere::new(Vector3::new(1.0,0.0,-1.0), 0.5, &metal);
    let left_sph = Sphere::new(Vector3::new(-0.8,0.0,-1.8), 0.5, &diffuse_red);
    let world_sph = Sphere::new(Vector3::new(0.0,-100.5,-1.0), 100.0, &diffuse_ground);
    let mirror = Plane::new(Vector3::new(-1.8,-0.5,0.0),Vector3::new(0.0,0.0,-1.0),Vector3::new(0.0,1.0,0.0),[5.0,1.0],&dirty_metal);

    // Camera configuration
    const VIEWPORT_HEIGHT: Fpr = 2.0;
    let viewport = ViewPort::new(ASPECT_RATIO,VIEWPORT_HEIGHT);
    
    const FOCAL_LENGTH: Fpr = 1.0;
    const CAMERA_ORIGIN: Vector3 = Vector3::new(0.0,0.0,0.0);
    const CAMERA_HORIZ: Vector3 = Vector3::new(1.0,0.0,0.0);
    const CAMERA_VERT: Vector3 = Vector3::new(0.0,1.0,0.0);
    let camera = Camera::new(CAMERA_ORIGIN,FOCAL_LENGTH,viewport,CAMERA_HORIZ,CAMERA_VERT);


    let bodies: Vec<&dyn RenderedBody> = vec![
        &centre_sph, &right_sph, &left_sph, &world_sph, &mirror
    ]; 
    let scene = Scene {
        bodies,
        camera
    };

    // Actual rendering
    let img = output::Image::new(IMAGE_WIDTH,IMAGE_HEIGHT);
    
    use std::sync::{Arc,Mutex};
    let imgmutex = Arc::new(Mutex::new(img));

    const PIXEL_SAMPLES: u32 = 256;
    const MAX_DEPTH: u32 = 16;
    const NTHREADS: u32 = 8;

    
    let _ = crossbeam::thread::scope(|s| {
        let mut threads = vec![];
        
        for t in 0..NTHREADS {
            let scene = scene.clone();
            let imgmutex = imgmutex.clone();
            
            let join_handle = s.spawn(move |_| {
                let mut rng = rand::thread_rng();

                for j in 0..IMAGE_HEIGHT {
                    if j % NTHREADS != t { continue; }

                    for i in 0..IMAGE_WIDTH {
                            
                        let mut colour = Colour::zeros();
                        for _ in 0..PIXEL_SAMPLES {
                            let image_x = i as Fpr + rng.gen::<Fpr>();
                            let image_y = (IMAGE_HEIGHT-1-j) as Fpr + rng.gen::<Fpr>();

                            let u = image_x / (IMAGE_WIDTH-1) as Fpr;
                            let v = image_y / (IMAGE_HEIGHT-1) as Fpr;

                            let ray = camera.get_ray(u, v);
                            colour += get_ray_colour(&ray,&scene,MAX_DEPTH);
                        }
                        
                        let mut img = imgmutex.lock().unwrap();
                        img.set_pixel(i, j, output::PixelColour::from(colour/PIXEL_SAMPLES as Fpr).into() );
                    }
                }
            });
            threads.push(join_handle);
        }

        for thread in threads {
            thread.join();
        }
    });

    let img = imgmutex.lock().unwrap();
    img.save("img.bmp")
}
