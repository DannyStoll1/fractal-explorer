use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::orbit_info::PointInfo;
use crate::traits::Polar;
use crate::types::Real;

pub mod algorithms;
pub mod palette;
pub mod types;

pub use algorithms::IncoloringAlgorithm;
use egui::Color32;
use image::Rgb;
use palette::ColorPalette;
use types::Hsv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use self::palette::DiscretePalette;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coloring
{
    algorithm: IncoloringAlgorithm,
    palette: ColorPalette,
}
impl Coloring
{
    #[must_use]
    pub const fn new(algorithm: IncoloringAlgorithm, palette: ColorPalette) -> Self
    {
        Self { algorithm, palette }
    }

    #[must_use]
    pub fn map_color32<D>(&self, point_info: &PointInfo<D>) -> Color32
    where
        D: Polar<Real>,
    {
        use PointInfo::*;
        match point_info
        {
            Escaping { potential } => self.palette.map_color32(*potential),
            Periodic(data) => self.algorithm.color_periodic(&self.palette, data),
            PeriodicKnownPotential(data) =>
            {
                self.algorithm.color_known_potential(&self.palette, data)
            }
            Bounded => self.palette.in_color,
            Wandering => self.palette.wandering_color,
            MarkedPoint {
                class_id,
                num_point_classes,
                ..
            } =>
            {
                let hue = (f32::from(*class_id)) / (*num_point_classes as f32);
                Hsv {
                    hue,
                    saturation: 0.8,
                    intensity: 1.0,
                }
                .into()
            }
        }
    }

    #[must_use]
    pub fn map_rgb<D>(&self, point_info: &PointInfo<D>) -> Rgb<u8>
    where
        D: Polar<Real>,
    {
        let (r, g, b, _a) = self.map_color32(point_info).to_tuple();
        Rgb([r, g, b])
    }

    pub fn set_palette(&mut self, palette: ColorPalette)
    {
        self.palette = palette;
    }
    #[must_use]
    pub const fn get_palette(&self) -> &ColorPalette
    {
        &self.palette
    }
    #[must_use]
    pub const fn get_period_coloring(&self) -> &DiscretePalette
    {
        &self.palette.period_coloring
    }
    #[must_use]
    pub const fn get_algorithm(&self) -> &IncoloringAlgorithm
    {
        &self.algorithm
    }
    pub fn get_algorithm_mut(&mut self) -> &mut IncoloringAlgorithm
    {
        &mut self.algorithm
    }
    pub fn set_interior_algorithm(&mut self, algorithm: IncoloringAlgorithm)
    {
        self.algorithm = algorithm;
    }

    #[cfg(feature = "serde")]
    pub fn save_to_file<P>(&self, filename: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        use std::io::Write;

        let toml_string =
            toml::to_string(self.get_palette()).expect("Failed to serialize palette.");
        let mut file = std::fs::File::create(filename)?;
        file.write_all(toml_string.as_bytes())?;

        Ok(())
    }

    #[cfg(feature = "serde")]
    pub fn load_palette<P>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let content = std::fs::read_to_string(path)?;
        let palette: ColorPalette = toml::from_str(&content)?;
        self.palette = palette;
        Ok(())
    }
}

impl Deref for Coloring
{
    type Target = ColorPalette;

    fn deref(&self) -> &Self::Target
    {
        &self.palette
    }
}

impl DerefMut for Coloring
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.palette
    }
}
