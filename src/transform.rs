use cgmath::{Angle, Decomposed, Euler, One, Quaternion, Rad, Vector3, Zero, num_traits::cast};

pub type Transform = Decomposed<Vector3<f32>, Quaternion<f32>>;

pub trait TransformExtensions {
    fn identity() -> Transform {
        Transform {
            disp: Vector3::zero(),
            rot: Quaternion::one(),
            scale: 1.0,
        }
    }
    fn new(disp: Vector3<f32>, rot: Quaternion<f32>, scale: f32) -> Transform {
        Transform { disp, rot, scale }
    }
    fn from_translation(disp: Vector3<f32>) -> Transform {
        Transform {
            disp,
            rot: Quaternion::one(),
            scale: 1.0,
        }
    }
    fn from_rotation(rot: Quaternion<f32>) -> Transform {
        Transform {
            disp: Vector3::zero(),
            rot,
            scale: 1.0,
        }
    }
}

impl TransformExtensions for Transform {}

pub trait QuaternionExtensions<A> where A: Angle + Into<Rad<<A as Angle>::Unitless>> {
    // For some ungodly reason Quaternion::from applies Euler angles in XYZ order, which isn't useful.
    fn from_euler_yxz(angles: Euler<A>) -> Quaternion<A::Unitless> {
        let half = cast(0.5f64).unwrap();
        let (s1, c1) = Rad::sin_cos(angles.y.into() * half);
        let (s2, c2) = Rad::sin_cos(angles.z.into() * half);
        let (s3, c3) = Rad::sin_cos(angles.x.into() * half);
        Quaternion::new(
            c1*c2*c3 - s1*s2*s3,
            s1*s2*c3 + c1*c2*s3,
            s1*c2*c3 + c1*s2*s3,
            c1*s2*c3 - s1*c2*s3,
        )
    }
}

impl<A> QuaternionExtensions<A> for Quaternion<A> where A: Angle + Into<Rad<<A as Angle>::Unitless>> {}
