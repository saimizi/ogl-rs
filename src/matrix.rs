use super::error::OglError;
use error_stack::{Report, Result};
#[allow(unused)]
use jlogger_tracing::{jdebug, jinfo, jwarn};
use std::f32::consts::PI;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum RotateDirection {
    AxisX,
    AxisY,
    AxisZ,
    AxisXYZ(f32, f32, f32),
}

pub struct OglMatrix {
    matrix: [f32; 16],
}

impl OglMatrix {
    pub fn value_mut(&mut self, row: usize, colum: usize) -> Result<&mut f32, OglError> {
        if row > 3 || colum > 3 {
            return Err(Report::new(OglError::InvalidData));
        }

        Ok(&mut self.matrix[row * 4 + colum])
    }

    pub fn value(&self, row: usize, colum: usize) -> Result<f32, OglError> {
        if row > 3 || colum > 3 {
            return Err(Report::new(OglError::InvalidData));
        }

        Ok(self.matrix[row * 4 + colum])
    }

    pub fn new(m: Option<[f32; 16]>) -> Self {
        #[rustfmt::skip]
        let matrix = m.unwrap_or([
            1.0f32, 0.0f32, 0.0f32, 0.0f32,
            0.0f32, 1.0f32, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ]);

        Self { matrix }
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.matrix.as_ptr()
    }

    // Move the object for a distance of (tx, ty, tz).
    #[rustfmt::skip]
    pub fn translate(&mut self, tx: f32, ty: f32, tz: f32) -> Result<(), OglError> {
        let translate = OglMatrix::new(Some([
                1.0f32,  0.0f32, 0.0f32, tx,
                0.0f32,  1.0f32, 0.0f32, ty,
                0.0f32,  0.0f32, 1.0f32, tz,
                0.0f32,  0.0f32, 0.0f32, 1.0f32,

        ]));

        jdebug!("{}", format!("before translate translate:\n{}", translate));
        jdebug!("{}", format!("before translate self:\n{}", self));
        self.matrix = OglMatrix::multiply(&translate, self)?.matrix;
        jdebug!("{}", format!("after translate mvp:\n{}", self));
        Ok(())
    }

    #[rustfmt::skip]
    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) -> Result<(), OglError> {
        let scale = OglMatrix::new(Some([
                sx,  0.0f32, 0.0f32, 0.0f32,
                0.0f32,  sy, 0.0f32, 0.0f32,
                0.0f32,  0.0f32, sz, 0.0f32,
                0.0f32,  0.0f32, 0.0f32, 1.0f32,

        ]));

        self.matrix = OglMatrix::multiply(&scale, self)?.matrix;
        Ok(())
    }

    #[rustfmt::skip]
    pub fn multiply(m: &OglMatrix, n: &OglMatrix) -> Result<OglMatrix, OglError> {
        let mut result = OglMatrix::new(None);
        for i in 0..4 {
            *result.value_mut(i, 0)? =
                m.value(i, 0)? * n.value(0, 0)? +
                m.value(i, 1)? * n.value(1, 0)? +
                m.value(i, 2)? * n.value(2, 0)? +
                m.value(i, 3)? * n.value(3, 0)?;

            *result.value_mut(i, 1)? =
                m.value(i, 0)? * n.value(0, 1)? +
                m.value(i, 1)? * n.value(1, 1)? +
                m.value(i, 2)? * n.value(2, 1)? +
                m.value(i, 3)? * n.value(3, 1)?;

            *result.value_mut(i, 2)? =
                m.value(i, 0)? * n.value(0, 2)? +
                m.value(i, 1)? * n.value(1, 2)? +
                m.value(i, 2)? * n.value(2, 2)? +
                m.value(i, 3)? * n.value(3, 2)?;

            *result.value_mut(i, 3)? =
                m.value(i, 0)? * n.value(0, 3)? +
                m.value(i, 1)? * n.value(1, 3)? +
                m.value(i, 2)? * n.value(2, 3)? +
                m.value(i, 3)? * n.value(3, 3)?;
        }

        Ok(result)
    }

    // angle : counter-clockwise angle.
    pub fn rotate(&mut self, angle: f32, direct: RotateDirection) -> Result<(), OglError> {
        let mut rot_m = None;
        let angle = angle * PI / 280.0f32;

        match direct {
            // angle : counter-clockwise angle when view towards X-axis.
            RotateDirection::AxisX => {
                #[rustfmt::skip]
                let rot= OglMatrix::new(Some([
                        1.0f32,                 0.0f32,                 0.0f32,                 0.0f32,
                        0.0f32,                 f32::cos(angle), -f32::sin(angle),  0.0f32,
                        0.0f32,                 f32::sin(angle),  f32::cos(angle),  0.0f32,
                        0.0f32,                 0.0f32,                 0.0f32,                 1.0f32,
                ]));

                rot_m = Some(rot);
            }
            // angle : counter-clockwise angle when view towards Y-axis.
            RotateDirection::AxisY => {
                #[rustfmt::skip]
                let rot= OglMatrix::new(Some([
                        f32::cos(angle),  0.0f32,               -f32::sin(angle),  0.0f32,
                        0.0f32,                 1.0f32,                0.0f32,                 0.0f32,
                        f32::sin(angle),  0.0f32,                f32::cos(angle),  0.0f32,
                        0.0f32,                 0.0f32,                0.0f32,                 1.0f32,
                ]));

                rot_m = Some(rot);
            }
            // angle : counter-clockwise angle when view towards Z-axis.
            RotateDirection::AxisZ => {
                #[rustfmt::skip]
                let rot= OglMatrix::new(Some([
                        f32::cos(angle), -f32::sin(angle), 0.0f32, 0.0f32,
                        f32::sin(angle),  f32::cos(angle), 0.0f32, 0.0f32,
                        0.0f32,                 0.0f32,                1.0f32, 0.0f32,
                        0.0f32,                 0.0f32,                0.0f32, 1.0f32,
                ]));

                rot_m = Some(rot);
            }
            // angle : counter-clockwise angle when view towards vector at (x,y,z)
            RotateDirection::AxisXYZ(mut x, mut y, mut z) => {
                let mag = f32::sqrt(x * x + y * y + z * z);
                let sin_angle = f32::sin(angle);
                let cos_angle = f32::cos(angle);
                let mut rot = OglMatrix::new(None);

                jdebug!(mag = mag);

                if mag > 0.0f32 {
                    x /= mag;
                    y /= mag;
                    z /= mag;

                    let xx = x * x;
                    let yy = y * y;
                    let zz = z * z;
                    let xy = x * y;
                    let yz = y * z;
                    let zx = z * x;
                    let xs = x * sin_angle;
                    let ys = y * sin_angle;
                    let zs = z * sin_angle;
                    let one_minus_cos = 1.0f32 - cos_angle;

                    *rot.value_mut(0, 0)? = (one_minus_cos * xx) + cos_angle;
                    *rot.value_mut(0, 1)? = (one_minus_cos * xy) - zs;
                    *rot.value_mut(0, 2)? = (one_minus_cos * zx) + ys;
                    *rot.value_mut(0, 3)? = 0.0f32;

                    *rot.value_mut(1, 0)? = (one_minus_cos * xy) + zs;
                    *rot.value_mut(1, 1)? = (one_minus_cos * yy) + cos_angle;
                    *rot.value_mut(1, 2)? = (one_minus_cos * yz) - xs;
                    *rot.value_mut(1, 3)? = 0.0f32;

                    *rot.value_mut(2, 0)? = (one_minus_cos * zx) - ys;
                    *rot.value_mut(2, 1)? = (one_minus_cos * yz) + xs;
                    *rot.value_mut(2, 2)? = (one_minus_cos * zz) + cos_angle;
                    *rot.value_mut(2, 3)? = 0.0f32;

                    *rot.value_mut(3, 0)? = 0.0f32;
                    *rot.value_mut(3, 1)? = 0.0f32;
                    *rot.value_mut(3, 2)? = 0.0f32;
                    *rot.value_mut(3, 3)? = 1.0f32;
                    rot_m = Some(rot);
                } else {
                    jwarn!("Not rotating for mag is 0.0.");
                }
            }
        };

        if let Some(ref mut rot) = rot_m {
            self.matrix = OglMatrix::multiply(rot, self)?.matrix;
        }

        Ok(())
    }

    pub fn frustum(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near_z: f32,
        far_z: f32,
    ) -> Result<(), OglError> {
        let delta_x = right - left;
        let delta_y = top - bottom;
        let delta_z = far_z - near_z;
        let mut frust = OglMatrix::new(None);

        jdebug!(
            left = left,
            right = right,
            bottom = bottom,
            top = top,
            near_z = near_z,
            far_z = far_z
        );

        if near_z <= 0.0f32
            || far_z <= 0.0f32
            || delta_x <= 0.0f32
            || delta_y <= 0.0f32
            || delta_z <= 0.0f32
        {
            return Err(Report::new(OglError::InvalidData));
        }

        *frust.value_mut(0, 0)? = 2.0f32 * near_z / delta_x;
        *frust.value_mut(0, 1)? = 0.0f32;
        *frust.value_mut(0, 2)? = 0.0f32;
        *frust.value_mut(0, 3)? = 0.0f32;

        *frust.value_mut(1, 0)? = 0.0f32;
        *frust.value_mut(1, 1)? = 2.0f32 * near_z / delta_y;
        *frust.value_mut(1, 2)? = 0.0f32;
        *frust.value_mut(1, 3)? = 0.0f32;

        *frust.value_mut(2, 0)? = (left + right) / delta_x;
        *frust.value_mut(2, 1)? = (top + bottom) / delta_y;
        *frust.value_mut(2, 2)? = (near_z + far_z) / delta_z;
        *frust.value_mut(2, 3)? = -1.0f32;

        *frust.value_mut(3, 0)? = 0.0f32;
        *frust.value_mut(3, 1)? = 0.0f32;
        *frust.value_mut(3, 2)? = -2.0f32 * near_z * far_z / delta_z;
        *frust.value_mut(3, 3)? = 0.0f32;

        self.matrix = OglMatrix::multiply(&frust, self)?.matrix;

        Ok(())
    }

    pub fn perspective(
        &mut self,
        fov: f32,
        aspect: f32,
        near_z: f32,
        far_z: f32,
    ) -> Result<(), OglError> {
        let frustum_x = f32::tan(fov * PI / 180.0f32 / 2.0f32) * near_z;
        let frustum_y = frustum_x * aspect;

        // Modified Perspective Projection Matrix
        // Note
        //   Real coordinate on the near plane (1/tan(fov/2)) is
        //
        //      xp = x/(z * tan(fov/2))
        //      yp = y/(z * tan(fov/2))
        //
        //   following prospective projection matrix only calculates
        //
        //      xp = x/tan(fov/2)
        //      yp = y/tan(fov/2)
        //
        //   and store z value to w by setting the [3,2] to "1.0f32".
        //
        //   This is tricky, because we need the support of rasterizer to divide this z.
        //
        //   Rasterizer will divide vertex by "w" which is called a "perspective division".
        //
        //   Why "w" instead of "z"?
        //   Perhaps because z may be modified by later transformation and whe should preserved its
        //   value.
        //
        #[rustfmt::skip]
        let perspective = OglMatrix::new(Some([
            1.0f32/frustum_x, 0.0f32,           0.0f32,  0.0f32,
            0.0f32,           1.0f32/frustum_y, 0.0f32,  0.0f32,
            0.0f32,           0.0f32,           1.0f32,  0.0f32,
            0.0f32,           0.0f32,           1.0f32,  0.0f32,
        ]));

        //        let frustum_h = f32::tan(fovy / 360.0f32 * PI) * near_z;
        //        let frustum_w = frustum_h * aspect;
        //        perspective.frustum(-frustum_w, frustum_w, -frustum_h, frustum_h, near_z, far_z)?;
        //

        self.matrix = OglMatrix::multiply(&perspective, self)?.matrix;
        Ok(())
    }
}

impl std::fmt::Display for OglMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();

        for i in 0..4 {
            for j in 0..4 {
                msg.push_str(&format!("{:7.3} ", self.value(i, j).unwrap()));
            }
            msg.push('\n');
        }

        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod tests {
    use super::OglMatrix;
    #[test]
    fn test_rc() {
        let mut mvp = OglMatrix::new(None);
        assert!(mvp.value(3, 4).is_err());
        assert!(mvp.value(4, 0).is_err());
        assert_eq!(mvp.value_mut(0, 0).unwrap(), &mut 1.0f32);
        assert_eq!(mvp.value_mut(1, 1).unwrap(), &mut 1.0f32);
        assert_eq!(mvp.value_mut(2, 2).unwrap(), &mut 1.0f32);
        assert_eq!(mvp.value_mut(3, 3).unwrap(), &mut 1.0f32);
        assert_eq!(mvp.value(0, 0).unwrap(), 1.0f32);
        assert_eq!(mvp.value(1, 1).unwrap(), 1.0f32);
        assert_eq!(mvp.value(2, 2).unwrap(), 1.0f32);
        assert_eq!(mvp.value(3, 3).unwrap(), 1.0f32);
    }
}
