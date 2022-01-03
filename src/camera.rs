use crate::{Fpr,Vector3,Ray};

#[derive(Copy,Clone)]
pub(crate) struct ViewPort {
    aspect_ratio: Fpr,
    width: Fpr,
    height: Fpr,
}

impl ViewPort {
    pub(crate) fn new(aspect_ratio: Fpr, height: Fpr) -> Self {
        let width = aspect_ratio * height;
        
        ViewPort {
            aspect_ratio,
            width,
            height,
        }
    }
}

#[derive(Copy,Clone)]
pub(crate) struct Camera {
    origin: Vector3,
    focal_length: Fpr,
    viewport: ViewPort,
    horizontal: Vector3,
    vertical: Vector3,
    image_origin: Vector3,
}

impl Camera {
    pub(crate) fn new(origin: Vector3, focal_length: Fpr, viewport: ViewPort, horizontal: Vector3, vertical: Vector3) -> Self {
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

    pub(crate) fn get_ray(&self, u: Fpr, v: Fpr) -> Ray {
        Ray::new(self.origin,self.image_origin + u*self.horizontal + v*self.vertical - self.origin)
    }
}
