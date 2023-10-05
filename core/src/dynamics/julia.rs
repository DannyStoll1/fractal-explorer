use crate::dynamics::ParameterPlane;
use crate::macros::basic_plane_impl;
use fractal_common::coloring::{algorithms::InteriorColoringAlgorithm, Coloring};
use fractal_common::consts::{ONE, TAU, ZERO};
use fractal_common::globals::{RAY_DEPTH, RAY_SHARPNESS};
use fractal_common::math_utils::newton_until_convergence_d;
use fractal_common::point_grid::{Bounds, PointGrid};
use fractal_common::types::{
    Cplx, EscapeState, NoParam, ParamList, ParamStack, Period, PointInfo, Real,
};

use super::symbolic::OrbitSchema;

#[derive(Clone)]
pub struct JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    pub point_grid: PointGrid,
    pub max_iter: Period,
    pub min_iter: Period,
    pub parent: T,
    pub meta_params: T::MetaParam,
    pub local_param: T::Param,
    pub parent_selection: Cplx,
}

impl<T> JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    #[must_use]
    pub fn new(parent: T, parent_selection: Cplx, _max_iter: Period) -> Self
    {
        let local_param = parent.param_map(parent_selection);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(parent_selection, local_param));
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            min_iter: parent.min_iter(),
            meta_params: parent.get_meta_params(),
            local_param,
            parent_selection,
        }
    }

    pub fn map_and_multiplier_lazy(&self, z: T::Var) -> (T::Var, T::Deriv)
    {
        self.parent.map_and_multiplier(z, self.local_param)
    }
}

impl<T> From<T> for JuliaSet<T>
where
    T: ParameterPlane + Clone,
{
    fn from(parent: T) -> Self
    {
        let parent_selection = parent.default_selection();
        let local_param = parent.param_map(parent_selection);
        let point_grid = parent
            .point_grid()
            .new_with_same_height(parent.default_julia_bounds(parent_selection, local_param));
        Self {
            point_grid,
            parent: parent.clone(),
            max_iter: parent.max_iter(),
            min_iter: parent.min_iter(),
            meta_params: parent.get_meta_params(),
            local_param,
            parent_selection,
        }
    }
}

impl<T> ParameterPlane for JuliaSet<T>
where
    T: ParameterPlane,
{
    type Var = T::Var;
    type Param = NoParam;
    type MetaParam = ParamStack<T::MetaParam, T::Param>;
    type Deriv = T::Deriv;
    type Child = Self;
    basic_plane_impl!();

    #[inline]
    fn map(&self, z: Self::Var, _c: Self::Param) -> Self::Var
    {
        self.parent.map(z, self.local_param)
    }

    #[inline]
    fn dynamical_derivative(&self, z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        self.parent.dynamical_derivative(z, self.local_param)
    }

    #[inline]
    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        Self::Deriv::from(0.0)
        // self.parent.parameter_derivative(z, self.local_param)
    }

    #[inline]
    fn map_and_multiplier(&self, z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.parent.map_and_multiplier(z, self.local_param)
    }

    #[inline]
    fn gradient(&self, z: Self::Var, _c: Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        self.parent.gradient(z, self.local_param)
    }

    #[inline]
    fn min_iter(&self) -> Period
    {
        self.min_iter
    }

    #[inline]
    fn param_map(&self, _z: Cplx) -> Self::Param
    {
        NoParam
    }

    #[inline]
    fn param_map_d(&self, _z: Cplx) -> (Self::Param, Self::Deriv)
    {
        (NoParam, Self::Deriv::from(1.0))
    }

    #[inline]
    fn start_point(&self, point: Cplx, _param: Self::Param) -> Self::Var
    {
        self.parent.dynam_map(point)
    }

    #[inline]
    fn start_point_d(&self, point: Cplx, _param: Self::Param) -> (Self::Var, Self::Deriv)
    {
        self.parent.dynam_map_d(point)
    }

    #[inline]
    fn cycle_active_plane(&mut self)
    {
        self.parent.cycle_active_plane();
    }

    fn encode_escape_result(
        &self,
        state: EscapeState<Self::Var, Self::Deriv>,
        _base_param: Self::Param,
    ) -> PointInfo<Self::Var, Self::Deriv>
    {
        self.parent.encode_escape_result(state, self.local_param)
    }

    #[inline]
    fn set_meta_param(
        &mut self,
        ParamStack {
            meta_params,
            local_param,
        }: Self::MetaParam,
    )
    {
        self.meta_params = meta_params;
        self.local_param = local_param;
    }

    #[inline]
    fn set_param(&mut self, local_param: T::Param)
    {
        self.local_param = local_param;
    }

    #[inline]
    fn get_meta_params(&self) -> Self::MetaParam
    {
        ParamStack {
            local_param: self.local_param,
            meta_params: self.meta_params,
        }
    }

    #[inline]
    fn get_param(&self) -> T::Param
    {
        self.local_param
    }

    // #[inline]
    // fn julia_set(&self, _param: ComplexNum) -> Option<JuliaSet<Self>>
    // {
    //     None
    // }

    #[inline]
    fn degree(&self) -> f64
    {
        self.parent.degree()
    }

    #[inline]
    fn escaping_period(&self) -> Period
    {
        self.parent.escaping_period()
    }

    #[inline]
    fn default_bounds(&self) -> Bounds
    {
        self.parent
            .default_julia_bounds(self.parent_selection, self.local_param)
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Self::Param) -> Bounds
    {
        self.point_grid.bounds.clone()
    }

    #[inline]
    fn critical_points_child(&self, _param: Self::Param) -> Vec<Self::Var>
    {
        self.parent.critical_points_child(self.local_param)
    }

    #[inline]
    fn critical_points(&self) -> Vec<Self::Var>
    {
        self.parent.critical_points_child(self.local_param)
    }

    #[inline]
    fn cycles_child(&self, _param: Self::Param, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles_child(self.local_param, period)
    }

    #[inline]
    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        self.parent.cycles_child(self.local_param, period)
    }

    #[inline]
    fn precycles(&self, orbit_schema: OrbitSchema) -> Vec<Self::Var>
    {
        self.parent.precycles_child(self.local_param, orbit_schema)
    }

    #[inline]
    fn name(&self) -> String
    {
        "JuliaSet".to_owned()
    }

    #[inline]
    fn periodicity_tolerance(&self) -> Real
    {
        self.parent.periodicity_tolerance()
    }

    fn default_coloring(&self) -> Coloring
    {
        let mut coloring = Coloring::default();
        let _periodicity_tolerance = self.periodicity_tolerance();
        coloring.set_interior_algorithm(self.preperiod_smooth_coloring());
        coloring
    }

    fn preperiod_smooth_coloring(&self) -> InteriorColoringAlgorithm
    {
        InteriorColoringAlgorithm::InternalPotential {
            periodicity_tolerance: self.periodicity_tolerance(),
        }
    }

    fn preperiod_period_smooth_coloring(&self) -> InteriorColoringAlgorithm
    {
        InteriorColoringAlgorithm::PreperiodPeriodSmooth {
            periodicity_tolerance: self.periodicity_tolerance(),
            fill_rate: 0.015,
        }
    }

    fn is_dynamical(&self) -> bool
    {
        true
    }

    /// Compute an external ray for a given angle in [0,1).
    /// depth: Controls how deep the ray goes. Higher values bring the landing point closer to the
    /// bifurcation locus. [Suggested starting value: 25]
    /// sharpness: Controls the density of points used to approxmate the external ray. [Suggested starting value: 20]
    fn external_ray(&self, theta: Real) -> Option<Vec<Cplx>>
    {
        let escape_radius = 400.;
        let deg = self.degree().powi(self.escaping_period() as i32);
        if deg.is_nan()
        {
            return None;
        }
        let deg_log2 = deg.log2();

        let pixel_width = self.point_grid().pixel_width() * 0.3;
        let error = self.point_grid().res_x as Real * 1e-8;

        let angle = theta * TAU;
        let base_point = escape_radius * Cplx::new(0., angle).exp();
        let mut z_list = vec![base_point];

        // degree raised to the power k
        let mut deg_k = 1.0;

        for k in 0..RAY_DEPTH
        {
            let us = (0..RAY_SHARPNESS).map(|j| {
                escape_radius.ln()
                    * ((-Real::from(j) * deg_log2) / Real::from(RAY_SHARPNESS)).exp2()
            });
            let v = Cplx::new(0., angle * deg_k);
            deg_k *= deg;
            let targets = us.map(|u| (u + v).exp());

            let mut temp_z = *z_list.last()?;
            let mut dist: Real;

            let fk_and_dfk = |z: Cplx| {
                let mut z_k = z.into();
                let mut d_k = ONE;
                for _ in 0..k * self.escaping_period()
                {
                    let (f, df_dz) = self.map_and_multiplier_lazy(z_k);
                    d_k *= df_dz.into();
                    z_k = f;
                }
                (z_k.into(), d_k)
            };

            for target in targets
            {
                let (sol, z_k, d_k) = newton_until_convergence_d(fk_and_dfk, temp_z, target, error);

                temp_z = sol;

                dist = (2. * z_k.norm() * (z_k.norm()).log(deg)) / d_k.norm();

                z_list.push(temp_z);
                if dist < pixel_width
                {
                    return Some(z_list);
                }
            }
        }

        Some(z_list)
    }
}
