use rand::prelude::*;

use crate::{Fpr,Vector3};
use crate::{Ray,Hit,Colour};

pub(crate) trait Material : Sync {
    fn response(&self, ray: &Ray, hit: &Hit) -> (Colour,Option<Ray>);
}

pub(crate) struct Diffuse {
    colour: Colour,
}

impl Diffuse {
    pub(crate) fn new(colour: Colour) -> Self {
        Diffuse {
            colour
        }
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

impl Material for Diffuse {
    fn response(&self, _ray: &Ray, hit: &Hit) -> (Colour,Option<Ray>) {
        let target = hit.position + hit.normal + random_in_sphere();
        (self.colour, Some(Ray::new(hit.position,target-hit.position)))
    }
}

pub(crate) struct Metal {
    colour: Colour,
}

impl Metal {
    pub(crate) fn new(colour: Colour) -> Self {
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

pub(crate) struct Dielectric {
    colour: Colour,
    refractive_index: Fpr,
}

impl Dielectric {
    pub(crate) fn new(colour: Colour, refractive_index: Fpr) -> Self {
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