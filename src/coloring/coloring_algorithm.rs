use super::palette::*;
use crate::primitive_types::*;
use epaint::Color32;

fn multiplier_coloring_rate(multiplier: ComplexNum) -> f64
{
    let scaling_rate = multiplier.norm();

    if scaling_rate > 1e-10
    {
        -scaling_rate.log2() / 40.
    }
    else
    {
        10.
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ColoringAlgorithm
{
    PeriodMultiplier,
    Period,
    Solid,
    PreperiodSmooth
    {
        periodicity_tolerance: f64,
    },
    Preperiod,
    Multiplier,
}
impl ColoringAlgorithm
{
    pub fn color_periodic(
        &self,
        palette: ColorPalette,
        period: Period,
        preperiod: Period,
        multiplier: ComplexNum,
        final_error: ComplexNum,
    ) -> Hsv
    {
        match self
        {
            Self::Solid => Hsv::new(0., 0., 0.),
            Self::Period =>
            {
                let hue = period as f32;
                palette.period_coloring.map_hsv(hue, 0.)
            }
            Self::PeriodMultiplier =>
            {
                let hue = period as f32;
                let luminosity = 1.0_f32;//multiplier.norm() as f32;
                palette.period_coloring.map_hsv(hue, luminosity)
            }
            Self::Preperiod =>
            {
                let coloring_rate = 0.02;

                let hue = IterCount::from(period);

                let v = IterCount::from(preperiod);
                let luminosity = (v * coloring_rate / hue).tanh();
                palette
                    .period_coloring
                    .map_hsv(hue as f32, luminosity as f32)
            }
            Self::PreperiodSmooth {
                periodicity_tolerance,
            } =>
            {
                let hue = IterCount::from(period);
                let luminosity: IterCount;

                let mult_norm = multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-10
                {
                    let w = 2.
                        * (final_error.norm_sqr().log2() / periodicity_tolerance.log2()).log2()
                            as IterCount;
                    let v = preperiod as IterCount - hue * w;
                    luminosity = (0.1 * v / hue).tanh();
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let w = final_error.norm_sqr() / periodicity_tolerance;
                    let v = preperiod as IterCount - hue * w;
                    luminosity = (0.1 * v / hue).tanh();
                }
                else
                {
                    let coloring_rate = multiplier_coloring_rate(multiplier);

                    let mut w = -(final_error.norm_sqr() / periodicity_tolerance)
                        .log(multiplier.norm()) as IterCount;
                    if w.is_infinite() || w.is_nan()
                    {
                        w = -0.2;
                    }
                    let v = preperiod as IterCount + hue * w;
                    luminosity = (v * coloring_rate / hue).tanh();
                }
                palette
                    .period_coloring
                    .map_hsv(hue as f32, luminosity as f32)
            }
            Self::Multiplier => Hsv::new(
                (multiplier.arg() / TAU) as f32 + 0.5,
                1.,
                multiplier.norm() as f32,
            ),
        }
    }

    pub fn encode_periodic(
        &self,
        period: Period,
        preperiod: Period,
        multiplier: ComplexNum,
        final_error: ComplexNum,
    ) -> IterCount
    {
        let hue: f64;
        let luminosity: f64;
        match self
        {
            Self::Solid => return 0.,
            Self::Period =>
            {
                hue = IterCount::from(period);
                luminosity = 1.;
            }
            Self::PeriodMultiplier =>
            {
                hue = IterCount::from(period);
                luminosity = multiplier.norm();
            }
            Self::Preperiod =>
            {
                let coloring_rate = 0.02;

                hue = IterCount::from(period);

                let v = IterCount::from(preperiod);
                luminosity = (v * coloring_rate / hue).tanh();
            }
            Self::PreperiodSmooth {
                periodicity_tolerance,
            } =>
            {
                hue = IterCount::from(period);
                let mult_norm = multiplier.norm();

                // Superattracting case
                if mult_norm <= 1e-10
                {
                    let w = 2.
                        * (final_error.norm_sqr().log2() / periodicity_tolerance.log2()).log2()
                            as IterCount;
                    let v = preperiod as IterCount - hue * w;
                    luminosity = (0.1 * v / hue).tanh();
                }
                // Parabolic case
                else if 1. - mult_norm <= 1e-5
                {
                    let w = final_error.norm_sqr() / periodicity_tolerance;
                    let v = preperiod as IterCount - hue * w;
                    luminosity = (0.1 * v / hue).tanh();
                }
                else
                {
                    let coloring_rate = multiplier_coloring_rate(multiplier);

                    let mut w = -(final_error.norm_sqr() / periodicity_tolerance)
                        .log(multiplier.norm()) as IterCount;
                    if w.is_infinite() || w.is_nan()
                    {
                        w = -0.2;
                    }
                    let v = preperiod as IterCount + hue * w;
                    luminosity = (v * coloring_rate / hue).tanh();
                }
            }
            Self::Multiplier =>
            {
                hue = multiplier.arg() / TAU + 0.5;
                luminosity = multiplier.norm();
            }
        }
        -(hue + 0.9999 * luminosity)
    }
}
