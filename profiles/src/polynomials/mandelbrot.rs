use dynamo_common::symbolic_dynamics::OrbitSchema;

use crate::macros::{cplx_arr, degree_impl, horner, horner_monic, profile_imports};

profile_imports!();

fn f(z: Cplx, c: Cplx) -> Cplx
{
    z * z + c
}
fn df_dz(z: Cplx, _c: Cplx) -> Cplx
{
    z + z
}

#[derive(Clone, Debug)]
pub struct Mandelbrot
{
    point_grid: PointGrid,
    max_iter: Period,
}

impl Mandelbrot
{
    const DEFAULT_BOUNDS: Bounds = Bounds {
        min_x: -2.1,
        max_x: 0.55,
        min_y: -1.25,
        max_y: 1.25,
    };
}
impl Default for Mandelbrot
{
    fractal_impl!();
}

impl ParameterPlane for Mandelbrot
{
    parameter_plane_impl!();
    default_name!();
    default_bounds!();

    fn param_map(&self, point: Cplx) -> Self::Param
    {
        point
    }

    fn escape_radius(&self) -> Real
    {
        1e26
    }

    fn start_point(&self, _point: Cplx, _c: Self::Param) -> Self::Var
    {
        ZERO
    }

    // #[inline]
    // fn critical_value(&self, c: Self::Param) -> Self::Var
    // {
    //     c
    // }

    fn map(&self, z: Self::Var, c: Self::Param) -> Self::Var
    {
        f(z, c)
    }

    fn dynamical_derivative(&self, z: Self::Var, c: Self::Param) -> Self::Deriv
    {
        df_dz(z, c)
    }

    fn parameter_derivative(&self, _z: Self::Var, _c: Self::Param) -> Self::Deriv
    {
        ONE
    }

    fn early_bailout(&self, _start: Cplx, c: Self::Param) -> EscapeState<Cplx, Cplx>
    {
        // Main cardioid
        let four_c = 4. * c;
        let y2 = four_c.im * four_c.im;
        let temp = four_c.re - 1.;
        let mu_norm2 = temp.mul_add(temp, y2);
        let a = mu_norm2 * mu_norm2.mul_add(0.25, temp);

        if a < y2
        {
            let multiplier = 1. - (1. - four_c).sqrt();
            let decay_rate = multiplier.norm();
            let fixed_point = 0.5 * multiplier;
            let init_dist = (c - fixed_point).norm_sqr();
            let potential = init_dist.log(decay_rate);
            let preperiod = potential as Period;
            return EscapeState::Periodic {
                data: PointInfoPeriodic {
                    value: fixed_point,
                    period: 1,
                    preperiod,
                    multiplier,
                    final_error: (1e-6),
                },
            };
        }

        // Basilica bulb
        let mu2 = four_c + 4.;
        if mu2.norm_sqr() < 1.
        {
            let decay_rate = mu2.norm();
            let fixed_point = -0.5 - 0.5 * (-four_c - 3.).sqrt();
            let init_dist = (c - fixed_point).norm_sqr();
            let potential = 2. * init_dist.log(decay_rate);
            let preperiod = potential as Period;
            return EscapeState::Periodic {
                data: PointInfoPeriodic {
                    value: fixed_point,
                    period: 2,
                    preperiod,
                    multiplier: mu2,
                    final_error: (1e-6),
                },
            };
        }

        EscapeState::NotYetEscaped
    }

    #[inline]
    fn critical_points_child(&self, _param: Cplx) -> ComplexVec
    {
        vec![Cplx::new(0., 0.)]
    }

    #[inline]
    fn default_julia_bounds(&self, _point: Cplx, _param: Cplx) -> Bounds
    {
        Bounds::centered_square(2.2)
    }

    fn description(&self) -> String
    {
        "The moduli space of quadratic polynomials, \
            parameterized in the coordinates $f_c(z) = z^2 + c$, \
            All such maps have a fixed critical point at infinity \
            and a free critical point at 0. A given parameter $c$ is \
            colored according to the activity of the free critical point \
            under forward iteration of $f_c$."
            .to_owned()
    }

    fn cycles(&self, period: Period) -> Vec<Self::Var>
    {
        match period
        {
            1 => vec![ZERO],
            2 => vec![-ONE],
            3 => solve_cubic(ONE, ONE, TWO).to_vec(),
            4 =>
            {
                const COEFFS: [Cplx; 6] = cplx_arr!([1, 2, 3, 3, 3, 1]);
                solve_polynomial(COEFFS)
            }
            5 =>
            {
                const COEFFS: [Cplx; 16] =
                    cplx_arr!([1, 1, 2, 5, 14, 26, 44, 69, 94, 114, 116, 94, 60, 28, 8, 1]);
                solve_polynomial(COEFFS)
            }
            _ => vec![],
        }
    }

    fn cycles_child(&self, c: Cplx, period: Period) -> ComplexVec
    {
        use dynamo_common::math_utils::polynomial_roots::solve_polynomial;
        match period
        {
            1 =>
            {
                let u = (1. - 4. * c).sqrt();
                vec![0.5 * (1. + u), 0.5 * (1. - u)]
            }
            2 =>
            {
                let u = (-3. - 4. * c).sqrt();
                vec![0.5 * (-1. + u), -0.5 * (1. + u)]
            }
            3 =>
            {
                let c2 = c * c;
                let coeffs = vec![
                    1. + c + (2. + c) * c2,
                    1. + c + c + c2,
                    1. + 3. * (c + c2),
                    1. + c + c,
                    1. + 3. * c,
                    ONE,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            4 =>
            {
                let c2 = c * c;
                let coeffs = vec![
                    1. + c2 * horner_monic!(c, 2., 3., 3., 3.),
                    c * horner_monic!(c, 2., 1., 2.),
                    c * horner!(c, 1., 5., 6., 12., 6.),
                    1. + 4. * c2 * (1. + c),
                    c * horner!(c, 4., 3., 18., 15.),
                    c * horner!(c, 2., 6.),
                    1. + c2 * (12. + 20. * c),
                    4. * c,
                    3. * c + 15. * c2,
                    ONE,
                    6. * c,
                    ZERO,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            5 =>
            {
                let v = horner_monic!(
                    c, 1., 2., 5., 14., 26., 44., 69., 94., 114., 116., 94., 60., 28., 8.
                );
                let u = 14. * c + 1.;
                let coeffs = [
                    v * c + 1.,
                    v,
                    horner!(
                        c, 1., 3., 9., 28., 66., 137., 265., 436., 642., 794., 766., 576., 316.,
                        105., 15.
                    ),
                    horner!(
                        c, 1., 4., 14., 40., 93., 196., 342., 528., 678., 672., 516., 288., 97.,
                        14.
                    ),
                    horner!(
                        c, 1., 5., 20., 67., 179., 437., 876., 1572., 2398., 2790., 2496., 1629.,
                        637., 105.
                    ),
                    horner!(
                        c, 1., 6., 27., 86., 241., 534., 1044., 1720., 2118., 1980., 1341., 540.,
                        91.
                    ),
                    horner!(
                        c, 1., 7., 35., 126., 401., 1000., 2196., 4200., 5990., 6445., 5071.,
                        2366., 455.
                    ),
                    horner!(
                        c, 1., 8., 40., 160., 466., 1152., 2480., 3872., 4465., 3730., 1826., 364.
                    ),
                    horner!(
                        c, 1., 9., 50., 221., 712., 1932., 4712., 8415., 11025., 10615., 6006.,
                        1365.
                    ),
                    horner!(c, 1., 10., 61., 246., 780., 2232., 4543., 6560., 6885., 4180., 1001.),
                    horner!(
                        c, 1., 11., 73., 324., 1116., 3527., 8113., 13140., 15741., 11011., 3003.
                    ),
                    horner!(c, 1., 12., 78., 336., 1295., 3570., 6580., 8856., 6831., 2002.),
                    horner!(c, 1., 13., 92., 427., 1779., 5467., 11172., 16962., 15015., 5005.),
                    horner!(c, 1., 14., 91., 484., 1897., 4592., 8106., 8184., 3003.),
                    horner!(c, 1., 15., 105., 598., 2565., 6822., 13398., 15444., 6435.),
                    horner!(c, 1., 14., 114., 668., 2230., 5292., 7260., 3432.),
                    horner!(c, 1., 15., 130., 815., 2970., 7722., 12012., 6435.),
                    horner!(c, 1., 16., 147., 740., 2430., 4752., 3003.),
                    horner!(c, 1., 17., 165., 900., 3190., 7007., 5005.),
                    horner!(c, 1., 18., 160., 760., 2255., 2002.),
                    horner!(c, 1., 19., 180., 913., 3003., 3003.),
                    horner!(c, 1., 20., 153., 748., 1001.),
                    horner!(c, 1., 21., 171., 910., 1365.),
                    horner!(c, 1., 18., 162., 364.),
                    horner!(c, 1., 19., 182., 455.),
                    horner!(c, 1., 20., 91.),
                    horner!(c, 1., 21., 105.),
                    u,
                    u + c,
                    ONE,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            6 =>
            {
                let c2 = c * c;
                let coeffs = [
                    horner_monic!(
                        c, 1., -1., 1., 3., 7., 17., 35., 76., 155., 298., 536., 927., 1525.,
                        2331., 3310., 4346., 5258., 5843., 5892., 5313., 4219., 2892., 1672., 792.,
                        293., 78., 13.
                    ),
                    -horner_monic!(
                        c, 1., -2., -1., -2., -1., -8., -16., -18., -26., -32., -31., 16., 149.,
                        384., 730., 1164., 1635., 2032., 2201., 2050., 1614., 1052., 554., 226.,
                        66., 12.
                    ),
                    c * horner!(
                        c, -1., 1., 2., 20., 38., 118., 336., 830., 1789., 3675., 7278., 13177.,
                        21870., 33146., 45768., 57763., 65879., 66831., 59408., 45336., 29012.,
                        15146., 6156., 1793., 325., 27.
                    ),
                    horner!(
                        c, 1., 0., 4., -4., 8., 50., 62., 112., 181., 342., 426., 12., -1202.,
                        -3608., -7396., -12632., -18711., -23728., -25494., -22864., -16778.,
                        -9862., -4466., -1440., -287., -26.
                    ),
                    horner!(
                        c, -1., 2., -3., 26., 33., 84., 364., 1166., 2932., 6726., 15798., 33828.,
                        65816., 115718., 182688., 262556., 340280., 390387., 390678., 333800.,
                        237612., 137324., 61590., 19710., 3900., 351.
                    ),
                    c * horner!(
                        c, 2., -2., -10., 62., 90., 190., 296., 764., 1772., 1900., -148., -6726.,
                        -19956., -42692., -77016., -116249., -145938., -150666., -125470., -82824.,
                        -41866., -14982., -3288., -325.
                    ),
                    horner!(
                        c, 1., -4., 14., 28., 6., 202., 966., 3128., 7588., 20584., 52104.,
                        119292., 245646., 444356., 728148., 1077208., 1405041., 1592556., 1533190.,
                        1220740., 784784., 390258., 137862., 29900., 2925.
                    ),
                    c * horner!(
                        c, 2., -14., 36., 80., 212., 328., 672., 3104., 5500., 5416., -3512.,
                        -28760., -81704., -185472., -340960., -508344., -612114., -584050.,
                        -435980., -247500., -98868., -24012., -2600.
                    ),
                    horner!(
                        c, -1., 3., 21., -17., 63., 463., 2390., 6152., 18021., 53705., 144586.,
                        354856., 738994., 1379504., 2338720., 3486132., 4503660., 4920750.,
                        4409715., 3170035., 1756986., 688666., 164450., 17550.
                    ),
                    horner!(
                        c, 1., -6., 7., 46., 131., 368., 76., 2940., 8045., 13180., 8944., -20888.,
                        -94332., -285488., -662360., -1198620., -1712196., -1894170., -1611725.,
                        -1034550., -464310., -125488., -14950.
                    ),
                    c * horner!(
                        c, 7., -7., 2., 97., 1220., 4040., 11356., 39435., 122452., 368668.,
                        894960., 1895936., 3694012., 6322848., 9368184., 11712477., 11898351.,
                        9619643., 5976432., 2613996., 690690., 80730.
                    ),
                    horner!(
                        c, -1., 0., 16., 36., 348., -268., 1424., 7344., 16828., 23760., 592.,
                        -61208., -283360., -886396., -2009148., -3484176., -4538121., -4442796.,
                        -3249114., -1650264., -499422., -65780.
                    ),
                    horner!(
                        c, 1., 0., -6., 10., 308., 2266., 5484., 21820., 73304., 281624., 821580.,
                        1961932., 4397120., 8677956., 14827548., 21411444., 24864744., 22751949.,
                        15943774., 7831362., 2302300., 296010.
                    ),
                    c * horner!(
                        c, 2., 2., 188., -142., 12., 4540., 13504., 30788., 19872., -10472.,
                        -162932., -814772., -2436252., -5304348., -8306280., -9462183., -7951614.,
                        -4608450., -1572648., -230230.
                    ),
                    c * horner!(
                        c, -2., 6., -36., 998., 2204., 9676., 30960., 156468., 586996., 1569376.,
                        4035152., 9213996., 18219084., 30707820., 41178216., 42945111., 34177770.,
                        18987650., 6249100., 888030.
                    ),
                    c2 * horner!(
                        c, 48., 40., -456., 2016., 6928., 26416., 27624., 17396., -17732.,
                        -480040., -2112240., -6098352., -11838720., -15916896., -15519504.,
                        -10360548., -4018652., -657800.
                    ),
                    c * horner!(
                        c, 2., -38., 282., 826., 3525., 9658., 59106., 330417., 991776., 2898731.,
                        7687317., 17576754., 34973940., 54838260., 65702076., 59828967., 37898388.,
                        14060475., 2220075.
                    ),
                    c * horner!(
                        c, 4., 52., -302., 667., 2132., 15620., 25434., 16512., 56936., -125345.,
                        -1228656., -5239728., -13253280., -21432240., -24515700., -19058292.,
                        -8479548., -1562275.
                    ),
                    c * horner!(
                        c, -6., 28., 302., 955., 2930., 11652., 145028., 505753., 1646205.,
                        5107179., 13387088., 31794360., 59241260., 82287480., 86437384., 63005349.,
                        26558675., 4686825.
                    ),
                    c * horner!(
                        c, 14., -88., 148., 340., 6058., 17970., 7176., 54549., 58630., -366080.,
                        -3212352., -11638120., -23264500., -31621700., -28992480., -14954577.,
                        -3124550.
                    ),
                    c * horner!(
                        c, -5., 92., 136., 1180., -1198., 47134., 212514., 742148., 2740947.,
                        8069776., 23034804., 52124176., 84859852., 103733388., 87929644.,
                        42493880., 8436285.
                    ),
                    horner!(
                        c, 1., -10., 6., 46., 1274., 9578., 2598., 25256., 79871., 75504.,
                        -1189760., -7897396., -20394764., -33440836., -36709596., -22227568.,
                        -5311735.
                    ),
                    horner!(
                        c, -1., 16., -6., 440., -1474., 9726., 74522., 264788., 1207569., 3868332.,
                        13175656., 37340160., 72239596., 103725636., 103500828., 57946200.,
                        13037895.
                    ),
                    c * horner!(
                        c, -6., 30., -4., 3636., 1756., 5404., 43240., 136968., -9204., -3967652.,
                        -14389752., -28989896., -38798760., -27992472., -7726160.
                    ),
                    horner!(
                        c, 1., -3., 99., -375., 488., 21490., 74074., 441546., 1502124., 5827640.,
                        21663746., 50749114., 86415420., 102965940., 67603900., 17383860.
                    ),
                    horner!(
                        c, -1., 10., -75., 922., 1198., -616., 14882., 63804., 311844., -1290380.,
                        -8091694., -20507916., -34213452., -29953728., -9657700.
                    ),
                    c * horner!(
                        c, 9., -23., -384., 4788., 16198., 132426., 497400., 1912410., 10054650.,
                        29340674., 59798928., 86532992., 67603900., 20058300.
                    ),
                    horner!(
                        c, 1., -16., 142., 496., -742., 3640., 11340., 218736., -92910., -3555040.,
                        -11724900., -25069968., -27249572., -10400600.
                    ),
                    c * horner!(
                        c, 7., -126., 734., 2860., 30822., 152436., 425820., 3641400., 13892230.,
                        34151436., 61244676., 57946200., 20058300.
                    ),
                    horner!(
                        c, -1., 10., 118., -188., 590., -1860., 81084., 180120., -1170450.,
                        -5329500., -15135780., -21038928., -9657700.
                    ),
                    horner!(
                        c, 1., -18., 66., 428., 4962., 44436., 49620., 974712., 5350818.,
                        15931652., 36283236., 42493880., 17383860.
                    ),
                    c * horner!(
                        c, 16., -16., 16., -1248., 15552., 126288., -257856., -1875984., -7418664.,
                        -13728792., -7726160.
                    ),
                    horner!(
                        c, -1., 3., 47., 444., 11067., -576., 165036., 1662804., 5979699.,
                        17814742., 26558675., 13037895.
                    ),
                    horner!(
                        c, 1., 0., -16., -174., 30., 47724., -19278., -490314., -2877930.,
                        -7518148., -5311735.
                    ),
                    c * horner!(
                        c, 2., 12., 1968., 120., 4392., 412566., 1768140., 7138395., 14060475.,
                        8436285.
                    ),
                    c * horner!(
                        c, -2., 6., -720., 11370., 11754., -89034., -842688., -3417777., -3124550.
                    ),
                    c2 * horner!(
                        c, 202., 660., -6620., 80256., 400862., 2278518., 6249100., 4686825.
                    ),
                    c * horner!(
                        c, 2., -150., 1570., 6220., -10450., -166782., -1269048., -1562275.
                    ),
                    c * horner!(c, 8., 210., -2020., 11660., 67914., 556094., 2302300., 2220075.),
                    c * horner!(c, -10., 70., 1700., -1100., -13860., -375452., -657800.),
                    c * horner!(c, 26., -285., 1067., 8778., 95634., 690690., 888030.),
                    c * horner!(c, -10., 291., -286., 3234., -85008., -230230.),
                    horner!(c, 1., -17., 11., 1056., 9108., 164450., 296010.),
                    horner!(c, -1., 28., -66., 1320., -13662., -65780.),
                    c * horner!(c, -11., 150., -230., 29900., 80730.),
                    horner!(c, 1., -6., 198., -1288., -14950.),
                    horner!(c, -1., 18., -198., 3900., 17550.),
                    c * horner!(c, 12., -12., -2600.),
                    horner!(c, 1., -24., 325., 2925.),
                    c * horner!(c, 12., -325.),
                    horner!(c, -1., 13., 351.),
                    1. - 26. * c,
                    c * 27.,
                    -ONE,
                    ONE,
                ];
                solve_polynomial(coeffs)
            }
            _ => vec![],
        }
    }

    fn precycles_child(&self, c: Cplx, orbit_schema: OrbitSchema) -> ComplexVec
    {
        use dynamo_common::math_utils::polynomial_roots::solve_polynomial;
        match (orbit_schema.preperiod, orbit_schema.period)
        {
            (2, 1) =>
            {
                let u = 0.5 * (1. - 4. * c).sqrt();
                let v0 = (-0.5 + u - c).sqrt();
                let v1 = (-0.5 - u - c).sqrt();
                vec![v0, -v0, v1, -v1]
            }
            (2, 2) =>
            {
                let u = 0.5 * (-3. - 4. * c).sqrt();
                let v0 = (0.5 + u - c).sqrt();
                let v1 = (0.5 - u - c).sqrt();
                vec![v0, -v0, v1, -v1]
            }
            (2, 3) =>
            {
                let coeffs = [
                    horner_monic!(c, 1., 0., 1., 2., 2., 2.),
                    horner!(c, -1., 0., 2., 4., 7., 6.),
                    horner!(c, 1., 0., 3., 8., 15.),
                    horner!(c, -1., 2., 2., 20.),
                    horner!(c, 1., -2., 15.),
                    horner!(c, -1., 6.),
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            (2, 4) =>
            {
                let coeffs = [
                    horner_monic!(c, 1., 0., 0., 2., 6., 8., 11., 18., 23., 22., 15., 6.),
                    c * horner!(c, -2., -2., 8., 15., 20., 54., 104., 135., 120., 60., 12.),
                    c * horner!(c, -2., 5., 18., 13., 54., 186., 348., 420., 270., 66.),
                    horner!(c, -1., 0., 12., 8., 12., 160., 484., 840., 720., 220.),
                    c * horner!(c, 4., 8., -12., 55., 384., 1050., 1260., 495.),
                    c * horner!(c, 4., -6., -12., 162., 840., 1512., 792.),
                    horner!(c, 1., 0., -16., 20., 420., 1260., 924.),
                    c * horner!(c, -4., -12., 120., 720., 792.),
                    c * horner!(c, -6., 15., 270., 495.),
                    horner!(c, -1., 0., 60., 220.),
                    c * horner!(c, 6., 66.),
                    c * 12.,
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            (3, 3) =>
            {
                let coeffs = [
                    horner!(c, 1., 0., 0., 2., 5., 6., 10., 16., 18., 18., 14., 6., 1.),
                    c * horner!(c, -2., 0., 8., 8., 20., 56., 80., 104., 110., 60., 12.),
                    horner!(c, -1., 0., 8., 4., 10., 84., 148., 244., 375., 270., 66.),
                    c * horner!(c, 4., 0., -8., 72., 156., 288., 720., 720., 220.),
                    horner!(c, 1., 0., -12., 38., 115., 160., 840., 1260., 495.),
                    c * horner!(c, -6., 12., 68., 8., 588., 1512., 792.),
                    horner!(c, -1., 2., 30., -36., 210., 1260., 924.),
                    c * horner!(c, 8., -16., 0., 720., 792.),
                    horner!(c, 1., -2., -30., 270., 495.),
                    c * horner!(c, -10., 60., 220.),
                    horner!(c, -1., 6., 66.),
                    c * 12.,
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            (3, 4) =>
            {
                let c2 = c * c;
                let coeffs = [
                    horner!(
                        c, 1., 0., 0., 0., 4., 14., 30., 56., 102., 192., 356., 626., 1015., 1494.,
                        1982., 2336., 2415., 2166., 1658., 1062., 555., 226., 66., 12., 1.
                    ),
                    c2 * horner!(
                        c, -4., -4., 8., 44., 112., 232., 504., 1150., 2536., 5096., 9096., 14412.,
                        19984., 23952., 24624., 21390., 15408., 9000., 4080., 1320., 264., 24.
                    ),
                    c * horner!(
                        c, -2., -2., -4., 30., 128., 312., 660., 1655., 4452., 11176., 24444.,
                        46830., 77336., 108528., 129000., 128151., 104472., 68580., 34800., 12540.,
                        2772., 276.
                    ),
                    c2 * horner!(
                        c, -8., 0., 72., 292., 600., 1432., 4400., 13940., 37944., 89384., 178704.,
                        297192., 412560., 472944., 439488., 326160., 186360., 75240., 18480.,
                        2024.
                    ),
                    c * horner!(
                        c, -2., -10., 18., 193., 454., 926., 2748., 10765., 37230., 110466.,
                        273796., 548058., 899220., 1202340., 1284192., 1084500., 702270., 319770.,
                        87780., 10626.
                    ),
                    c * horner!(
                        c, -6., 0., 72., 272., 520., 1296., 5264., 23520., 91272., 291392.,
                        717360., 1411536., 2230200., 2764944., 2676240., 1977984., 1023264.,
                        316008., 42504.
                    ),
                    horner!(
                        c, -1., 0., 12., 120., 236., 664., 1688., 9072., 49356., 218368., 683424.,
                        1642872., 3117492., 4540536., 5077800., 4316640., 2558160., 895356.,
                        134596.
                    ),
                    c2 * horner!(
                        c, 32., 64., 384., 512., 1632., 15168., 112992., 476856., 1436160.,
                        3342624., 5806944., 7572240., 7466400., 5116320., 2046528., 346104.
                    ),
                    c * horner!(
                        c, 4., 8., 168., 280., -156., 420., 36948., 240399., 942480., 2768832.,
                        5853276., 8996130., 10382580., 8314020., 3837240., 735471.
                    ),
                    c2 * horner!(
                        c, 40., 160., -120., -1900., 4792., 83496., 456720., 1767700., 4674384.,
                        8580000., 11704160., 11085360., 5969040., 1307504.
                    ),
                    c * horner!(
                        c, 4., 60., -12., -894., -1844., 17220., 156024., 858066., 2954952.,
                        6589440., 10735296., 12193896., 7759752., 1961256.
                    ),
                    c * horner!(
                        c, 12., 0., -192., -1216., 672., 32592., 306864., 1467648., 4071600.,
                        8009040., 11085360., 8465184., 2496144.
                    ),
                    horner!(
                        c, 1., 0., -16., -344., -672., 1260., 75180., 563472., 2014740., 4839900.,
                        8314020., 7759752., 2704156.
                    ),
                    c2 * horner!(
                        c, 0., -56., -168., -1680., 9912., 162288., 791280., 2350080., 5116320.,
                        5969040., 2496144.
                    ),
                    c * horner!(
                        c, -4., -12., -600., -540., 33192., 243000., 905760., 2558160., 3837240.,
                        1961256.
                    ),
                    c2 * horner!(
                        c, -96., -576., 4320., 56880., 272544., 1023264., 2046528., 1307504.
                    ),
                    c * horner!(c, -6., -138., 270., 9675., 62730., 319770., 895356., 735471.),
                    c * horner!(c, -18., 0., 1080., 10800., 75240., 316008., 346104.),
                    horner!(c, -1., 0., 60., 1360., 12540., 87780., 134596.),
                    c2 * horner!(c, 120., 1320., 18480., 42504.),
                    c * horner!(c, 6., 66., 2772., 10626.),
                    c2 * horner!(c, 264., 2024.),
                    c * horner!(c, 12., 276.),
                    c * 24.,
                    ONE,
                ];
                let zs = solve_polynomial(coeffs);
                zs.iter().map(|z| z.sqrt()).flat_map(|w| [w, -w]).collect()
            }
            _ => vec![],
        }
    }
}

impl HasDynamicalCovers for Mandelbrot
{
    fn marked_cycle_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| (0.25 - c * c, -2. * c);
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
                // param_map = |l| {
                //     // let coeffs = [
                //     //     horner!(l, 16., -3., 0.1875, -0.00390625),
                //     //     ZERO,
                //     //     horner!(l, 32., -1., -0.0625),
                //     //     l + 48.,
                //     //     l + 48.,
                //     //     Cplx::from(48.),
                //     //     Cplx::from(16.),
                //     // ];
                //     // dbg!(l, &coeffs);
                //     let a0 = horner!(l, 1., -0.25, 1. / 64.);
                //     let a1 = 1. - 0.125 * l;
                //     let a2 = TWO;
                //     let cs = solve_cubic(a0, a1, a2);
                //     cs[0]
                // };
                // bounds = Bounds::square(16., 8.0.into());
            }
            3 =>
            {
                param_map = |t| (-1.75 * (1. + 7. * t * t), -24.5 * t);
                bounds = Bounds {
                    min_x: -0.3,
                    max_x: 0.3,
                    min_y: -0.5,
                    max_y: 0.5,
                };
                // param_map = |l| {
                //     let a0 = horner!(l, 1., -0.25, 1. / 64.);
                //     let a1 = 1. - 0.125 * l;
                //     let a2 = TWO;
                //     let cs = solve_cubic(a0, a1, a2);
                //     cs[1]
                // };
                // bounds = Bounds::square(16., 8.0.into());
            }
            // 4 => {
            //     param_map = |l| {
            //         let u = 256.*l + 12288.;
            //         let coeffs = [
            //             -horner_monic!(l, -4096., 768., -48.),
            //             ZERO,
            //             horner!(l, 8192., -256., 48., -16.),
            //             u,
            //             u,
            //             Cplx::new(12288., 0.),
            //             Cplx::new(4096., 0.),
            //         ];
            //         let cs = solve_polynomial(coeffs);
            //         cs[0]
            //     };
            //     bounds = Bounds::centered_square(4.);
            // }
            4 =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    (-0.25 * t2 - 0.75 - t.inv(), 0.5 * t - t2.inv())
                };
                bounds = Bounds {
                    min_x: -2.9,
                    max_x: 2.1,
                    min_y: -3.1,
                    max_y: 3.1,
                };
            }
            _ =>
            {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }

    fn dynatomic_curve(self, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match period
        {
            1 =>
            {
                param_map = |c| (0.25 - c * c, -2. * c);
                bounds = Bounds {
                    min_x: -1.8,
                    max_x: 1.8,
                    min_y: -1.0,
                    max_y: 1.0,
                };
            }
            2 =>
            {
                param_map = |t| {
                    let u = 9. / (t * t);
                    ((t - 1.) * u - 3., (t - 2.) * u / t)
                };
                bounds = Bounds {
                    min_x: 0.5,
                    max_x: 8.3,
                    min_y: -2.7,
                    max_y: 2.7,
                };
            }
            3 =>
            {
                param_map = |t| {
                    let t2 = t * t;

                    let v = t2 * (t2 - 3. * t + 6.) - t - t + 2.;
                    let dv_dt = horner!(t, -2., 12., -9., 4.);

                    let w = (t2 - t).inv();
                    let dw_dt = (1. - 2. * t) * w * w;

                    let u = v + w;
                    let du_dt = dv_dt + dw_dt;
                    (-0.25 * u * w, -0.25 * (du_dt * w + u * dw_dt))
                };
                bounds = Bounds {
                    min_x: -2.5,
                    max_x: 3.5,
                    min_y: -3.,
                    max_y: 3.,
                };
            }
            _ =>
            {
                param_map = |t| (t, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
    fn misiurewicz_curve(self, preperiod: Period, period: Period) -> CoveringMap<Self>
    {
        let param_map: fn(Cplx) -> (Cplx, Cplx);
        let bounds: Bounds;

        match (preperiod, period)
        {
            (2, 1) =>
            {
                param_map = |t| {
                    let t2 = t * t;
                    let u = (t2 - 1.).inv();
                    let u2 = u * u;
                    (-2. * (t2 + 1.) * u2, 4. * t * (t2 + 3.) * u2 * u)
                };
                bounds = Bounds {
                    min_x: -3.5,
                    max_x: 3.5,
                    min_y: -3.0,
                    max_y: 3.0,
                };
            }
            (2, 2) =>
            {
                param_map = |c| {
                    let c2 = c * c;
                    (
                        -(c2 * (c2 + c + c + 2.) - c - c + 1.) / (4. * c2),
                        -0.5 * (c2 + c - 1.) * (c2 + 1.) / (c2 * c),
                    )
                };
                bounds = Bounds {
                    min_x: -4.,
                    max_x: 2.4,
                    min_y: -2.5,
                    max_y: 2.5,
                };
            }
            // (3, 1) =>
            // {
            //     param_map = |t| {
            //         let numer = horner_monic!(t, 301., 42., 252., 112., 21., 0.);
            //         let denom = -36. * (t + 2.) * (t + 2.) * (t - 1.) * (t - 1.);
            //         numer / denom
            //     };
            //     bounds = Bounds {
            //         min_x: -4.,
            //         max_x: 2.4,
            //         min_y: -2.5,
            //         max_y: 2.5,
            //     };
            // }
            (_, _) =>
            {
                param_map = |c| (c, ONE);
                bounds = self.point_grid.bounds.clone();
            }
        };
        let grid = self.point_grid.new_with_same_height(bounds);
        CoveringMap::new(self, param_map, grid)
    }
}

degree_impl!(Mandelbrot, 2);
