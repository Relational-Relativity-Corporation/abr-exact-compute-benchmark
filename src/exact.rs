//! Exact rational measurement values.
//!
//! An [`Exact`] is a reduced ratio of `i128` integers with a positive
//! denominator. Its JSON form is a string holding one of
//!
//! * an integer: `"-12"`
//! * a decimal: `"0.1235"` (exactly 1235/10000)
//! * a ratio: `"247/2000"`
//!
//! The only sign allowed is a leading `-`. Exponents and whitespace are not
//! allowed. The text is kept as recorded and written back unchanged, so
//! record identity never depends on binary64. Two spellings of one value
//! (`"0.5"`, `"1/2"`) are equal values but different text.
//!
//! * **Checked arithmetic:** a result that does not fit `i128` is `None`.
//! * **Kernel boundary conversion:** the binary64 nearest the value, ties to
//!   even. [`Exact::f64_loss`] gives the exact loss.
//!
//! Two storage policies are provided so their costs can be measured
//! separately:
//!
//! * **Shared:** `clone` shares the text (reference counted) and the
//!   binary64 value, which is computed once.
//! * **Owned:** [`Exact::deep_copy`] allocates a new text and an empty cache,
//!   and [`Exact::to_f64_uncached`] recomputes the conversion each time.

use std::cmp::Ordering;
use std::fmt;
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Exact {
    text: Arc<str>,
    num: i128,
    den: i128,
    nearest: OnceLock<f64>,
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

impl Exact {
    /// Parse the recorded text.
    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = || format!("'{text}' is not an integer, decimal or ratio");
        let wide = || format!("'{text}' does not fit i128");
        let (negative, body) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let (num, den): (i128, i128) = match (body.split_once('/'), body.split_once('.')) {
            (Some((n, d)), None) if all_digits(n) && all_digits(d) => {
                (n.parse().map_err(|_| wide())?, d.parse().map_err(|_| wide())?)
            }
            (None, Some((i, f))) if all_digits(i) && all_digits(f) => {
                let scale = 10i128.checked_pow(f.len() as u32).ok_or_else(wide)?;
                (format!("{i}{f}").parse().map_err(|_| wide())?, scale)
            }
            (None, None) if all_digits(body) => (body.parse().map_err(|_| wide())?, 1),
            _ => return Err(bad()),
        };
        if den == 0 {
            return Err(format!("'{text}' has a zero denominator"));
        }
        let mut e = Self::reduce(if negative { -num } else { num }, den).ok_or_else(wide)?;
        e.text = Arc::from(text);
        Ok(e)
    }

    fn reduce(num: i128, den: i128) -> Option<Self> {
        if den == 0 || num == i128::MIN || den == i128::MIN {
            return None;
        }
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()).max(1) as i128;
        let sign = den.signum();
        Some(Exact { text: Arc::from(""), num: sign * num / g, den: sign * den / g, nearest: OnceLock::new() })
    }

    /// num/den reduced, with canonical text `"n"` or `"n/d"`.
    pub fn from_ratio(num: i128, den: i128) -> Option<Self> {
        let mut e = Self::reduce(num, den)?;
        e.text = Arc::from(if e.den == 1 { e.num.to_string() } else { format!("{}/{}", e.num, e.den) });
        Some(e)
    }

    /// The exact value of a finite, normal binary64. None if it does not fit i128.
    pub fn from_f64(x: f64) -> Option<Self> {
        if x == 0.0 {
            return Self::from_ratio(0, 1);
        }
        if !x.is_normal() {
            return None;
        }
        let bits = x.to_bits();
        let mut mantissa = (bits & ((1 << 52) - 1)) | (1 << 52);
        let mut exp2 = ((bits >> 52) & 0x7ff) as i32 - 1075;
        let tz = mantissa.trailing_zeros();
        mantissa >>= tz;
        exp2 += tz as i32;
        let signed = if x < 0.0 { -(mantissa as i128) } else { mantissa as i128 };
        let width = 64 - mantissa.leading_zeros() as i32;
        match exp2 {
            e if e >= 0 && width + e <= 126 => Self::from_ratio(signed << e, 1),
            e if e < 0 && -e <= 126 => Self::from_ratio(signed, 1i128 << -e),
            _ => None,
        }
    }

    pub fn text(&self) -> &str { &self.text }
    pub fn numer(&self) -> i128 { self.num }
    pub fn denom(&self) -> i128 { self.den }
    pub fn signum(&self) -> i8 { self.num.signum() as i8 }
    pub fn is_zero(&self) -> bool { self.num == 0 }

    /// A copy with newly allocated text and no cached conversion (the owned policy).
    pub fn deep_copy(&self) -> Self {
        Exact { text: Arc::from(&*self.text), num: self.num, den: self.den, nearest: OnceLock::new() }
    }

    /// The nearest binary64, computed once and kept (shared policy).
    pub fn to_f64(&self) -> f64 {
        *self.nearest.get_or_init(|| self.to_f64_uncached())
    }

    /// The nearest binary64 (ties to even), computed on every call (owned policy).
    pub fn to_f64_uncached(&self) -> f64 {
        if self.num == 0 {
            return 0.0;
        }
        let d = self.den as u128;
        let mut n = self.num.unsigned_abs();
        // Binary long division: 53 significant bits, a rounding bit, then a sticky bit.
        let (int, mut rem) = (n / d, n % d);
        let (mut exp, mut sig, mut taken, mut sticky): (i32, u64, u32, bool);
        if int > 0 {
            let width = 128 - int.leading_zeros();
            let keep = width.min(54);
            (exp, sig, taken) = (width as i32 - 1, (int >> (width - keep)) as u64, keep);
            sticky = int & ((1u128 << (width - keep)) - 1) != 0;
        } else {
            // Skip leading zero bits of the fraction; the first 1 bit is the leading significant bit.
            exp = 0;
            loop {
                rem <<= 1; // rem < d <= i128::MAX, so this cannot overflow
                exp -= 1;
                if rem >= d { rem -= d; break; }
            }
            (sig, taken, sticky) = (1, 1, false);
        }
        while taken < 54 {
            rem <<= 1;
            let bit = if rem >= d { rem -= d; 1 } else { 0 };
            sig = (sig << 1) | bit;
            taken += 1;
        }
        sticky |= rem != 0;
        n = (sig >> 1) as u128;
        if sig & 1 == 1 && (sticky || n & 1 == 1) {
            n += 1;
            if n == 1 << 53 { n >>= 1; exp += 1; }
        }
        // |value| is in [2^-127, 2^127): a normal binary64.
        let v = f64::from_bits(((exp + 1023) as u64) << 52 | (n as u64 & ((1 << 52) - 1)));
        if self.num < 0 { -v } else { v }
    }

    /// Exact loss of the boundary conversion: self - nearest binary64.
    pub fn f64_loss(&self) -> Option<Self> {
        self.checked_sub(&Self::from_f64(self.to_f64())?)
    }

    pub fn checked_add(&self, o: &Self) -> Option<Self> {
        let g = gcd(self.den as u128, o.den as u128) as i128;
        let num = self.num.checked_mul(o.den / g)?.checked_add(o.num.checked_mul(self.den / g)?)?;
        Self::from_ratio(num, (self.den / g).checked_mul(o.den)?)
    }

    pub fn checked_sub(&self, o: &Self) -> Option<Self> {
        self.checked_add(&Self::from_ratio(o.num.checked_neg()?, o.den)?)
    }

    pub fn checked_mul(&self, o: &Self) -> Option<Self> {
        let g1 = gcd(self.num.unsigned_abs(), o.den as u128).max(1) as i128;
        let g2 = gcd(o.num.unsigned_abs(), self.den as u128).max(1) as i128;
        Self::from_ratio((self.num / g1).checked_mul(o.num / g2)?, (self.den / g2).checked_mul(o.den / g1)?)
    }

    pub fn abs(&self) -> Self {
        Self::from_ratio(self.num.abs(), self.den).expect("|num| fits: i128::MIN is excluded")
    }

    /// Exact ordering. None if a cross product does not fit i128.
    pub fn checked_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.num.checked_mul(o.den)?.cmp(&o.num.checked_mul(self.den)?))
    }

    /// True when the value is an integer multiple of `step`.
    pub fn on_grid(&self, step: &Self) -> Option<bool> {
        let q = self.checked_mul(&Self::from_ratio(step.den, step.num)?)?;
        Some(q.den == 1)
    }
}

impl PartialEq for Exact {
    /// Equality of value.
    fn eq(&self, o: &Self) -> bool { self.num == o.num && self.den == o.den }
}

impl fmt::Display for Exact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.text) }
}

impl Serialize for Exact {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> { s.serialize_str(&self.text) }
}

impl<'de> Deserialize<'de> for Exact {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        Exact::parse(&text).map_err(serde::de::Error::custom)
    }
}
