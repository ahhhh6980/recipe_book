use crate::MixedRational;

/*
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Country {
Us,
Uk,
None,
}*/

pub enum UnitType {
    Imperial,
    Metric,

    UsCustomary,
    UsLegal,

    Traditional,
}

#[derive(Clone, Copy, Debug)]
pub struct MeasureType {
    pub count: MixedRational,
    pub unit: Measure,
}

impl std::ops::Mul<MixedRational> for MeasureType {
    type Output = MeasureType;
    fn mul(self, rhs: MixedRational) -> Self::Output {
        if self.unit.fluid {
            MeasureType {
                count: self.count * rhs,
                unit: self.unit,
            }
            .simplify(|m| m.fluid && m.unit as u8 != Unit::Gill as u8)
        } else {
            MeasureType {
                count: self.count * rhs,
                unit: self.unit,
            }
            .simplify(|m| !m.fluid && m.unit as u8 != Unit::Gill as u8)
        }
    }
}

#[allow(unused)]
impl MeasureType {
    pub fn convert(&self, new_unit: Unit) -> Option<Self> {
        let new_unit = Measure::from_enum(new_unit);
        if let Some(new_count) = self.unit.convert(self.count, new_unit) {
            Some(MeasureType {
                count: new_count,
                unit: new_unit,
            })
        } else {
            None
        }
    }
    pub fn new(name: String, count: MixedRational) -> Self {
        MeasureType {
            count,
            unit: Measure::new(name),
        }
    }
    pub fn memory_size(&self) -> usize {
        std::mem::size_of_val(&self.count) + std::mem::size_of_val(&self.unit)
    }
    pub fn simplify(&self, filter: fn(&Measure) -> bool) -> Self {
        let unit = self.unit;
        let test = STANDARD_COOKING_MEASUREMENT_UNITS
            .iter()
            .filter(|f| filter(f) && (f.unit as u8 != Unit::Other as u8))
            .min_by_key(|x| {
                if let Some(r) = unit.convert(self.count, **x) {
                    let (val, den, num) = (
                        (r.value as f64).log10().ceil() as i32,
                        (r.den as f64).log10().ceil() as i32,
                        (r.num as f64).log10().ceil() as i32,
                    );

                    let add = (r >= self.count) as i32
                        + if (val > 2) { val } else { 0 }
                        + if (num > 2) { num } else { 0 }
                        + if (den > 2) { den } else { 0 };

                    (if r.value != 0 && r.den != 0 {
                        r.value * r.num + r.den as i32
                    } else if r.den != 0 {
                        r.num + r.den as i32
                    } else {
                        r.value
                    }) + add
                } else {
                    i32::MAX
                }
            });
        let (new_count, new_unit) = if let Some(conversion) = test {
            if let Some(c) = unit.convert(self.count, *conversion) {
                (c, *conversion)
            } else {
                (self.count, self.unit)
            }
        } else {
            (self.count, self.unit)
        };
        MeasureType {
            count: new_count,
            unit: new_unit,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Measure {
    pub names: &'static [&'static str],
    pub unit: Unit,
    pub fluid: bool,
    // ,
}

impl std::fmt::Display for Measure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.names[0])
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Unit {
    // Spices
    Drop,
    Smidgen,
    Pinch,
    Dash,
    Tad,

    // Us & ??
    Teaspoon,
    Tablespoon,
    Cup,

    // Metric
    Milliliter,
    Liter,

    // Mass
    Ounce,
    Pound,
    Gram,
    Kilogram,

    Deciliter,

    FluidOunce,
    Gill,
    Pint,
    Quart,
    Gallon,
    Milligram,

    Other,
}
/*
pub fn min_measurements(measure: MeasureType) -> Vec<MeasureType> {
    use Unit::*;
    let as_cups = measure.convert(Cup).unwrap();
    let as_tablespoons = measure.convert(Tablespoon).unwrap();
    let as_teaspoons = measure.convert(Teaspoon).unwrap();
    let cup_to_table_spoon = MixedRational::whole(16);
    let cup_to_teaspoon = MixedRational::whole(48);
}*/

pub fn get_standard_size(unit: Unit) -> Vec<MixedRational> {
    match unit {
        Unit::Cup => {
            MixedRational::from_frac_list(vec![[1, 8], [1, 4], [1, 3], [1, 2], [3, 4], [1, 1]])
        }
        Unit::Teaspoon => {
            MixedRational::from_frac_list(vec![[1, 8], [1, 4], [1, 3], [1, 2], [3, 4], [1, 1]])
        }
        Unit::Drop => MixedRational::from_frac_list(vec![[1, 1]]),
        Unit::Smidgen => MixedRational::from_frac_list(vec![[1, 1]]),
        Unit::Pinch => MixedRational::from_frac_list(vec![[1, 1]]),
        Unit::Dash => MixedRational::from_frac_list(vec![[1, 1]]),
        Unit::Tad => MixedRational::from_frac_list(vec![[1, 1]]),
        _ => MixedRational::from_frac_list(vec![]),
    }
}

#[allow(unused)]
#[rustfmt::skip]
// https://en.wikibooks.org/wiki/Cookbook:Units_of_measurement
pub const STANDARD_COOKING_MEASUREMENT_UNITS: &[Measure] = &[
    Measure {fluid: true, names: &["ml", "milliliter", "millilitre", "cc", "mL"], unit: Unit::Milliliter,  },
    Measure {fluid: true, names: &["liter", "litre", "l", "L"], unit: Unit::Liter,},
    Measure {fluid: true, names: &["dl", "deciliter", "decilitre", "dL"], unit: Unit::Deciliter,  },
    Measure {fluid: false, names: &["tsp", "teaspoon", "t"], unit: Unit::Teaspoon,},
    Measure {fluid: false, names: &["tbsp", "tablespoon", "tbs", "tbl", "T"], unit: Unit::Tablespoon,   },
    Measure {fluid: true, names: &["fl oz", "fluid oz"], unit: Unit::FluidOunce,   },
    Measure {fluid: false, names: &["gill"], unit: Unit::Gill,},
    Measure {fluid: false, names: &["cup", "c"], unit: Unit::Cup,},
    Measure {fluid: true, names: &["pint", "fl pt", "pt", "p"], unit: Unit::Pint,},
    Measure {fluid: true, names: &["quart", "fl qt", "qt", "q"], unit: Unit::Quart,   },
    Measure {fluid: true, names: &["gal", "gallon", "g"], unit: Unit::Gallon, },
    Measure {fluid: false, names: &["mg", "milligram", "milligramme"], unit: Unit::Milligram, },
    Measure {fluid: false, names: &["kg", "kilogram", "kilogramme"], unit: Unit::Kilogram,   },
    Measure {fluid: false, names: &["g", "gram", "gramme"], unit: Unit::Gram,},
    Measure {fluid: false, names: &["lb", "pound"], unit: Unit::Pound,},
    Measure {fluid: false, names: &["oz", "ounce"], unit: Unit::Ounce,},

    Measure {fluid: false, names: &["tad"], unit: Unit::Tad,},
    Measure {fluid: false, names: &["dash"], unit: Unit::Dash,},
    Measure {fluid: false, names: &["pinch"], unit: Unit::Pinch,},
    Measure {fluid: false, names: &["smidgen"], unit: Unit::Smidgen,},
    Measure {fluid: false, names: &["drop"], unit: Unit::Drop,},

];

impl Measure {
    pub fn new(name: String) -> Self {
        if !name.is_empty() {
            for i in 0..5 {
                for m in STANDARD_COOKING_MEASUREMENT_UNITS {
                    if i >= m.names.len() {
                        break;
                    }
                    if name.contains(m.names[i]) {
                        return *m;
                    }
                }
            }
        }
        Measure {
            names: &[""],
            unit: Unit::Other,
            fluid: false,
            //Country::None,
        }
    }
    pub fn from_enum(unit: Unit) -> Self {
        match unit {
            Unit::Milliliter => STANDARD_COOKING_MEASUREMENT_UNITS[0],
            Unit::Liter => STANDARD_COOKING_MEASUREMENT_UNITS[1],
            Unit::Deciliter => STANDARD_COOKING_MEASUREMENT_UNITS[2],
            Unit::Teaspoon => STANDARD_COOKING_MEASUREMENT_UNITS[3],
            Unit::Tablespoon => STANDARD_COOKING_MEASUREMENT_UNITS[4],
            Unit::FluidOunce => STANDARD_COOKING_MEASUREMENT_UNITS[5],
            Unit::Gill => STANDARD_COOKING_MEASUREMENT_UNITS[6],
            Unit::Cup => STANDARD_COOKING_MEASUREMENT_UNITS[7],
            Unit::Pint => STANDARD_COOKING_MEASUREMENT_UNITS[8],
            Unit::Quart => STANDARD_COOKING_MEASUREMENT_UNITS[9],
            Unit::Gallon => STANDARD_COOKING_MEASUREMENT_UNITS[10],
            Unit::Milligram => STANDARD_COOKING_MEASUREMENT_UNITS[11],
            Unit::Kilogram => STANDARD_COOKING_MEASUREMENT_UNITS[12],
            Unit::Gram => STANDARD_COOKING_MEASUREMENT_UNITS[13],
            Unit::Pound => STANDARD_COOKING_MEASUREMENT_UNITS[14],
            Unit::Ounce => STANDARD_COOKING_MEASUREMENT_UNITS[15],
            _ => Measure {
                names: &[""],
                unit: Unit::Other,
                fluid: false,
                //Country::None,
            },
        }
    }
    #[allow(clippy::match_single_binding)]
    pub fn conversion_table(a: &Measure, b: &Measure) -> Option<MixedRational> {
        use Unit::*;
        let one = MixedRational::whole(1);
        if a.unit as u8 == b.unit as u8 {
            return Some(one);
        }
        // Return how many of A are in B
        // or, B per A
        match a.unit {
            // Fluids
            Milliliter => match b.unit {
                Unit::Liter => Some(MixedRational::whole(1000)),
                Unit::Deciliter => Some(MixedRational::whole(100)),
                Unit::Teaspoon => Some(MixedRational::fract(157725491, 32000000).approx_ratio(2)),
                Unit::Tablespoon => Some(MixedRational::fract(473176473, 32000000).approx_ratio(2)),
                Unit::FluidOunce => Some(MixedRational::fract(473176473, 16000000).approx_ratio(2)),
                Unit::Gill => Some(MixedRational::fract(473176473, 4000000).approx_ratio(2)),
                Unit::Cup => Some(MixedRational::fract(473176473, 2000000).approx_ratio(2)),
                Unit::Pint => Some(MixedRational::fract(473176473, 1000000).approx_ratio(2)),
                Unit::Quart => Some(MixedRational::fract(473176473, 500000).approx_ratio(2)),
                Unit::Gallon => Some(MixedRational::fract(473176473, 125000).approx_ratio(2)),
                _ => None,
            },
            Teaspoon => match b.unit {
                Unit::Milliliter => Some(MixedRational::fract(32000000, 157725491).approx_ratio(3)),
                Unit::Liter => Some(MixedRational::new(202, 884136, 1000000).approx_ratio(2)),
                Unit::Deciliter => Some(MixedRational::new(20, 2884136, 10000000).approx_ratio(2)),
                Unit::Tablespoon => Some(MixedRational::whole(3)),
                Unit::FluidOunce => Some(MixedRational::whole(6)),
                Unit::Gill => Some(MixedRational::whole(24)),
                Unit::Cup => Some(MixedRational::whole(48)),
                Unit::Pint => Some(MixedRational::whole(96)),
                Unit::Quart => Some(MixedRational::whole(192)),
                Unit::Gallon => Some(MixedRational::whole(768)),
                _ => None,
            },
            Tablespoon => match b.unit {
                Unit::Milliliter => Some(MixedRational::fract(32000000, 473176473).approx_ratio(3)),
                Unit::Liter => Some(MixedRational::new(67, 6280454, 10000000).approx_ratio(2)),
                Unit::Deciliter => Some(MixedRational::new(6, 76280454, 100000000).approx_ratio(2)),
                Unit::Tablespoon => Some(MixedRational::fract(1, 3)),
                Unit::FluidOunce => Some(MixedRational::whole(2)),
                Unit::Gill => Some(MixedRational::whole(8)),
                Unit::Cup => Some(MixedRational::whole(16)),
                Unit::Pint => Some(MixedRational::whole(32)),
                Unit::Quart => Some(MixedRational::whole(64)),
                Unit::Gallon => Some(MixedRational::whole(256)),
                _ => None,
            },
            Cup => match b.unit {
                Unit::Milliliter => Some(MixedRational::fract(2000000, 473176473).approx_ratio(2)),
                Unit::Liter => Some(MixedRational::new(4, 22675284, 100000000).approx_ratio(2)),
                Unit::Deciliter => Some(MixedRational::fract(200000000, 473176473).approx_ratio(2)),
                Unit::Teaspoon => Some(MixedRational::fract(1, 48)),
                Unit::Tablespoon => Some(MixedRational::fract(1, 16)),
                Unit::FluidOunce => Some(MixedRational::fract(1, 8)),
                Unit::Gill => Some(MixedRational::fract(1, 2)),
                Unit::Pint => Some(MixedRational::whole(2)),
                Unit::Quart => Some(MixedRational::whole(4)),
                Unit::Gallon => Some(MixedRational::whole(16)),
                _ => None,
            },
            // Dry
            Gram => match b.unit {
                Unit::Milligram => Some(MixedRational::fract(1, 1000)),
                Unit::Kilogram => Some(MixedRational::whole(1000)),
                Unit::Pound => Some(MixedRational::new(453, 74, 125).approx_ratio(2)),
                Unit::Ounce => Some(MixedRational::new(28, 699, 2000).approx_ratio(2)),
                _ => None,
            },
            Milligram => match b.unit {
                Unit::Gram => Some(MixedRational::whole(1000)),
                Unit::Kilogram => Some(MixedRational::whole(1000000)),
                Unit::Pound => Some(MixedRational::whole(453592)),
                Unit::Ounce => Some(MixedRational::new(28349, 1, 2)),
                _ => None,
            },
            Kilogram => match b.unit {
                Unit::Milligram => Some(MixedRational::fract(1, 1000000)),
                Unit::Gram => Some(MixedRational::fract(1, 1000)),
                Unit::Pound => Some(MixedRational::fract(56699, 125000).approx_ratio(3)),
                Unit::Ounce => Some(MixedRational::fract(56699, 2000000).approx_ratio(5)),
                _ => None,
            },
            Pound => match b.unit {
                Unit::Milligram => Some(MixedRational::fract(551, 250000000)),
                Unit::Gram => Some(MixedRational::fract(110231, 50000000).approx_ratio(2)),
                Unit::Kilogram => Some(MixedRational::new(2, 10231, 50000).approx_ratio(2)),
                Unit::Ounce => Some(MixedRational::fract(1, 16)),
                _ => None,
            },
            Ounce => match b.unit {
                Unit::Gram => Some(MixedRational::fract(17637, 500000).approx_ratio(3)),
                Unit::Milligram => Some(MixedRational::fract(17637, 500000000)),
                Unit::Kilogram => Some(MixedRational::new(35, 137, 500).approx_ratio(2)),
                Unit::Pound => Some(MixedRational::whole(16)),
                _ => None,
            },
            _ => None,
        }
    }
    #[allow(unused)]
    pub fn convert(&self, quantity: MixedRational, new_measure: Measure) -> Option<MixedRational> {
        Measure::conversion_table(self, &new_measure).map(|scale| quantity / scale)
    }
    #[allow(unused)]
    pub fn convert_universal(
        &self,
        quantity: MixedRational,
        new_measure: Measure,
        density: Option<MixedRational>,
    ) -> Option<MixedRational> {
        if self.fluid == new_measure.fluid {
            Measure::conversion_table(self, &new_measure).map(|scale| quantity / scale)
        } else if let Some(d) = density {
            if self.fluid {
                if let Some(new_quantity) =
                    self.convert(quantity, Measure::from_enum(Unit::FluidOunce))
                {
                    let ounces = new_quantity * MixedRational::new(1, 43176, 1000000) * d;
                    self.convert(ounces, new_measure)
                } else {
                    None
                }
            } else if let Some(new_quantity) =
                self.convert(quantity, Measure::from_enum(Unit::Ounce))
            {
                let fluid_ounces = new_quantity / (MixedRational::new(1, 43176, 1000000) * d);
                self.convert(fluid_ounces, new_measure)
            } else {
                None
            }
        } else {
            None
        }
    }
}
