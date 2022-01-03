use crate::{Fpr,Vector3,Ray,Hit,Material};

pub(crate) trait RenderedBody<'a> : Sync {
    fn get_hit(&'a self, ray: &Ray, tmin: Option<Fpr>, tmax: Option<Fpr>) -> Option<Hit>;
    fn get_material(&self) -> &'a dyn Material;
}

pub(crate) struct Sphere<'a> {
    centre: Vector3,
    radius: Fpr,
    material: &'a dyn Material,
}

impl<'a> Sphere<'a> {
	pub fn new(centre: Vector3, radius: Fpr, material: &'a dyn Material) -> Self {
		Self {
			centre,
			radius,
			material,
		}
	}

}

impl<'a> RenderedBody<'a> for Sphere<'a> {
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

    fn get_material(&self) -> &'a dyn Material {
        self.material
    }

}