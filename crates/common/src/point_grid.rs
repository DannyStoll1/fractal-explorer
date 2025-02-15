use crate::types::{Cplx, Real};
use ndarray::Array2;
use rayon::iter::{IterBridge, ParallelBridge};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bounds
{
    pub min_x: Real,
    pub max_x: Real,
    pub min_y: Real,
    pub max_y: Real,
}
impl Bounds
{
    #[inline]
    #[must_use]
    pub const fn range_x(&self) -> Real
    {
        self.max_x - self.min_x
    }

    #[inline]
    #[must_use]
    pub const fn range_y(&self) -> Real
    {
        self.max_y - self.min_y
    }

    #[inline]
    #[must_use]
    pub const fn area(&self) -> Real
    {
        self.range_x() * self.range_y()
    }

    #[inline]
    #[must_use]
    pub const fn aspect_ratio(&self) -> Real
    {
        self.range_y() / self.range_x()
    }

    #[inline]
    #[must_use]
    pub const fn mid_x(&self) -> Real
    {
        (self.max_x + self.min_x) / 2.
    }

    #[inline]
    #[must_use]
    pub const fn mid_y(&self) -> Real
    {
        (self.max_y + self.min_y) / 2.
    }

    pub fn translate(&mut self, translation: Cplx)
    {
        self.min_x += translation.re;
        self.max_x += translation.re;
        self.min_y += translation.im;
        self.max_y += translation.im;
    }

    pub fn zoom(&mut self, scale: Real, base_point: Cplx)
    {
        self.translate(-base_point);
        self.min_x *= scale;
        self.max_x *= scale;
        self.min_y *= scale;
        self.max_y *= scale;
        self.translate(base_point);
    }

    #[inline]
    #[must_use]
    pub const fn center(&self) -> Cplx
    {
        let re = self.mid_x();
        let im = self.mid_y();
        Cplx::new(re, im)
    }

    #[inline]
    pub fn recenter(&mut self, new_center: Cplx)
    {
        let old_center = self.center();
        self.translate(new_center - old_center);
    }

    #[must_use]
    pub const fn centered_square(radius: Real) -> Self
    {
        Self {
            min_x: -radius,
            max_x: radius,
            min_y: -radius,
            max_y: radius,
        }
    }

    #[must_use]
    pub const fn square(radius: Real, center: Cplx) -> Self
    {
        Self::rect(radius, radius, center)
    }

    #[must_use]
    pub const fn rect(radius_x: Real, radius_y: Real, center: Cplx) -> Self
    {
        Self {
            min_x: -radius_x + center.re,
            max_x: radius_x + center.re,
            min_y: -radius_y + center.im,
            max_y: radius_y + center.im,
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn is_nan(&self) -> bool
    {
        self.min_x.is_nan() || self.max_x.is_nan() || self.min_y.is_nan() || self.max_y.is_nan()
    }
}

impl Default for Bounds
{
    fn default() -> Self
    {
        Self {
            min_x: -1.,
            max_x: 1.,
            min_y: -1.,
            max_y: 1.,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PointGrid
{
    pub res_x: usize,
    pub res_y: usize,
    pub bounds: Bounds,
}

impl PointGrid
{
    #[must_use]
    pub const fn new(res_x: usize, res_y: usize, bounds: Bounds) -> Self
    {
        Self {
            res_x,
            res_y,
            bounds,
        }
    }

    #[must_use]
    #[allow(clippy::similar_names)]
    #[allow(clippy::cast_sign_loss)]
    pub const fn infer_height(res_x: usize, bounds: &Bounds) -> usize
    {
        debug_assert!(res_x > 0);

        let res_x_float = res_x as Real;
        let res_y_float =
            res_x_float * (bounds.max_y - bounds.min_y) / (bounds.max_x - bounds.min_x);
        res_y_float as usize
    }

    #[must_use]
    #[allow(clippy::similar_names)]
    #[allow(clippy::cast_sign_loss)]
    pub const fn infer_width(res_y: usize, bounds: &Bounds) -> usize
    {
        debug_assert!(res_y > 0);

        let res_y_float = res_y as Real;
        let res_x_float =
            res_y_float * (bounds.max_x - bounds.min_x) / (bounds.max_y - bounds.min_y);
        res_x_float as usize
    }

    #[must_use]
    pub const fn new_by_res_x(res_x: usize, bounds: Bounds) -> Self
    {
        let res_y = Self::infer_height(res_x, &bounds);

        Self::new(res_x, res_y, bounds)
    }

    #[must_use]
    pub const fn new_by_res_y(res_y: usize, bounds: Bounds) -> Self
    {
        let res_x = Self::infer_width(res_y, &bounds);

        Self::new(res_x, res_y, bounds)
    }

    #[must_use]
    pub const fn new_with_same_height(&self, bounds: Bounds) -> Self
    {
        Self::new_by_res_y(self.res_y, bounds)
    }

    #[must_use]
    pub const fn new_with_same_width(&self, bounds: Bounds) -> Self
    {
        Self::new_by_res_x(self.res_x, bounds)
    }

    #[inline]
    #[must_use]
    pub const fn with_same_height(self, bounds: Bounds) -> Self
    {
        Self::new_by_res_y(self.res_y, bounds)
    }

    #[inline]
    #[must_use]
    pub const fn with_same_width(self, bounds: Bounds) -> Self
    {
        Self::new_by_res_x(self.res_x, bounds)
    }

    #[inline]
    #[must_use]
    pub const fn with_width(self, res_x: usize) -> Self
    {
        Self::new_by_res_x(res_x, self.bounds)
    }

    #[inline]
    #[must_use]
    pub const fn with_height(self, res_y: usize) -> Self
    {
        Self::new_by_res_y(res_y, self.bounds)
    }

    #[must_use]
    pub fn map_pixel(&self, pixel_x: usize, pixel_y: usize) -> Cplx
    {
        let re = (pixel_x as Real).mul_add(self.pixel_width(), self.bounds.min_x);
        let im = (pixel_y as Real).mul_add(self.pixel_height(), self.bounds.min_y);
        Cplx::new(re, im)
    }

    #[must_use]
    pub fn map_pos(&self, pos: [f32; 2]) -> Cplx
    {
        let re = f64::from(pos[0]).mul_add(self.pixel_width(), self.bounds.min_x);
        let im = f64::from(pos[1]).mul_add(-self.pixel_height(), self.bounds.max_y);
        Cplx::new(re, im)
    }

    #[must_use]
    pub fn map_vec2(&self, vec2: [f32; 2]) -> Cplx
    {
        let re = f64::from(vec2[0]) * self.pixel_width();
        let im = -f64::from(vec2[1]) * self.pixel_height();
        Cplx::new(re, im)
    }

    #[inline]
    #[must_use]
    pub fn pixel_width(&self) -> Real
    {
        self.bounds.range_x() / self.res_x as Real
    }

    #[inline]
    #[must_use]
    pub fn pixel_height(&self) -> Real
    {
        self.bounds.range_y() / self.res_y as Real
    }

    #[inline]
    #[must_use]
    pub const fn shape(&self) -> (usize, usize)
    {
        (self.res_x, self.res_y)
    }

    #[must_use]
    pub fn locate_point(&self, z: Cplx) -> [f32; 2]
    {
        let x = (z.re - self.bounds.min_x) / (self.pixel_width());
        let y = (z.im - self.bounds.min_y) / (self.pixel_height());

        [x as f32, self.res_y as f32 - 1. - y as f32]
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn locate_point_safe(&self, z: Cplx) -> Option<(usize, usize)>
    {
        if z.re >= self.bounds.max_x
            || z.re < self.bounds.min_x
            || z.im >= self.bounds.max_y
            || z.re < self.bounds.min_y
        {
            return None;
        }

        let x = (z.re - self.bounds.min_x) / (self.pixel_width());
        let y = (z.im - self.bounds.min_y) / (self.pixel_height());

        Some((x as usize, self.res_y - 1 - y as usize))
    }

    #[inline]
    #[must_use]
    pub const fn center(&self) -> Cplx
    {
        let re = self.bounds.mid_x();
        let im = self.bounds.mid_y();
        Cplx::new(re, im)
    }

    pub fn recenter(&mut self, new_center: Cplx)
    {
        let old_center = self.center();
        self.translate(new_center - old_center);
    }

    pub fn change_bounds(&mut self, new_bounds: Bounds)
    {
        self.res_y = Self::infer_height(self.res_x, &new_bounds);
        self.bounds = new_bounds;
    }

    #[inline]
    pub fn resize_x(&mut self, res_x: usize)
    {
        self.res_x = res_x;
        self.res_y = Self::infer_height(res_x, &self.bounds);
    }

    #[inline]
    pub fn resize_y(&mut self, res_y: usize)
    {
        self.res_y = res_y;
        self.res_x = Self::infer_width(res_y, &self.bounds);
    }

    #[must_use]
    pub fn to_array(&self) -> Array2<Cplx>
    {
        let mut points = Array2::zeros((self.res_x, self.res_y));
        let pixel_width = self.pixel_width();
        let pixel_height = self.pixel_height();
        points.indexed_iter_mut().for_each(|((i, j), value)| {
            let re = (i as Real).mul_add(pixel_width, self.bounds.min_x);
            let im = (j as Real).mul_add(pixel_height, self.bounds.min_y);
            *value = Cplx::new(re, im);
        });
        points
    }

    #[must_use]
    pub fn par_iter(&self) -> IterBridge<PointGridIterator>
    {
        self.iter().par_bridge()
    }

    #[must_use]
    pub fn iter(&self) -> PointGridIterator
    {
        PointGridIterator::new(self.res_x, self.res_y, &self.bounds)
    }
}

impl Default for PointGrid
{
    fn default() -> Self
    {
        Self {
            res_x: 256,
            res_y: 256,
            bounds: Bounds::default(),
        }
    }
}

impl Deref for PointGrid
{
    type Target = Bounds;

    fn deref(&self) -> &Self::Target
    {
        &self.bounds
    }
}

impl DerefMut for PointGrid
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.bounds
    }
}

impl IntoIterator for PointGrid
{
    type Item = ((usize, usize), Cplx);
    type IntoIter = PointGridIterator;

    fn into_iter(self) -> PointGridIterator
    {
        PointGridIterator::new(self.res_x, self.res_y, &self.bounds)
    }
}

pub struct PointGridIterator
{
    step_x: Real,
    step_y: Real,
    res_x: usize,
    res_y: usize,
    min_x: Real,
    min_y: Real,
    idx_x: usize,
    idx_y: usize,
}

impl PointGridIterator
{
    #[must_use]
    pub fn new(res_x: usize, res_y: usize, bounds: &Bounds) -> Self
    {
        let step_x = bounds.range_x() / (res_x as Real);
        let step_y = bounds.range_y() / (res_y as Real);

        Self {
            step_x,
            step_y,
            res_x,
            res_y,
            min_x: bounds.min_x,
            min_y: bounds.min_y,
            idx_x: 0,
            idx_y: 0,
        }
    }
}

impl Iterator for PointGridIterator
{
    type Item = ((usize, usize), Cplx);

    fn next(&mut self) -> Option<Self::Item>
    {
        self.idx_x += 1;
        self.idx_y += self.idx_x / self.res_x;

        if self.idx_y == self.res_y {
            return None;
        }

        self.idx_x %= self.res_x;

        let z = Cplx::new(
            (self.idx_x as Real).mul_add(self.step_x, self.min_x),
            (self.idx_y as Real).mul_add(self.step_y, self.min_y),
        );

        Some(((self.idx_x, self.idx_y), z))
    }
}

impl IntoIterator for &PointGrid
{
    type IntoIter = PointGridIterator;
    type Item = ((usize, usize), Cplx);
    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}
