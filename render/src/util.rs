use nalgebra_glm as glm;

macro_rules! clip {
    ($left:expr, $right:expr, $comp:ident, $c:expr) => {
        let t = ($c - $left.$comp) / ($right.$comp - $left.$comp);
        if t > 0.0 && t < 1.0 {
            let clip = glm::lerp($left, $right, t);
            if $left.$comp < $c {
                *$left = clip;
            } else {
                *$right = clip;
            }
        }
    };
    (@, $left:expr, $right:expr, $comp:ident, $c:expr) => {
        let t = ($c - $left.$comp) / ($right.$comp - $left.$comp);
        if t > 0.0 && t < 1.0 {
            let clip = glm::lerp($left, $right, t);
            if $left.$comp > $c {
                *$left = clip;
            } else {
                *$right = clip;
            }
        }
    };
}

pub fn clip_xy(left: &mut glm::Vec3, right: &mut glm::Vec3, eps: f32) {
    clip!(left, right, y, eps); // y=0
    clip!(@, left, right, y, 1.0 - eps); // y=1
    clip!(left, right, x, eps - 1.0); // x=-1
    clip!(@, left, right, x, 1.0 - eps); // x=1
}

pub fn clip_y(left: &mut glm::DVec4, right: &mut glm::DVec4, eps: f64) {
    clip!(left, right, y, eps); // y=0
}

pub fn clip_x(left: &mut glm::DVec4, right: &mut glm::DVec4, eps: f64) {
    clip!(left, right, x, eps - 1.0); // x=-1
    clip!(@, left, right, x, 1.0 - eps); // x=1
}
