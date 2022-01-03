use crate::{Fpr,Ray,Hit};

use crate::geometry::RenderedBody;
use crate::camera::Camera;

#[derive(Clone)]
pub(crate) struct Scene<'a> {
    pub bodies: Vec<&'a dyn RenderedBody<'a>>,
    pub camera: Camera,
}

impl<'a> Scene<'a> {
    pub(crate) fn get_hit(&self, ray: &Ray, tmin: Option<Fpr>, tmax: Option<Fpr>) -> Option<Hit> {
        let mut closest = tmax.unwrap_or(Fpr::INFINITY);
        let mut hit = None;

        for hittable in &self.bodies {
            if let Some(h) = hittable.get_hit(ray,tmin,Some(closest)) {
                closest = h.t;
                hit = Some(h);
            }
        }

        hit
    }
}
