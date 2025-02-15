use crate::macros::{degree_impl, horner, horner_monic, profile_imports};
use dynamo_common::math_utils::weierstrass_p;
profile_imports!();

// Quadratic rational maps with a critical 4-cycle: 0 => ∞ -> 1 -> c -> 0
#[derive(Clone, Debug)]
pub struct QuadRatPer4
{
    point_grid: PointGrid,
    compute_mode: ComputeMode,
    max_iter: IterCount,
}

impl QuadRatPer4
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -1.,
        max_x: 0.2,
        min_y: -0.5,
        max_y: 0.5,
    };
}
impl Default for QuadRatPer4
{
    fractal_impl!();
}

impl DynamicalFamily for QuadRatPer4
{
    type Var = Cplx;
    type Param = Cplx;
    type Deriv = Cplx;
    type MetaParam = NoParam;
    basic_plane_impl!();
    default_name!();

    fn description(&self) -> String
    {
        "The moduli space of quadratic rational maps with a critical 4-cycle, \
            parameterized as $f_c(z) = (z-c)(z(c-1)-2c+1)/(z^2(c-1))$. \
            In these coordinates, 0 -> ∞ -> 1 -> c is the critical 4-cycle. \
            The plane is colored according to the \
            activity of the free critical point 0."
            .to_owned()
    }

    #[inline]
    fn param_map(&self, t: Cplx) -> Cplx
    {
        let pole = 2.618_033_988_749_89;
        1. / t + pole
    }

    #[inline]
    fn param_map_d(&self, t: Cplx) -> (Cplx, Cplx)
    {
        let pole = 2.618_033_988_749_89;
        let u = t.inv();
        (u + pole, -u.powi(2))
    }

    #[inline]
    fn start_point(&self, _point: Cplx, c: &Cplx) -> Cplx
    {
        let c2 = c.powi(2);
        2. * (2. * c2 - c) / (c2 + c - 1.)
    }

    #[inline]
    fn start_point_d(&self, _point: Cplx, c: &Cplx) -> (Cplx, Cplx, Cplx)
    {
        let c2 = c.powi(2);
        let denom = (c2 + c - 1.).inv();
        (
            2. * (2. * c2 - c) * denom,
            ZERO,
            (6. * c2 - 8. * c + 2.) * denom * denom,
        )
    }

    #[inline]
    fn map(&self, z: Cplx, c: &Cplx) -> Cplx
    {
        (c * (z - 2.) - z + 1.) * (z - c) / (z.powi(2) * (c - 1.))
    }

    #[inline]
    fn map_and_multiplier(&self, z: Cplx, c: &Cplx) -> (Cplx, Cplx)
    {
        let c2 = c.powi(2);
        let c_minus_1 = c - 1.;
        let u = (c_minus_1 * z.powi(2)).inv();
        let two_c = 2. * c;

        (
            (z - c) * (c_minus_1 * z - two_c + 1.) * u,
            (c2 + c_minus_1 - (4. * c2 - two_c) / z) * u,
        )
    }

    #[inline]
    fn gradient(&self, z: Self::Var, c: &Self::Param) -> (Self::Var, Self::Deriv, Self::Deriv)
    {
        let v = c - 1.;
        let c2 = c.powi(2);
        let z2 = z.powi(2);
        let u = (v * z2).inv();
        let two_c = 2. * c;
        (
            (z - c) * (c * z - z - two_c + 1.) * u,
            (c2 + v - (4. * c2 - two_c) / z) * u,
            (1. + (two_c - c2) * (z - 2.)) * u / v,
        )
    }
}

impl FamilyDefaults for QuadRatPer4
{
    default_bounds!();
}

impl HasJulia for QuadRatPer4
{
    #[inline]
    fn default_bounds_child(&self, _point: Cplx, _param: &Cplx) -> Bounds
    {
        Bounds::square(4., (2.).into())
    }
}

impl MarkedPoints for QuadRatPer4
{
    #[inline]
    fn critical_points_child(&self, c: &Self::Param) -> Vec<Self::Var>
    {
        let c2 = c.powi(2);
        vec![ZERO, 2. * (2. * c2 - c) / (c2 + c - 1.)]
    }

    fn cycles_child(&self, c: &Cplx, period: Period) -> ComplexVec
    {
        match period {
            1 => {
                let x0 = c - 1.;
                let x1 = x0.inv();
                let x2 = c.powi(2);
                let x3 = x1 * (x0 + x2);
                let x4 = x1 * (c - (x2 + x2));
                let x5 = 1. - 3. * x3;
                let s = -4. * x5.powf(3.);
                let t = 9. * x3 + 27. * x4 - 2.;
                let u = (s + t.powi(2)).sqrt();
                let x6 = (0.5 * (t + u)).powf(ONE_THIRD);
                let x7 = x6 / 3.;
                let x8 = x5 / (3. * x6);
                let r1 = -x7 * OMEGA_BAR - x8 * OMEGA + ONE_THIRD;
                let r2 = -x7 * OMEGA - x8 * OMEGA_BAR + ONE_THIRD;
                vec![-x7 - x8 + ONE_THIRD, r1, r2]
            }
            2 => {
                let c2 = c.powi(2);
                let x0 = c2 * 3.;
                let denom = 0.5 / (c - 1.);
                let disc = (x0.powi(2) - c * (8. * c2 - 6. * c + 4.) + 1.).sqrt();
                vec![denom * (x0 + disc - 1.), denom * (x0 - disc - 1.)]
            }
            3 => {
                let c2 = c.powi(2);
                let coeffs = [
                    c2 * c * horner!(c, 1., -7., 18., -20., 8.),
                    c2 * horner!(c, -4., 25., -54., 41., 4., -12.),
                    c * horner!(c, 5., -24., 26., 33., -72., 23., 10.),
                    horner!(c, -2., 2., 29., -83., 71., -4., -10., -5.),
                    horner_monic!(c, 4., -17., 19., 11., -36., 23., -4.),
                    horner!(c, -2., 9., -16., 14., -4., -3., 2.),
                    c * horner_monic!(c, 1., -4., 6., -4.),
                ];
                solve_polynomial(coeffs)
            }
            4 => {
                let c2 = c.powi(2);
                let c3 = c * c2;
                let c4 = c2.powi(2);
                let coeffs = [
                    c3 * c4 * horner!(c, -1., 12., -61., 170., -280., 272., -144., 32.),
                    c4 * horner!(
                        c, 1., -15., 103., -419., 1089., -1817., 1835., -896., -72., 272., -80.
                    ),
                    c3 * horner!(
                        c, -4., 57., -360., 1300., -2868., 3747., -2293., -527., 1686., -732.,
                        -104., 96.
                    ),
                    c2 * horner!(
                        c, 6., -79., 445., -1345., 2127., -841., -3011., 5721., -3916., 382., 726.,
                        -144., -72.
                    ),
                    c * horner!(
                        c, -4., 45., -191., 261., 737., -3856., 7348., -6869., 2028., 1633.,
                        -1223., -90., 151., 34.
                    ),
                    horner!(
                        c, 1., -6., -21., 322., -1375., 2999., -3272., 469., 3191., -3641., 1294.,
                        192., -105., -41., -9.
                    ),
                    horner_monic!(
                        c, -2., 24., -117., 264., -90., -1028., 2817., -3546., 2169., -238., -392.,
                        115., 26., -3.
                    ),
                    horner!(
                        c, 1., -14., 87., -312., 701., -987., 774., -121., -362., 329., -98., 1.,
                        -1., 2.
                    ),
                    c2 * c3 * horner_monic!(c, -1., 7., -21., 35., -35., 21., -7.),
                ];
                let mut rs = solve_polynomial(coeffs);
                rs.extend([ONE, *c, ZERO]);
                rs
            }
            _ => vec![],
        }
    }
}

impl HasDynamicalCovers for QuadRatPer4
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        match period {
            3 => {
                let param_map = |c: Cplx| {
                    // cbrt(12)
                    let alpha = Cplx::new(2.289_428_485_106_66, 0.);
                    let g2 = alpha;
                    let g3 = Cplx::new(-19. / 12., 0.);

                    let (p, _dp) = weierstrass_p(g2, g3, c, 0.01);
                    let x = (alpha * p + 1.) / 3.;
                    // let y = (dp - 1.5) / x;

                    // TODO: derivative
                    (x / (x + 1.), ONE)
                    // let xx = x + 1.;
                    // let yy = y - 3. * x - 3.;
                    //
                    // let x0 = yy / x;
                    // let _s1 = x0 * xx / x;

                    // x / xx
                };
                let bounds = Bounds {
                    min_x: -3.6,
                    max_x: 3.6,
                    min_y: -2.4,
                    max_y: 2.4,
                };
                CoveringMap::new(self, param_map).with_orig_bounds(bounds)
            }
            _ => CoveringMap::from(self),
        }
    }
}

impl InfinityFirstReturnMap for QuadRatPer4
{
    degree_impl!(2, 4);

    #[inline]
    fn escape_coeff(&self, c: &Self::Param) -> Cplx
    {
        let c2 = c.powi(2);
        let c12 = c2 - 2. * c + 1.; // (c-1)^2

        let d0 = c2 + c - 1.; // c^2 + c - 1
        let d1 = d0 - 4. * c + 2.; // c^2 - 3c + 1
        let d2 = 2. * c2 + d1; // 3c^2 - 3c + 1

        // (2*a - 1) * (a - 1)^5 * a^5 * (a^2 - 3*a + 1)^-2 * (3a^2 - 3a + 1)^-2 * (a^2 + a - 1)^-2
        let q_numer = c * (c - 1.) * (2. * c - 1.) * (c2 * c12).powi(2);
        let q_denom = (d0 * d1 * d2).powi(2);
        q_numer / q_denom
    }

    #[inline]
    fn angle_map_large_param(&self, angle: RationalAngle) -> RationalAngle
    {
        angle + RationalAngle::ONE_HALF
    }
}

impl EscapeEncoding for QuadRatPer4
{
    basic_escape_encoding!(None, 4);
}
impl ExternalRays for QuadRatPer4 {}
