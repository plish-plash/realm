use building_blocks::{core::sdfu::{SDF, Line, ops::{MinFunction, PolySmoothMin}}, prelude::Point3f};

/// The union of multiple line SDFs, for constructing flora.
#[derive(Clone, Copy, Debug)]
pub struct LineUnionList<'a> {
    sdf_list: &'a [Line<f32, Point3f>],
    min_func: PolySmoothMin<f32>,
}


impl<'a> LineUnionList<'a> {
    pub fn new(sdf_list: &'a [Line<f32, Point3f>], smoothness: f32) -> Self {
        LineUnionList {
            sdf_list,
            min_func: PolySmoothMin::new(smoothness),
        }
    }
}

impl<'a> SDF<f32, Point3f> for LineUnionList<'a> {
    fn dist(&self, p: Point3f) -> f32 {
        let mut dist = None;
        for sdf in self.sdf_list {
            let val = if let Some(dist) = dist {
                self.min_func.min(dist, sdf.dist(p))
            } else {
                sdf.dist(p)
            };
            dist = Some(val);
        }
        dist.expect("no SDFs in list")
    }
}