use crate::macros::{
    dynamo_menu_button, dynamo_menu_button_dyn, dynamo_menu_button_mc, dynamo_menu_button_mis,
};
use dynamo_common::consts::{OMEGA, ONE};
use dynamo_common::types::{Cplx, ParamList};
use dynamo_core::dynamics::covering_maps::HasDynamicalCovers;
use dynamo_core::dynamics::julia::JuliaSet;
use dynamo_core::dynamics::{Displayable, ParameterPlane};
use dynamo_gui::hotkeys::{
    Hotkey, ANNOTATION_HOTKEYS, FILE_HOTKEYS, IMAGE_HOTKEYS, INCOLORING_HOTKEYS, PALETTE_HOTKEYS,
    SELECTION_HOTKEYS,
};
use dynamo_gui::interface::{Interface, MainInterface};
use dynamo_profiles::*;
use egui::Ui;
use egui_dock::{NodeIndex, SurfaceIndex};
use seq_macro::seq;

#[cfg(feature = "scripting")]
use crate::script_editor::{Popup, Response};
#[cfg(feature = "scripting")]
use std::path::Path;

#[derive(Clone, Copy, Default, Debug)]
pub enum MenuState
{
    #[default]
    Closed,
    Open,
}
impl MenuState
{
    pub fn close(&mut self)
    {
        *self = Self::Closed
    }
    pub fn open(&mut self)
    {
        *self = Self::Open
    }
    pub fn is_open(&self) -> bool
    {
        matches!(self, Self::Open)
    }
    pub fn is_closed(&self) -> bool
    {
        matches!(self, Self::Closed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabID
{
    pub surface: SurfaceIndex,
    pub node: NodeIndex,
}
impl Default for TabID {
    fn default() -> Self {
        Self {
            surface: SurfaceIndex::main(),
            node: NodeIndex(0),
        }
    }
}

impl From<TabID> for (SurfaceIndex, NodeIndex)
{
    fn from(value: TabID) -> Self
    {
        (value.surface, value.node)
    }
}

pub struct FractalTab
{
    pub interface: Box<dyn Interface>,
    pub id: TabID,
    pub menu_state: MenuState,
    #[cfg(feature = "scripting")]
    pub popup: Option<Popup>,
}

impl FractalTab
{
    #[must_use]
    pub const fn with_id(
        mut self,
        tab_id: TabID,
    ) -> Self
    {
        self.id = tab_id;
        self
    }

    pub fn update(&mut self, ui: &mut Ui)
    {
        ui.label(self.interface.name());
        self.show_menu(ui);
        if self.should_update_interface()
        {
            self.interface.update(ui.ctx());
        }
        self.interface.show(ui);

        #[cfg(feature = "scripting")]
        self.show_popup(ui);
    }

    fn show_menu(&mut self, ui: &mut Ui)
    {
        self.menu_state.close();
        egui::menu::bar(ui, |ui| {
            self.file_menu(ui);
            self.dynamo_menu(ui);
            self.image_menu(ui);
            self.selection_menu(ui);
            self.annotations_menu(ui);
            self.coloring_menu(ui);
            #[cfg(feature = "scripting")]
            self.transpiled_scripts_menu(ui);
        });
    }

    fn file_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("File", |ui| {
            self.menu_state.open();
            FILE_HOTKEYS.iter().for_each(|hotkey| {
                self.hotkey_button(ui, hotkey);
            });
        });
    }

    fn dynamo_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Fractal", |ui| {
            self.menu_state.open();
            self.polynomials_menu(ui);
            self.rational_maps_menu(ui);
            self.transcendental_menu(ui);
            self.non_analytic_menu(ui);
        });
    }

    fn coloring_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Coloring", |ui| {
            self.menu_state.open();
            ui.menu_button("Palette", |ui| {
                PALETTE_HOTKEYS.iter().for_each(|hotkey| {
                    self.hotkey_button(ui, hotkey);
                });
            });

            ui.menu_button("Incoloring", |ui| {
                INCOLORING_HOTKEYS.iter().for_each(|hotkey| {
                    self.hotkey_button(ui, hotkey);
                });
            });
            // ui.menu_button("Algorithm", |ui| {
            //     if ui.button("[0] Solid").clicked()
            //     {
            //         self.interface
            //             .set_coloring_algorithm(InteriorColoringAlgorithm::Solid);
            //     }
            //     else if ui.button("[1] Period").clicked()
            //     {
            //         self.interface
            //             .set_coloring_algorithm(InteriorColoringAlgorithm::Period);
            //     }
            //     else if ui.button("[2] Period and Multiplier").clicked()
            //     {
            //         self.interface
            //             .set_coloring_algorithm(InteriorColoringAlgorithm::PeriodMultiplier);
            //     }
            //     else if ui.button("[3] Multiplier").clicked()
            //     {
            //         self.interface
            //             .set_coloring_algorithm(InteriorColoringAlgorithm::Multiplier);
            //     }
            //     else if ui.button("[4] Preperiod").clicked()
            //     {
            //         self.interface
            //             .set_coloring_algorithm(InteriorColoringAlgorithm::Preperiod);
            //     }
            //     else if ui.button("[5] Internal potential").clicked()
            //     {
            //         self.interface
            //             .parent_mut()
            //             .select_preperiod_smooth_coloring();
            //         self.interface
            //             .child_mut()
            //             .select_preperiod_smooth_coloring();
            //     }
            //     else if ui.button("Preperiod and Period").clicked()
            //     {
            //         self.interface
            //             .set_coloring_algorithm(InteriorColoringAlgorithm::PreperiodPeriod);
            //     }
            //     else if ui.button("Internal potential and Period").clicked()
            //     {
            //         self.interface
            //             .parent_mut()
            //             .select_preperiod_period_smooth_coloring();
            //         self.interface
            //             .child_mut()
            //             .select_preperiod_period_smooth_coloring();
            //     }
            //     else
            //     {
            //         return;
            //     }
            //     self.interface.consume_click();
            //     ui.close_menu();
            // });
        });
    }

    fn image_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Image", |ui| {
            self.menu_state.open();
            ui.menu_button("Set height", |ui| {
                if ui.button("256").clicked()
                {
                    self.interface.change_height(256);
                }
                else if ui.button("512").clicked()
                {
                    self.interface.change_height(512);
                }
                else if ui.button("768").clicked()
                {
                    self.interface.change_height(768);
                }
                else if ui.button("1024").clicked()
                {
                    self.interface.change_height(1024);
                }
                else if ui.button("1536").clicked()
                {
                    self.interface.change_height(1536);
                }
                else
                {
                    return;
                }
                self.interface.consume_click();
                ui.close_menu();
            });

            IMAGE_HOTKEYS.iter().for_each(|hotkey| {
                self.hotkey_button(ui, hotkey);
            });
        });
    }

    fn selection_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Selection", |ui| {
            self.menu_state.open();
            SELECTION_HOTKEYS.iter().for_each(|hotkey| {
                self.hotkey_button(ui, hotkey);
            });
        });
    }

    fn annotations_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Annotations", |ui| {
            self.menu_state.open();
            ANNOTATION_HOTKEYS.iter().for_each(|hotkey| {
                self.hotkey_button(ui, hotkey);
            });
        });
    }

    #[cfg(feature = "scripting")]
    fn transpiled_scripts_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("User Scripts", |ui| {
            if ui.button("Script Editor").clicked()
            {
                self.popup = Some(Popup::edit());
                ui.close_menu();
            }
            if ui.button("Load script...").clicked()
            {
                self.popup = Some(Popup::load());
                ui.close_menu();
            }
        });
    }

    #[cfg(feature = "scripting")]
    fn handle_popup_response(&mut self, response: Response)
    {
        use Response::*;
        match response
        {
            DoNothing =>
            {}
            Close =>
            {
                self.popup = None;
            }
            Load(path) =>
            {
                self.load_user_script(path);
                self.popup = None;
            }
        }
    }

    #[cfg(feature = "scripting")]
    fn should_update_interface(&self) -> bool
    {
        self.popup.is_none() && self.menu_state.is_closed()
    }

    #[cfg(not(feature = "scripting"))]
    fn should_update_interface(&self) -> bool
    {
        self.menu_state.is_closed()
    }

    fn polynomials_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Polynomials", |ui| {
            ui.set_max_width(250.);
            ui.menu_button("Quadratic Family", |ui| {
                dynamo_menu_button!(self, ui, "Base Curve", Mandelbrot);
                ui.menu_button("Marked Cycle", |ui| {
                    dynamo_menu_button_mc!(self, ui, Mandelbrot, 1);
                    dynamo_menu_button_mc!(self, ui, Mandelbrot, 3);
                    dynamo_menu_button_mc!(self, ui, Mandelbrot, 4);
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    dynamo_menu_button_mc!(self, ui, Mandelbrot, 1);
                    dynamo_menu_button_dyn!(self, ui, Mandelbrot, 2);
                    dynamo_menu_button_dyn!(self, ui, Mandelbrot, 3);
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    dynamo_menu_button_mis!(self, ui, Mandelbrot, 2, 1);
                    dynamo_menu_button_mis!(self, ui, Mandelbrot, 2, 2);
                    // dynamo_menu_button_mis!(self, ui, Mandelbrot, 3, 1);
                });
            });
            ui.menu_button("Cubic Family", |ui| {
                ui.menu_button("Real Slices", |ui| {
                    dynamo_menu_button!(self, ui, "Real critical point", RealCubicRealCrit);
                    dynamo_menu_button!(self, ui, "Imag critical point", RealCubicImagCrit);
                });
                ui.menu_button("Odd Cubics", |ui| {
                    dynamo_menu_button!(self, ui, "Base curve", OddCubic);
                    ui.menu_button("Marked Cycle", |ui| {
                        dynamo_menu_button_mc!(self, ui, OddCubic, 1);
                        dynamo_menu_button_mc!(self, ui, OddCubic, 2);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        dynamo_menu_button_dyn!(self, ui, OddCubic, 1);
                        dynamo_menu_button_dyn!(self, ui, OddCubic, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        dynamo_menu_button_mis!(self, ui, OddCubic, 1, 1);
                        dynamo_menu_button_mis!(self, ui, OddCubic, 1, 2);
                    });
                });
                ui.menu_button("Cubic Per(1)", |ui| {
                    dynamo_menu_button!(self, ui, "Base Curve", CubicPer1_0);
                    ui.menu_button("Marked Cycle", |ui| {
                        dynamo_menu_button_mc!(self, ui, CubicPer1_0, 1);
                        dynamo_menu_button_mc!(self, ui, CubicPer1_0, 2);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        dynamo_menu_button_dyn!(self, ui, CubicPer1_0, 1);
                        dynamo_menu_button_dyn!(self, ui, CubicPer1_0, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        dynamo_menu_button_mis!(self, ui, CubicPer1_0, 1, 1);
                    });
                });
                ui.menu_button("Cubic Per(2)", |ui| {
                    dynamo_menu_button!(self, ui, "Base curve", CubicPer2CritMarked);
                    ui.menu_button("Marked Cycle", |ui| {
                        dynamo_menu_button_mc!(self, ui, CubicPer2CritMarked, 1);
                        dynamo_menu_button_mc!(self, ui, CubicPer2CritMarked, 2);
                    });
                });
                dynamo_menu_button!(self, ui, "Per(3)", CubicPer3_0);
                ui.menu_button("Cubic Per(1, 1)", |ui| {
                    dynamo_menu_button!(self, ui, "Base Curve", CubicPer1_1);
                    ui.menu_button("Marked Cycle", |ui| {
                        dynamo_menu_button_mc!(self, ui, CubicPer1_1, 2);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        dynamo_menu_button_dyn!(self, ui, CubicPer1_1, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        dynamo_menu_button_mis!(self, ui, CubicPer1_1, 1, 1);
                    });
                });
                ui.menu_button("Cubic Per(1, λ)", |ui| {
                    dynamo_menu_button!(self, ui, "λ-plane", CubicPer1LambdaParam);
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.3",
                        CubicPer1Lambda,
                        with_param,
                        Cplx::from(0.3)
                    );
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.3 moduli",
                        CubicPer1LambdaModuli,
                        with_param,
                        Cplx::from(0.3)
                    );
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.2+0.7i moduli",
                        CubicPer1LambdaModuli,
                        with_param,
                        Cplx::new(0.2, 0.7)
                    );
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.99 moduli",
                        CubicPer1LambdaModuli,
                        with_param,
                        Cplx::from(0.99)
                    );
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.99i",
                        CubicPer1Lambda,
                        with_param,
                        Cplx::new(0., 0.99)
                    );
                });
                ui.menu_button("Per(2, λ)", |ui| {
                    dynamo_menu_button!(self, ui, "λ-plane", CubicPer2LambdaParam);
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.3",
                        CubicPer2Lambda,
                        with_param,
                        Cplx::from(0.3)
                    );
                    dynamo_menu_button!(
                        self,
                        ui,
                        "λ=0.99i",
                        CubicPer2Lambda,
                        with_param,
                        Cplx::new(0., 0.99)
                    );
                });
                ui.menu_button("2-cycle 0 <-> 1", |ui| {
                    dynamo_menu_button!(self, ui, "Base curve", CubicMarked2Cycle);
                    ui.menu_button("Marked Cycle", |ui| {
                        dynamo_menu_button_mc!(self, ui, CubicMarked2Cycle, 1);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        dynamo_menu_button_dyn!(self, ui, CubicMarked2Cycle, 2);
                    });
                    ui.menu_button("Marked Preperiodic Point", |ui| {
                        dynamo_menu_button_mis!(self, ui, CubicMarked2Cycle, 1, 1);
                        dynamo_menu_button_mis!(self, ui, CubicMarked2Cycle, 1, 2);
                    });
                });
            });
            ui.menu_button("Unicritical Maps: z -> c*(1+z/d)^d", |ui| {
                ui.menu_button("Degree 3", |ui| {
                    dynamo_menu_button!(self, ui, "Base curve", Unicritical<3>);
                    ui.menu_button("Marked Cycle", |ui| {
                        dynamo_menu_button_mc!(self, ui, Unicritical<3>, 1);
                        dynamo_menu_button_mc!(self, ui, Unicritical<3>, 2);
                        dynamo_menu_button_mc!(self, ui, Unicritical<3>, 3);
                    });
                    ui.menu_button("Marked Periodic Point", |ui| {
                        dynamo_menu_button_mc!(self, ui, Unicritical<3>, 1);
                        dynamo_menu_button_dyn!(self, ui, Unicritical<3>, 2);
                    });
                });
                seq!(D in 4..=8 {
                    dynamo_menu_button!(self, ui, format!("Degree {}", D), Unicritical<D>);
                });
            });
            #[allow(clippy::identity_op)]
            ui.menu_button("Chebyshev family: z -> (-1)^k * c * cheb2k(z/2)", |ui| {
                seq!(D in 1..=5 {
                    dynamo_menu_button!(self, ui, format!("Degree {}", 2*D), Chebyshev<D>);
                });
            });
            ui.menu_button("Biquadratic Maps", |ui| {
                dynamo_menu_button!(self, ui, "λ-plane", BiquadraticMultParam);
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=0.3",
                    BiquadraticMult,
                    with_param,
                    Cplx::from(0.3)
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=0.2+0.7j",
                    BiquadraticMult,
                    with_param,
                    Cplx::new(0.2, 0.7)
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=0.99i",
                    BiquadraticMult,
                    with_param,
                    Cplx::new(0., 0.99)
                );
                dynamo_menu_button!(self, ui, "Section (b=1): λ-plane", BiquadraticMultSection);
            });
        });
    }
    fn rational_maps_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Rational Maps", |ui| {
            ui.set_max_width(250.);
            ui.menu_button("QuadRat Per(2)", |ui| {
                dynamo_menu_button!(self, ui, "Moduli space", QuadRatPer2);
                dynamo_menu_button!(self, ui, "3-fold cover", QuadRatPer2Cover);
                ui.menu_button("Marked Cycle", |ui| {
                    dynamo_menu_button_mc!(self, ui, QuadRatPer2, 1);
                    dynamo_menu_button_mc!(self, ui, QuadRatPer2, 4);
                    dynamo_menu_button_mc!(self, ui, QuadRatPer2, 5);
                });
                ui.menu_button("Marked Periodic Point", |ui| {
                    dynamo_menu_button_mc!(self, ui, QuadRatPer2, 1);
                    dynamo_menu_button_dyn!(self, ui, QuadRatPer2, 3);
                    dynamo_menu_button_dyn!(self, ui, QuadRatPer2, 4);
                });
                ui.menu_button("Marked Preperiodic Point", |ui| {
                    dynamo_menu_button_mis!(self, ui, QuadRatPer2, 1, 1);
                    dynamo_menu_button_mis!(self, ui, QuadRatPer2, 2, 1);
                    dynamo_menu_button_mis!(self, ui, QuadRatPer2, 2, 2);
                });
            });
            ui.menu_button("QuadRat Per(3)", |ui| {
                dynamo_menu_button!(self, ui, "Base Curve", QuadRatPer3);
                ui.menu_button("Marked Cycle curves", |ui| {
                    dynamo_menu_button_mc!(self, ui, QuadRatPer3, 1);
                    dynamo_menu_button_mc!(self, ui, QuadRatPer3, 4);
                });
            });
            ui.menu_button("QuadRat Per(4)", |ui| {
                dynamo_menu_button!(self, ui, "Base Curve", QuadRatPer4);
                ui.menu_button("Marked Cycle curves", |ui| {
                    dynamo_menu_button_mc!(self, ui, QuadRatPer4, 3);
                });
            });
            dynamo_menu_button!(self, ui, "QuadRat Per(5)", QuadRatPer5);
            ui.menu_button("QuadRat Preper(2, 1)", |ui| {
                dynamo_menu_button!(self, ui, "Base Curve", QuadRatPreper21);
                ui.menu_button("Marked Cycle", |ui| {
                    dynamo_menu_button_mc!(self, ui, QuadRatPreper21, 3);
                    dynamo_menu_button_mc!(self, ui, QuadRatPreper21, 4);
                });
            });
            dynamo_menu_button!(self, ui, "QuadRat Preper(2, 2)", QuadRatPreper22);
            ui.menu_button("QuadRat Per(1, λ)", |ui| {
                dynamo_menu_button!(self, ui, "λ-plane", QuadRatPer1LambdaParam);
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=1",
                    QuadRatPer1_1
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=-1",
                    QuadRatPer1Lambda,
                    with_param,
                    -ONE
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=ω",
                    QuadRatPer1Lambda,
                    with_param,
                    OMEGA
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=i",
                    QuadRatPer1Lambda,
                    with_param,
                    Cplx::new(0., 1.)
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=exp(φτi)",
                    QuadRatPer1Lambda,
                    with_param,
                    Cplx::new(-0.737368878078320, 0.675490294261524)
                );
            });
            ui.menu_button("QuadRat Per(2, λ)", |ui| {
                dynamo_menu_button!(self, ui, "λ-plane", QuadRatPer2LambdaParam);
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=1",
                    QuadRatPer2Lambda,
                    with_param,
                    ONE
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=i",
                    QuadRatPer2Lambda,
                    with_param,
                    Cplx::new(0., 1.)
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=-3",
                    QuadRatPer2Lambda,
                    with_param,
                    Cplx::from(-3.)
                );
                dynamo_menu_button!(
                    self,
                    ui,
                    "λ=-27",
                    QuadRatPer2Lambda,
                    with_param,
                    Cplx::from(-27.)
                );
            });

            dynamo_menu_button!(self, ui, "QuadRat Symmetry Locus", QuadRatSymmetryLocus);
            dynamo_menu_button!(self, ui, "Newton Cubic", NewtonCubic);
            ui.menu_button("McMullen Family: z -> z^m + 1/(c*z^n)", |ui| {
                seq!(N in 2..=8 {
                    dynamo_menu_button!(self, ui, format!("(m=2, n={})", N), McMullenFamily<2, N>);
                });
                seq!(M in 2..=8 {
                    dynamo_menu_button!(self, ui, format!("(m={}, n={})", M, M), McMullenFamily<M, M>);
                });
            });
            ui.menu_button("Minsik Han Φ: z -> az/(z^d+d-1)", |ui| {
                seq!(D in 2..=8 {
                    dynamo_menu_button!(self, ui, format!("Degree {}", D), MinsikHanPhi<D>);
                });
            });
        });
    }

    fn transcendental_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Transcendental maps", |ui| {
            dynamo_menu_button!(self, ui, "z -> λexp(z)", Exponential);
            dynamo_menu_button!(self, ui, "z -> λcos(z)", Cosine);
            dynamo_menu_button!(self, ui, "z -> cos(z) + c", CosineAdd);
            dynamo_menu_button!(self, ui, "z -> sin(z) + z + τc", SineWander);
            dynamo_menu_button!(self, ui, "Riemann Xi Newton [SLOW!]", RiemannXi);
        });
    }

    fn non_analytic_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Non-analytic maps", |ui| {
            ui.menu_button("Tricorne", |ui| {
                seq!(D in 2..=5 {
                    dynamo_menu_button!(self, ui, format!("Degree {}", D), Tricorne<D>);
                });
            });
            ui.menu_button("Burning Ship", |ui| {
                seq!(D in 2..=5 {
                    dynamo_menu_button!(self, ui, format!("Degree {}", D), BurningShip<D>);
                });
            });
            dynamo_menu_button!(self, ui, "Sailboat Param", SailboatParam);
            dynamo_menu_button!(self, ui, "Rulkov Map", Rulkov);
        });
    }

    fn change_fractal<P, J, C, M, T>(&mut self, create_plane: fn() -> P, create_child: fn(P) -> J)
    where
        P: Displayable + Clone + 'static,
        J: Displayable + ParameterPlane<MetaParam = M, Child = C> + Clone + 'static,
        C: Displayable + From<J>,
        M: ParamList<Param = T>,
        T: From<P::Param> + std::fmt::Display,
    {
        use dynamo_gui::interface::PanePair;
        let image_height = self.interface.get_image_height();
        let max_iters = 1024;

        let parent_plane = create_plane()
            .with_max_iter(max_iters)
            .with_res_y(image_height);
        let child_plane = create_child(parent_plane.clone());

        let mut interface = MainInterface::new(parent_plane, child_plane, image_height);
        interface.update_panes();
        self.interface = Box::new(interface);
    }

    #[cfg(feature = "scripting")]
    fn load_user_script<P: AsRef<Path>>(&mut self, script_path: P)
    {
        use script_loader::Loader;
        let image_height = self.interface.get_image_height();
        let loader = Loader::new(script_path.as_ref(), image_height);
        unsafe {
            match loader.run()
            {
                Ok(int) =>
                {
                    self.interface = Box::new(int);
                }
                Err(e) =>
                {
                    println!("Error loading user profile: {:?}", e);
                }
            }
        }
    }

    fn hotkey_button(&mut self, ui: &mut Ui, hotkey: &Hotkey)
    {
        if let Some(action) = hotkey.menu_action()
        {
            if ui
                .add(
                    egui::Button::new(action.short_description())
                        .shortcut_text(hotkey.shortcut_text().unwrap_or_default()),
                )
                .clicked()
            {
                self.interface.process_action(action);
                self.interface.consume_click();
                ui.close_menu();
            }
        }
    }

    #[cfg(feature = "scripting")]
    fn show_popup(&mut self, ui: &mut Ui)
    {
        if let Some(popup) = self.popup.as_mut()
        {
            popup.show(ui.ctx());
            let response = popup.pop_response();
            self.handle_popup_response(response);
        }
    }
}

impl Default for FractalTab
{
    fn default() -> Self
    {
        type Profile = Mandelbrot;

        let height = 768;

        let parent_plane = Profile::default().with_res_y(height).with_max_iter(1024);
        let child_plane = <Profile as ParameterPlane>::Child::from(parent_plane.clone());

        let interface = Box::new(MainInterface::new(parent_plane, child_plane, height));

        Self {
            interface,
            menu_state: Default::default(),
            id: TabID::default(),
            #[cfg(feature = "scripting")]
            popup: None,
        }
    }
}
