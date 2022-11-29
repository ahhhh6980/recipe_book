use std::fmt::{self, Display};


#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MixedRational {
    #[serde(skip_serializing_if = "MixedRational::skip", default="MixedRational::zero")]
    pub value: i32,
    #[serde(skip_serializing_if = "MixedRational::skip", default="MixedRational::zero")]
    pub num: i32,
    #[serde(skip_serializing_if = "MixedRational::skip", default="MixedRational::zero")]
    pub den: u32,
}


impl MixedRational {
    pub fn from_string(s: String) -> (Self, Option<Self>) {
        fn replace_unicode(s: String) -> String {
            let mut new_s = s;
            for (c, new) in "¼½¾⅐⅑⅒⅓⅔⅕⅖⅗⅘⅙⅚⅛⅜⅝⅞".chars().zip(
                ["1/4", "1/2", "3/4", "1/7", "1/9", "1/10", "1/3", "2/3", "1/5", 
                 "2/5", "3/5", "4/5", "1/6", "5/6", "1/8",  "3/8", "5/8", "7/8"]) {
                new_s = new_s.replace(c, &format!(" {}", new))
            }
            new_s
        }
        let valid = Self::valid_chars();
        let mut txt = replace_unicode(s.to_lowercase().chars().filter(|x| valid.contains(*x) || valid.contains("to")).collect());
        if let Some(period) = txt.find('.') {
            let ( l,  r) = txt.split_at(period);
            let value = l.replace(|c: char| !c.is_ascii_digit(), "");
            let decimal = r.replace(|c: char| !c.is_ascii_digit(), "");
            let dec_l = decimal.len() as u32;
            return (MixedRational::new(value.parse::<i32>().unwrap(), decimal.parse::<i32>().unwrap(), 10u32.pow(dec_l)), None);
        }
        // Fix spaces :)
        txt = txt.trim().split(' ')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if txt.is_empty() {return (Self::default(), None)}
        while txt.as_bytes()[0] == b' ' {
            txt.remove(0);
        }

        let (a, b): (String, String) = if let Some(dash) = txt.find('-') {
            let (a,b) = txt.split_at(dash);
            (a.into(), b[1..].into())
        } else if let Some(to) = txt.find("to") {
            let (a,b) = txt.split_at(to);
            (a.into(), b[2..].into())
        } else {
            (txt, "".into())
        };
        if let Some(f) = parse_fract(a) {
            (f, parse_fract(b))
        } else {
            (MixedRational::whole(0), None)
        }
    }
    pub fn valid_chars() -> String {
        "-1234567890 /¼½¾⅐⅑⅒⅓⅔⅕⅖⅗⅘⅙⅚⅛⅜⅝⅞.".into()
    }
    pub fn new(w: i32, n: i32, d: u32) -> Self {
        MixedRational {
            value: w,
            num: n,
            den: d,
        }.simplify()
    }
    pub fn fract(n: i32, d: u32) -> Self {
        MixedRational {
            value: 0,
            num: n,
            den: d,
        }.simplify()
    }
    pub fn whole(v: i32) -> Self {
        MixedRational {
            value: v,
            num: 0,
            den: 0,
        }
    }

    pub fn from_frac_list(l: Vec<[i32;2]>) -> Vec<Self> {
        l.iter().map(|f| MixedRational::fract(f[0], f[1] as u32)).collect()
    }
    pub fn to_float(self) -> f32 {
        self.into()
    }
    fn gcd32(a: i32, b: i32) -> i32 {
        let (mut a, mut b) = (a, b);
        let mut r = 1;
        while r > 0 {
            r = a % b;
            a = b;
            b = r;
        }
        a
    }
    #[rustfmt::skip]
    pub fn simplify(&self) -> Self {
        let sign = [1, -1][(self.value < 0) as usize]
                 * [1, -1][  (self.num < 0) as usize];
        let mut fract = if self.den != 0 {
            (self.num.abs() + (self.den as i32 * self.value.abs()), self.den as i32)
        } else {
            (self.value.abs(), 1)
        };
        let r = Self::gcd32(fract.0, fract.1);
        fract.0 /= r;
        fract.1 /= r;
        let whole = fract.0 / fract.1;
        let new_num = fract.0 - (fract.1 * whole);
        if new_num == 0 {
            return MixedRational {
                value: whole,
                num: 0,
                den: 0,
            };
        } 
        if whole == 0 {
            return MixedRational {
                value: 0,
                num: new_num * sign,
                den: fract.1 as u32,
            };
        }
        MixedRational {
            value: whole,
            num: new_num,
            den: fract.1 as u32,
        }
    }
    pub fn approx_ratio_scaled(&self, digits: usize, step_scale: f32) -> Self {
        let mut new_num = self.num.abs() as f32;
        let mut new_den = self.den as f32;
        let (mut i, mut div) = (0, 1.);
        while (new_num.log10().ceil() as usize > digits || new_den.log10().ceil() as usize > digits)
            && i < 1000000
        {
            i += 1;
            div *= step_scale;
            new_num = (self.num / (div as i32)) as f32;
            new_den = (self.den / (div as u32)) as f32;
            if new_num as u32 == 0 || new_den as u32 == 0 {
                div /= step_scale;
                break;
            }
        }
        MixedRational::new(self.value, self.num / (div as i32), self.den / (div as u32))
    }
    pub fn approx_ratio_error_scaled(&self, digits: usize, step_scale: f32) -> (Self, f32) {
        let approx = self.approx_ratio_scaled(digits, step_scale);
        (approx, self.to_float() - approx.to_float())
    }
    pub fn approx_ratio_error(&self, digits: usize) -> (Self, f32) {
        let approx = self.approx_ratio_scaled(digits, 1.1);
        (approx, self.to_float() - approx.to_float())
    }
    pub fn approx_ratio(&self, digits: usize) -> Self {
        self.approx_ratio_scaled(digits, 1.1)
    }
    
    pub fn memory_size(&self) -> usize {
        std::mem::size_of_val(&self.value)
            + std::mem::size_of_val(&self.num)
            + std::mem::size_of_val(&self.den)
    }

    // Serde Stuff
    fn skip<T>(v: &T) -> bool where T: Default+PartialEq {*v == T::default()}
    fn zero<T>() -> T where T: Default {T::default()}
}

impl Display for MixedRational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.den != 0, self.value != 0) {
            (true, true) => write!(f, "{} {}/{}", self.value, self.num, self.den),
            (true, false) => write!(f, "{}/{}", self.num, self.den),
            (false, true) => write!(f, "{}", self.value),
            (false, false) => write!(f, ""),
        }
    }
}

#[allow(clippy::blocks_in_if_conditions)]
pub fn parse_fract(txt: String) -> Option<MixedRational> {
    if let Some(slash) = txt.find('/') {
        let mut mr = MixedRational::new(0,0,0);
        // Split the left and denom part from the fraction
        let (left_s, den_s) = txt.split_at(slash);
        // Parse denominator
        let filtered_den = den_s.chars().filter(|x| x.is_numeric()).collect::<String>();
        if filtered_den.parse::<u32>().map(|d| {mr.den = d;}).is_err() {
            return None;
        }
        // of form: "v n/d"
        if let Some(space) = txt.find(' ') {
            // Split value and numerator from left
            let (value_s, num_s) = left_s.split_at(space);
            // Parse value
            let filtered_value = value_s.chars().filter(|x| x.is_numeric()).collect::<String>();
            if filtered_value.parse::<i32>().map(|v| {mr.value = v;}).is_ok() {
                // Parse numerator
                let filtered_num = num_s.chars().filter(|x| x.is_numeric()).collect::<String>();
                if filtered_num.parse::<i32>().map(|n| {mr.num = n;}).is_ok() {
                    return Some(mr);
                }
            }
        // of form: "n/d"
        } else {
            // Parse numerator
            let filtered_num = left_s.chars().filter(|x| x.is_numeric()).collect::<String>();
            if filtered_num.parse::<i32>().map(|n| {mr.num = n;}).is_ok() {
                return Some(mr)
            }
        }
    } else {
        // Parse value
        let filtered_value = txt.chars().filter(|x| x.is_numeric()).collect::<String>();
        if let Ok(value) = filtered_value.parse::<i32>() {
            return Some(MixedRational::whole(value))
        } 
    }
    None
}


impl From<i32> for MixedRational {
    fn from(v: i32) -> Self {
        Self::whole(v)
    }
}

#[allow(clippy::from_over_into)]
impl Into<f32> for MixedRational {
    fn into(self) -> f32 {
        if self.value != 0 && self.den!= 0 {
            (self.value as f32)  + (self.num as f32 / self.den as f32)
        } else if self.den!= 0 {
            self.num as f32 / self.den as f32
        } else {
            self.value as f32
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for MixedRational {
    fn into(self) -> String {
        match (self.den != 0, self.value != 0) {
            (true, true) => format!("{} {}/{}", self.value, self.num, self.den),
            (true, false) => format!("{}/{}", self.num, self.den),
            (false, true) => format!("{}", self.value),
            (false, false) => format!("{:?}", self),
        }
    }
}

impl PartialOrd for MixedRational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.den != 0 {
            (self.den as i32 * other.den as i32 * self.num + other.value * self.den as i32 * other.den as i32)
            .partial_cmp(&(self.den as i32* other.den as i32 * other.num + self.value *self.den as i32 * other.den as i32))
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
}
impl Ord for MixedRational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.den != 0 {
            (self.den as i32 * other.den as i32 * self.num + other.value * self.den as i32 * other.den as i32)
            .cmp(&(self.den as i32* other.den as i32 * other.num + self.value *self.den as i32 * other.den as i32))
        } else {
            self.value.cmp(&other.value)
        }
    }
}

impl std::ops::Mul<u32> for MixedRational {
    type Output = MixedRational;
    fn mul(self, rhs: u32) -> Self::Output {
        MixedRational {
            value: self.value * (rhs as i32),
            num: self.num * (rhs as i32),
            den: self.den,
        }
        .simplify()
    }
}
// (if self.num != 0 { self.num.abs() } else { 0 })
// [0, self.num.abs()][self.num != 0 as usize]
impl std::ops::Mul<MixedRational> for MixedRational {
    type Output = MixedRational;
    #[rustfmt::skip]
    fn mul(self, rhs: MixedRational) -> Self::Output {
        // Get the sign
        let sign = [1, -1][(self.value < 0) as usize] * [1, -1][(self.num < 0) as usize]
                 * [1, -1][(rhs.value < 0) as usize] * [1, -1][(rhs.num < 0) as usize];
        // Convert to fractions
        let frac_l = [
            (self.value.abs(), 1),
            (self.num.abs() + (self.den as i32 * self.value.abs()), self.den as i32),
        ][(self.den != 0) as usize];
        let frac_r = [
            (rhs.value.abs(), 1),
            (rhs.num.abs() + (rhs.den as i32 * rhs.value.abs()), rhs.den as i32),
        ][(rhs.den != 0) as usize];
        // Return
        MixedRational::fract(frac_l.0 * frac_r.0 * sign, (frac_r.1 * frac_l.1) as u32)
    }
}

impl std::ops::Div<MixedRational> for MixedRational {
    type Output = MixedRational;
    fn div(self, rhs: MixedRational) -> Self::Output {
        // Get the sign
        let sign = [1, -1][(self.value < 0) as usize]
            * [1, -1][(self.num < 0) as usize]
            * [1, -1][(rhs.value < 0) as usize]
            * [1, -1][(rhs.num < 0) as usize];
        // Convert to fractions
        let frac_l = [
            (self.value.abs(), 1),
            (
                self.num.abs() + (self.den as i32 * self.value.abs()),
                self.den as i32,
            ),
        ][(self.den != 0) as usize];
        let frac_r = [
            (rhs.value.abs(), 1),
            (
                rhs.num.abs() + (rhs.den as i32 * rhs.value.abs()),
                rhs.den as i32,
            ),
        ][(rhs.den != 0) as usize];
        // Return
        MixedRational::fract(frac_l.0 * frac_r.1 * sign, (frac_l.1 * frac_r.0) as u32)
    }
}
