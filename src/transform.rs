use cgmath::{Angle, Euler, One, Rad, Zero, num_traits::cast};

// pub type Vector2f = cgmath::Vector2<f32>;
pub type Vector3f = cgmath::Vector3<f32>;
// pub type Vector2i = cgmath::Vector2<i32>;
// pub type Vector3i = cgmath::Vector3<i32>;
// pub type Point2f = cgmath::Point2<f32>;
pub type Point3f = cgmath::Point3<f32>;
// pub type Point2i = cgmath::Point2<i32>;
// pub type Point3i = cgmath::Point3<i32>;
pub type Quaternion = cgmath::Quaternion<f32>;
pub type Transform = cgmath::Decomposed<Vector3f, Quaternion>;

pub trait TransformExtensions {
    fn identity() -> Transform {
        Transform {
            disp: Vector3f::zero(),
            rot: Quaternion::one(),
            scale: 1.0,
        }
    }
    fn new(disp: Vector3f, rot: Quaternion, scale: f32) -> Transform {
        Transform { disp, rot, scale }
    }
    fn from_translation(disp: Vector3f) -> Transform {
        Transform {
            disp,
            rot: Quaternion::one(),
            scale: 1.0,
        }
    }
    fn from_rotation(rot: Quaternion) -> Transform {
        Transform {
            disp: Vector3f::zero(),
            rot,
            scale: 1.0,
        }
    }
}

impl TransformExtensions for Transform {}

pub trait QuaternionExtensions<A> where A: Angle + Into<Rad<<A as Angle>::Unitless>> {
    // For some ungodly reason Quaternion::from applies Euler angles in XYZ order, which isn't useful.
    fn from_euler_yxz(angles: Euler<A>) -> cgmath::Quaternion<A::Unitless> {
        let half = cast(0.5f64).unwrap();
        let (s1, c1) = Rad::sin_cos(angles.y.into() * half);
        let (s2, c2) = Rad::sin_cos(angles.z.into() * half);
        let (s3, c3) = Rad::sin_cos(angles.x.into() * half);
        cgmath::Quaternion::new(
            c1*c2*c3 - s1*s2*s3,
            s1*s2*c3 + c1*c2*s3,
            s1*c2*c3 + c1*s2*s3,
            c1*s2*c3 - s1*c2*s3,
        )
    }
}

impl<A> QuaternionExtensions<A> for cgmath::Quaternion<A> where A: Angle + Into<Rad<<A as Angle>::Unitless>> {}
