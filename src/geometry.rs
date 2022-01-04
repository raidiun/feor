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


pub(crate) struct Plane<'a> {
    origin: Vector3,
    x: Vector3,
    y: Vector3,
    extents: [Fpr;2],
    material: &'a dyn Material,
    normal: Vector3,
}

impl<'a> Plane<'a> {
	pub fn new(origin: Vector3, x: Vector3, y: Vector3, extents: [Fpr;2], material: &'a dyn Material) -> Self {
		Self {
			origin,
			x,
            y,
            extents,
			material,
            normal: x.cross(&y),
		}
	}

}

impl<'a> RenderedBody<'a> for Plane<'a> {
    fn get_hit(&self, ray: &Ray, tmin_opt: Option<Fpr>, tmax_opt: Option<Fpr>) -> Option<Hit> {
        let ln = self.normal.dot(&ray.direction);

        if ln.abs() < 0.0001 {
            // No sensible intersect
            return None;
        }

        let t = (self.origin - ray.origin).dot(&self.normal) / ln;

        let tmin = tmin_opt.unwrap_or(0.0001);
        let tmax = tmax_opt.unwrap_or(Fpr::INFINITY);

        if t < tmin || t > tmax {
            return None;
        }

        let p_intersect = ray.at(t);

        let inplane = p_intersect - self.origin;

        let x = inplane.dot(&self.x);
        let y = inplane.dot(&self.y);

        if 0.0 < x && x < self.extents[0] && 0.0 < y && y < self.extents[1] {
            // Within bounds
            Some( Hit {
                t,
                position: p_intersect,
                normal: self.normal,
                material: self.get_material(),
            })
        }
        else {
            None
        }   
    }

    fn get_material(&self) -> &'a dyn Material {
        self.material
    }

}
