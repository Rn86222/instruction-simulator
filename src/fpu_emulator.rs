use std::cmp::*;
use std::fmt::Debug;
use std::ops::*;

use crate::types::Int;

#[derive(Copy, Clone)]
pub struct FloatingPoint {
    value: u32,
}

impl FloatingPoint {
    pub fn new(value: u32) -> Self {
        FloatingPoint { value }
    }

    pub fn new_f32(value_f32: f32) -> Self {
        FloatingPoint {
            value: value_f32.to_bits(),
        }
    }

    pub fn get_sign(&self) -> i32 {
        if self.value & 0x80000000 != 0 {
            -1
        } else {
            1
        }
    }

    pub fn get_exp(&self) -> i32 {
        let mut exp = self.value & 0x7f800000;
        exp >>= 23;
        exp as i32 - 127
    }

    pub fn get_fraction(&self) -> u32 {
        self.value & 0x7fffff
    }

    pub fn get_all(&self) -> (i32, i32, u32) {
        (self.get_sign(), self.get_exp(), self.get_fraction())
    }

    pub fn get_1_8_23_bits(&self) -> (u32, u32, u32) {
        (
            to_n_bits_u32((self.value & 0x80000000) >> 31, 1),
            to_n_bits_u32((self.value & 0x7f800000) >> 23, 8),
            to_n_bits_u32(self.value & 0x7fffff, 23),
        )
    }

    pub fn get_f32_value(&self) -> f32 {
        f32::from_bits(self.value)
    }

    pub fn get_32_bits(&self) -> u32 {
        self.value
    }
}

fn to_n_bits_u32(num: u32, n: u32) -> u32 {
    let mut n = 1 << n;
    n -= 1;
    num & n
}

fn to_n_bits_u64(num: u64, n: u32) -> u64 {
    let mut n = 1 << n;
    n -= 1;
    num & n
}

// impl Display for FloatingPoint {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut result = self.value;
//         let mut exp = 0;
//         while result & 0x80000000 == 0 {
//             result <<= 1;
//             exp -= 1;
//         }
//         result <<= 1;
//         exp -= 1;
//         result >>= 9;
//         result &= 0x7fffff;
//         result |= (exp as u32 + 127) << 23;
//         write!(f, "{:x}", result)
//     }
// }

impl Debug for FloatingPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (sign, exp, fraction) = self.get_all();
        write!(f, "{} x 1.{:>023b} x 2^{}", sign, fraction, exp)
    }
}

impl Add for FloatingPoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let (s1, e1, m1) = self.get_1_8_23_bits();
        let (s2, e2, m2) = other.get_1_8_23_bits();
        let (m1a, e1a) = if e1 == 0 {
            (to_n_bits_u32(m1, 25), 1)
        } else {
            (to_n_bits_u32(m1 | 0x800000, 25), e1)
        };
        let (m2a, e2a) = if e2 == 0 {
            (to_n_bits_u32(m2, 25), 1)
        } else {
            (to_n_bits_u32(m2 | 0x800000, 25), e2)
        };
        let (ce, tde) = if e1a > e2a {
            (0_u32, to_n_bits_u32(e1a - e2a, 8))
        } else {
            (1_u32, to_n_bits_u32(e2a - e1a, 8))
        };
        let de = if tde >> 5 != 0 {
            31
        } else {
            to_n_bits_u32(tde, 5)
        };
        let sel = if de == 0 {
            if m1a > m2a {
                0
            } else {
                1
            }
        } else {
            ce
        };
        let (ms, mi, es, ss) = if sel == 0 {
            (m1a, m2a, e1a, s1)
        } else {
            (m2a, m1a, e2a, s2)
        };
        let mie = to_n_bits_u64((mi as u64) << 31, 56);
        let mia = to_n_bits_u64(mie >> (de as u64), 56);
        let tstck: u32 = if to_n_bits_u64(mia, 29) != 0 { 1 } else { 0 };
        let mye = if s1 == s2 {
            to_n_bits_u64(((ms as u64) << 2) + (mia >> 29), 27)
        } else {
            to_n_bits_u64(((ms as u64) << 2) - (mia >> 29), 27)
        };
        let esi = to_n_bits_u32(es + 1, 8);
        let (eyd, myd, stck) = if mye & (1 << 26) != 0 {
            if esi == 255 {
                (255, 1 << 25, 0)
            } else {
                (esi, to_n_bits_u64(mye >> 1, 27), tstck | (mye & 1) as u32)
            }
        } else {
            (es, mye, tstck)
        };
        let se = to_n_bits_u64(myd, 26).leading_zeros() - 38;
        let eyf = eyd as i64 - se as i64;
        let (myf, eyr) = if eyf > 0 {
            (to_n_bits_u64(myd << se, 56), (eyf & 0xFF) as u32)
        } else {
            (to_n_bits_u64(myd << ((eyd & 31) - 1), 56), 0)
        };
        let myr = if myf & 0b10 != 0 && myf & 0b1 != 0
            || myf & 0b10 != 0 && stck == 0 && myf & 0b100 != 0
            || myf & 0b10 != 0 && s1 == s2 && stck == 1
        {
            to_n_bits_u64(to_n_bits_u64(myf >> 2, 25) + 1, 25)
        } else {
            to_n_bits_u64(myf >> 2, 25)
        };
        let eyri = to_n_bits_u32(eyr + 1, 8);
        let (ey, my) = if (myr >> 24) & 1 != 0 {
            (eyri, 0)
        } else if to_n_bits_u64(myr, 24) == 0 {
            (0, 0)
        } else {
            (eyr, to_n_bits_u64(myr, 23))
        };
        let sy = if ey == 0 && my == 0 { s1 & s2 } else { ss };
        let nzm1 = if to_n_bits_u32(m1, 23) != 0 { 1 } else { 0 };
        let nzm2 = if to_n_bits_u32(m2, 23) != 0 { 1 } else { 0 };
        let y = if e1 == 255 && e2 != 255 {
            (s1 << 31) + (255 << 23) + (nzm1 << 22) + to_n_bits_u32(m1, 22)
        } else if e1 != 255 && e2 == 255 {
            (s2 << 31) + (255 << 23) + (nzm2 << 22) + to_n_bits_u32(m2, 22)
        } else if e1 == 255 && e2 == 255 && nzm1 == 1 {
            (s1 << 31) + (255 << 23) + (1 << 22) + to_n_bits_u32(m1, 22)
        } else if e1 == 255 && e2 == 255 && nzm2 == 1 {
            (s2 << 31) + (255 << 23) + (1 << 22) + to_n_bits_u32(m2, 22)
        } else if e1 == 255 && e2 == 255 && s1 == s2 {
            (s1 << 31) + (255 << 23)
        } else if e1 == 255 && e2 == 255 {
            (1 << 31) + (255 << 23) + (1 << 22)
        } else {
            (sy << 31) | (ey << 23) | (my as u32)
        };

        let _ovf = if e1 == 255 && e2 == 255 {
            0
        } else if ((mye >> 26) & 1 == 1 && esi == 255) || (myr >> 24) & 1 == 1 && eyri == 255 {
            1
        } else {
            0
        };
        FloatingPoint { value: y }
    }
}

impl Sub for FloatingPoint {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let neg_other = FloatingPoint {
            value: other.value ^ 0x80000000,
        };
        self + neg_other
    }
}

impl Mul for FloatingPoint {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        let (s1, e1, m1) = self.get_1_8_23_bits();
        let (s2, e2, m2) = other.get_1_8_23_bits();
        let (h1, h2) = (m1 >> 11, m2 >> 11);
        let (l1, l2) = (m1 & 0x7ff, m2 & 0x7ff);
        let h1i = h1 | 0x1000;
        let h2i = h2 | 0x1000;
        let h1h2 = (h1i * h2i) as u64;
        let h1l2 = (h1i * l2) as u64;
        let l1h2 = (l1 * h2i) as u64;
        let sy = s1 ^ s2;
        let eys = e1 + e2 + 129;
        let m1m2 = h1h2 + (h1l2 >> 11) + (l1h2 >> 11) + 2;
        let eysi = eys + 1;
        let ey = if e1 == 0 || e2 == 0 || (eys >> 8) & 1 == 0 {
            0
        } else if m1m2 & (1 << 25) != 0 {
            to_n_bits_u32(eysi, 8)
        } else {
            to_n_bits_u32(eys, 8)
        };
        let my = if ey == 0 {
            0
        } else if m1m2 & (1 << 25) != 0 {
            to_n_bits_u64(m1m2 >> 2, 23)
        } else {
            to_n_bits_u64(m1m2 >> 1, 23)
        };
        let y = (sy << 31) | (ey << 23) | (my as u32);
        FloatingPoint { value: y }
    }
}

pub type InvMap = Vec<(FloatingPoint, FloatingPoint)>;

pub fn create_inv_map() -> InvMap {
    let eps = 2_f64.powf(-10.);
    let mut inv_map = Vec::new();
    for i in 0..1024 {
        let left = 1. + (i as f64) * eps;
        let right = 1. + ((i + 1) as f64) * eps;
        let middle_x = (left + right) / 2.;
        let left_inv = 1. / left;
        let right_inv = 1. / right;
        let a = (right_inv - left_inv) / eps;
        let middle_y_up = (left_inv + right_inv) / 2.;
        let middle_y_down = 1. / middle_x;
        let middle_y = (middle_y_up + middle_y_down) / 2.;
        let b = middle_y - a * middle_x;
        let a_fp = FloatingPoint::new_f32(a.abs() as f32);
        let b_fp = FloatingPoint::new_f32(b as f32);
        inv_map.push((a_fp, b_fp));
    }
    inv_map
}

fn inv(x: FloatingPoint, inv_map: &InvMap) -> FloatingPoint {
    let value = x.get_f32_value();
    assert!((1. ..2.).contains(&value));
    let (_, _, m) = x.get_1_8_23_bits();
    let index = (m >> 13) as usize;
    let (a, b) = inv_map[index];
    b - a * x
}

pub fn div_fp(this: FloatingPoint, other: FloatingPoint, inv_map: &InvMap) -> FloatingPoint {
    let (s1, e1, m1) = this.get_1_8_23_bits();
    let (s2, e2, m2) = other.get_1_8_23_bits();
    if e1 == 0 {
        return FloatingPoint { value: 0 };
    }
    let normailized_this = FloatingPoint::new((127 << 23) + m1);
    let normilized_other = FloatingPoint::new((127 << 23) + m2);
    let normalized_other_inv = inv(normilized_other, inv_map);
    let yi = normailized_this * normalized_other_inv;
    let (_, ei, my) = yi.get_1_8_23_bits();
    let eyi = (e1 as i32 - 127) - (e2 as i32 - 127) + (ei as i32 - 127) + 127;
    let ey = if eyi < 0 {
        0
    } else {
        to_n_bits_u32(eyi as u32, 8)
    };
    let sy = s1 ^ s2;
    let y = (sy << 31) | (ey << 23) | my;
    FloatingPoint { value: y }
}

pub type SqrtMap = Vec<(FloatingPoint, FloatingPoint)>;

pub fn create_sqrt_map() -> SqrtMap {
    let mut sqrt_map = Vec::new();
    let mut eps = 2_f64.powf(-9.);
    let mut start = 1.;
    for _ in 0..2 {
        for i in 0..512 {
            let left = start + (i as f64) * eps;
            let right = start + ((i + 1) as f64) * eps;
            let middle_x = (left + right) / 2.;
            let left_sqrt = left.sqrt();
            let right_sqrt = right.sqrt();
            let a = (right_sqrt - left_sqrt) / eps;
            let middle_y_up = middle_x.sqrt();
            let middle_y_down = (left_sqrt + right_sqrt) / 2.;
            let middle_y = (middle_y_up + middle_y_down) / 2.;
            let b = middle_y - a * middle_x;
            let a_fp = FloatingPoint::new_f32(a as f32);
            let b_fp = FloatingPoint::new_f32(b as f32);
            sqrt_map.push((a_fp, b_fp));
        }
        eps *= 2.;
        start += 1.;
    }
    sqrt_map
}

pub fn sqrt_fp(this: FloatingPoint, sqrt_map: &SqrtMap) -> FloatingPoint {
    let (s, e, m) = this.get_1_8_23_bits();
    if s == 1 {
        panic!("sqrt of negative number");
    }
    if e == 0 {
        return FloatingPoint { value: 0 };
    }
    let (sh, offset_e) = if e < 127 {
        if (127 - e) % 2 == 0 {
            (0, 127 - e)
        } else {
            (0, 128 - e)
        }
    } else if e > 128 {
        if (e - 128) % 2 == 0 {
            (1, e - 128)
        } else {
            (1, e - 127)
        }
    } else {
        (0, 0)
    };
    let ei = if sh == 0 { e + offset_e } else { e - offset_e };
    let normalized_x = FloatingPoint::new((ei << 23) + m);
    let index = (((!ei & 1) << 9) + (m >> 14)) as usize;
    let (a, b) = sqrt_map[index];
    let yi = b + a * normalized_x;
    let (_, eyi, my) = yi.get_1_8_23_bits();
    let ey = if sh == 0 {
        to_n_bits_u32(eyi - offset_e / 2, 8)
    } else {
        to_n_bits_u32(eyi + offset_e / 2, 8)
    };
    let y = (ey << 23) | my;
    FloatingPoint { value: y }
}

pub fn fp_to_int(this: FloatingPoint) -> Int {
    let (s, e, m) = this.get_1_8_23_bits();
    if e == 0 {
        return 0;
    }
    let mi = m | 0x800000;
    let mis = mi << 7;
    let (msb, myi) = if e < 126 {
        (0, 0)
    } else if e == 126 {
        (1, 0)
    } else if e < 127 + 30 {
        ((mis >> (30 - (e - 127 + 1))) & 1, mis >> (30 - (e - 127)))
    } else if e == 127 + 30 {
        (0, mis)
    } else if s == 1 {
        (0, 1 << 31)
    } else {
        (0, (1 << 31) - 1)
    };
    let my = myi + msb;
    if s == 0 || e >= 127 + 31 {
        my as Int
    } else if my == 0 {
        0
    } else {
        !(my as Int) + 1
    }
}

pub fn int_to_fp(x: Int) -> FloatingPoint {
    if x == std::i32::MIN {
        return FloatingPoint { value: 0xcf000000 };
    }
    if x == 0 {
        return FloatingPoint { value: 0 };
    }
    let ux = if x < 0 { !(x - 1) as u32 } else { x as u32 };
    let se = ux.leading_zeros();
    let mye = if se == 31 {
        0
    } else {
        (ux & !(1 << (31 - se))) << (se + 1)
    };
    let myi = mye >> 9;
    let myi2 = if mye & (1 << 8) != 0 { myi + 1 } else { myi };
    let my = to_n_bits_u32(myi2, 23);
    let ey = if myi.count_ones() == 23 && mye & (1 << 8) != 0 {
        to_n_bits_u32(127 + 31 - se + 1, 8)
    } else {
        to_n_bits_u32(127 + 31 - se, 8)
    };
    let sy = if x < 0 { 1 } else { 0 };
    let y = (sy << 31) | (ey << 23) | my;
    FloatingPoint { value: y }
}

#[allow(dead_code)]
fn sin(x: FloatingPoint) -> FloatingPoint {
    let x2 = x * x;
    let x4 = x2 * x2;
    x - (x2 * x) * FloatingPoint::new_f32(0.16666668)
        + (x4 * x) * FloatingPoint::new_f32(0.008332824)
        - (x4 * x2 * x) * FloatingPoint::new_f32(0.00019587841)
}

#[allow(dead_code)]
fn cos(x: FloatingPoint) -> FloatingPoint {
    let x2 = x * x;
    let x4 = x2 * x2;
    FloatingPoint::new_f32(1.) - (x2 * FloatingPoint::new_f32(0.5))
        + (x4 * FloatingPoint::new_f32(0.04166368))
        - (x4 * x2 * FloatingPoint::new_f32(0.0013695068))
}

#[allow(dead_code)]
fn atan_sub(x: FloatingPoint) -> FloatingPoint {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x8 = x4 * x4;
    x - (x2 * x * FloatingPoint::new_f32(0.3333333)) + (x4 * x * FloatingPoint::new_f32(0.2))
        - (x4 * x2 * x * FloatingPoint::new_f32(0.14285715))
        + (x8 * x * FloatingPoint::new_f32(0.111111104))
        - (x8 * x2 * x * FloatingPoint::new_f32(0.08976446))
        + (x8 * x4 * x * FloatingPoint::new_f32(0.060035485))
}

#[allow(dead_code)]
fn atan(x: FloatingPoint, inv_map: &InvMap) -> FloatingPoint {
    let pi = FloatingPoint::new_f32(f32::from_bits(0x40490fdb));
    if x < FloatingPoint::new_f32(0.) {
        -atan_sub(-x)
    } else if x < FloatingPoint::new_f32(0.4375) {
        atan_sub(x)
    } else if x < FloatingPoint::new_f32(2.4375) {
        div_fp(pi, FloatingPoint::new_f32(4.), inv_map)
            + atan_sub(div_fp(
                x - FloatingPoint::new_f32(1.),
                x + FloatingPoint::new_f32(1.),
                inv_map,
            ))
    } else {
        div_fp(pi, FloatingPoint::new_f32(2.), inv_map)
            - atan_sub(div_fp(FloatingPoint::new_f32(1.), x, inv_map))
    }
}

impl Neg for FloatingPoint {
    type Output = Self;
    fn neg(self) -> Self {
        let mut result = self.value;
        result ^= 0x80000000;
        FloatingPoint { value: result }
    }
}

impl PartialEq for FloatingPoint {
    fn eq(&self, other: &Self) -> bool {
        let (s1, e1, m1) = self.get_1_8_23_bits();
        let (s2, e2, m2) = other.get_1_8_23_bits();
        (e1 == 0 && e2 == 0) || (s1 == s2 && e1 == e2 && m1 == m2)
    }
}

impl PartialOrd for FloatingPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for FloatingPoint {}

impl Ord for FloatingPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        let (s1, e1, m1) = self.get_1_8_23_bits();
        let (s2, e2, m2) = other.get_1_8_23_bits();
        if e1 == 0 && e2 == 0 {
            return Ordering::Equal;
        }
        if s1 != s2 {
            if s1 == 1 {
                return Ordering::Less;
            } else {
                return Ordering::Greater;
            }
        }
        if s1 == 0 {
            if e1 > e2 {
                Ordering::Greater
            } else if e1 < e2 {
                Ordering::Less
            } else if m1 > m2 {
                Ordering::Greater
            } else if m1 < m2 {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        } else if e1 > e2 {
            Ordering::Less
        } else if e1 < e2 {
            Ordering::Greater
        } else if m1 > m2 {
            Ordering::Less
        } else if m1 < m2 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

pub fn fp_sign_injection(this: FloatingPoint, other: FloatingPoint) -> FloatingPoint {
    let (_, e1, m1) = this.get_1_8_23_bits();
    let (s2, _, _) = other.get_1_8_23_bits();
    let sy = s2;
    let ey = e1;
    let my = m1;
    let y = (sy << 31) | (ey << 23) | my;
    FloatingPoint { value: y }
}

pub fn fp_negative_sign_injection(this: FloatingPoint, other: FloatingPoint) -> FloatingPoint {
    let (_, e1, m1) = this.get_1_8_23_bits();
    let (s2, _, _) = other.get_1_8_23_bits();
    let sy = s2 ^ 1;
    let ey = e1;
    let my = m1;
    let y = (sy << 31) | (ey << 23) | my;
    FloatingPoint { value: y }
}

pub fn fp_xor_sign_injection(this: FloatingPoint, other: FloatingPoint) -> FloatingPoint {
    let (s1, e1, m1) = this.get_1_8_23_bits();
    let (s2, _, _) = other.get_1_8_23_bits();
    let sy = s1 ^ s2;
    let ey = e1;
    let my = m1;
    let y = (sy << 31) | (ey << 23) | my;
    FloatingPoint { value: y }
}

#[cfg(test)]
mod tests {
    use std::io::{stdout, Write};

    use super::*;
    use rand::prelude::*;

    fn gen_one_random_operand(rng: &mut ThreadRng, left: f32, right: f32) -> f32 {
        rng.gen_range(left..=right)
    }

    fn gen_two_random_operands(rng: &mut ThreadRng, left: f32, right: f32) -> (f32, f32) {
        let op1 = gen_one_random_operand(rng, left, right);
        let op2 = gen_one_random_operand(rng, left, right);
        (op1, op2)
    }

    fn gen_two_floating_points_from_f32(op1: f32, op2: f32) -> (FloatingPoint, FloatingPoint) {
        let fp1 = FloatingPoint::new_f32(op1);
        let fp2 = FloatingPoint::new_f32(op2);
        (fp1, fp2)
    }

    const ITER_NUM: usize = 1000000000;

    #[test]
    fn test_add() {
        let relative_eps = 2_f64.powf(-23.);
        let absolute_eps = 2_f64.powf(-126.);
        let mut rng = rand::thread_rng();
        let left = -1e37;
        let right = 1e37;
        for _ in 0..ITER_NUM {
            let (op1, op2) = gen_two_random_operands(&mut rng, left, right);
            let correct_result = op1 as f64 + op2 as f64;
            let (fp1, fp2) = gen_two_floating_points_from_f32(op1, op2);
            let result = fp1 + fp2;
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            assert!(
                abs_diff < relative_eps * correct_result.abs()
                    || abs_diff < relative_eps * (op1 as f64).abs()
                    || abs_diff < relative_eps * (op2 as f64).abs()
                    || abs_diff < absolute_eps,
                "op1: {}, op2: {}, correct_result: {}, result: {}",
                op1,
                op2,
                correct_result,
                result.get_f32_value()
            );
        }
    }

    #[test]
    fn test_sub() {
        let relative_eps = 2_f64.powf(-23.);
        let absolute_eps = 2_f64.powf(-126.);
        let mut rng = rand::thread_rng();
        let left = -1e37;
        let right = 1e37;
        for _ in 0..ITER_NUM {
            let (op1, op2) = gen_two_random_operands(&mut rng, left, right);
            let correct_result = op1 as f64 - op2 as f64;
            let (fp1, fp2) = gen_two_floating_points_from_f32(op1, op2);
            let result = fp1 - fp2;
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            assert!(
                abs_diff < relative_eps * correct_result.abs()
                    || abs_diff < relative_eps * (op1 as f64).abs()
                    || abs_diff < relative_eps * (op2 as f64).abs()
                    || abs_diff < absolute_eps,
                "op1: {}, op2: {}, correct_result: {}, result: {}",
                op1,
                op2,
                correct_result,
                result.get_f32_value()
            );
        }
    }

    #[test]
    fn test_mul() {
        let mut rng = rand::thread_rng();
        let relative_eps = 2_f64.powf(-22.);
        let absolute_eps = 2_f64.powf(-126.);
        let left = -1e18;
        let right = 1e18;
        for _ in 0..ITER_NUM {
            let (op1, op2) = gen_two_random_operands(&mut rng, left, right);
            let correct_result = op1 as f64 * op2 as f64;
            let (fp1, fp2) = gen_two_floating_points_from_f32(op1, op2);
            let result = fp1 * fp2;
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            assert!(
                abs_diff < relative_eps * correct_result.abs() || abs_diff < absolute_eps,
                "op1: {}, op2: {}, correct_result: {}, result: {}",
                op1,
                op2,
                correct_result,
                result.get_f32_value()
            );
        }
    }

    #[test]
    fn test_div() {
        let inv_map = create_inv_map();
        let mut rng = rand::thread_rng();
        let relative_eps = 2_f64.powf(-20.);
        let absolute_eps = 2_f64.powf(-126.);
        let left = -1e37;
        let right = 1e37;
        for _ in 0..ITER_NUM {
            let (op1, op2) = gen_two_random_operands(&mut rng, left, right);
            if op2 == 0. {
                continue;
            }
            let correct_result = op1 as f64 / op2 as f64;
            let (fp1, fp2) = gen_two_floating_points_from_f32(op1, op2);
            let result = div_fp(fp1, fp2, &inv_map);
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            assert!(
                abs_diff < relative_eps * correct_result.abs() || abs_diff < absolute_eps,
                "op1: {}, op2: {}, correct_result: {}, result: {}",
                op1,
                op2,
                correct_result,
                result.get_f32_value()
            );
        }
    }

    #[test]
    fn test_sqrt() {
        let sqrt_map = create_sqrt_map();
        let relative_eps = 2_f64.powf(-20.);
        let absolute_eps = 2_f64.powf(-126.);
        let s = 0;
        let min_e = 1;
        let max_e = 254;
        let min_m = 0;
        let max_m = 0x7fffff;
        let e = 0;
        for m in min_m..=max_m {
            let op = (s << 31) + (e << 23) + m;
            let fp = FloatingPoint::new(op);
            let correct_result: f64 = 0.;
            let result = sqrt_fp(fp, &sqrt_map);
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            assert!(
                abs_diff < relative_eps * correct_result.abs() || abs_diff < absolute_eps,
                "op: {}, correct_result: {}, result: {}",
                f32::from_bits(op),
                correct_result,
                result.get_f32_value(),
            );
        }
        for e in min_e..=max_e {
            print!(
                "\r{:.0}%",
                (e - min_e) as f32 / (max_e - min_e + 1) as f32 * 100.0
            );
            stdout().flush().unwrap();
            for m in min_m..=max_m {
                let op = (s << 31) + (e << 23) + m;
                let fp = FloatingPoint::new(op);
                let correct_result: f64 = (f32::from_bits(op) as f64).sqrt();
                let result = sqrt_fp(fp, &sqrt_map);
                let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
                assert!(
                    abs_diff < relative_eps * correct_result.abs() || abs_diff < absolute_eps,
                    "op: {}, correct_result: {}, result: {}",
                    f32::from_bits(op),
                    correct_result,
                    result.get_f32_value(),
                );
            }
        }
        println!();
    }

    #[test]
    fn test_fp_to_int() {
        let min_s = 0;
        let max_s = 1;
        let min_e = 1;
        let max_e = 254;
        let min_m = 0;
        let max_m = 0x7fffff;
        let e = 0;
        for s in min_s..=max_s {
            for m in min_m..=max_m {
                let op = (s << 31) + (e << 23) + m;
                let fp = FloatingPoint::new(op);
                let result = fp_to_int(fp);
                let abs_diff = (result as f32).abs();
                assert!(abs_diff <= 0.5, "op: {}, result: {}", op, result);
            }
        }
        for s in min_s..=max_s {
            println!("s: {}", s);
            for e in min_e..=max_e {
                print!(
                    "\r{:.0}%",
                    (e - min_e) as f32 / (max_e - min_e + 1) as f32 * 100.0
                );
                stdout().flush().unwrap();
                for m in min_m..=max_m {
                    let op = (s << 31) + (e << 23) + m;
                    let float = f32::from_bits(op) as f64;
                    if float < std::i32::MIN as f64 || float > std::i32::MAX as f64 {
                        continue;
                    }
                    let fp = FloatingPoint::new(op);
                    let result = fp_to_int(fp);
                    let abs_diff = (result as f64 - float).abs();
                    assert!(abs_diff <= 0.5, "op: {}, result: {}", op, result,);
                }
            }
            println!();
        }
    }

    use float_next_after::NextAfter;
    #[test]
    fn test_int_to_fp() {
        for x in std::i32::MIN..=std::i32::MAX {
            if x % 1000000 == 0 {
                print!(
                    "\r{:.0}%",
                    (x as f32 - std::i32::MIN as f32)
                        / (std::i32::MAX as f32 - std::i32::MIN as f32 + 1.)
                        * 100.0
                );
                stdout().flush().unwrap();
            }
            let correct_result = x as f64;
            let result = int_to_fp(x);
            let float = result.get_f32_value();
            let next_smaller = float.next_after(float - 1.0);
            let next_larger = float.next_after(float + 1.0);
            let abs_diff_smaller = (next_smaller as f64 - correct_result).abs();
            let abs_diff_larger = (next_larger as f64 - correct_result).abs();
            let abs_diff = (float as f64 - correct_result).abs();
            assert!(
                abs_diff <= abs_diff_smaller && abs_diff <= abs_diff_larger,
                "x: {}, float: {}, next_smaller: {}, next_larger: {}, correct_result: {}",
                x,
                float,
                next_smaller,
                next_larger,
                correct_result
            );
        }
        println!();
    }

    #[test]
    fn test_sin() {
        let inv_map = create_inv_map();
        let pi_over_4 = div_fp(
            FloatingPoint::new_f32(std::f32::consts::PI),
            FloatingPoint::new_f32(4.),
            &inv_map,
        );
        let mut x = 0.;
        let absolute_eps = 2_f64.powf(-126.);
        while FloatingPoint::new_f32(x) <= pi_over_4 {
            let correct_result = (x as f64).sin();
            let result = sin(FloatingPoint::new_f32(x));
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            if abs_diff >= absolute_eps {
                let ulp_of_result = if (correct_result as f32) > result.get_f32_value() {
                    result
                        .get_f32_value()
                        .next_after(result.get_f32_value() + 1.)
                        - result.get_f32_value()
                } else {
                    result
                        .get_f32_value()
                        .next_after(result.get_f32_value() - 1.)
                        - result.get_f32_value()
                };
                let mut ulp_count: i32 = 0;
                while correct_result as f32
                    != result.get_f32_value() + ulp_count as f32 * ulp_of_result
                {
                    ulp_count += 1;
                    if ulp_count > 5 {
                        panic!(
                            "x: {}, correct_result: {}, result: {}",
                            x,
                            correct_result,
                            result.get_f32_value()
                        );
                    }
                }
            }
            x = x.next_after(x + 0.01);
        }
    }

    #[test]
    fn test_cos() {
        let inv_map = create_inv_map();
        let pi_over_4 = div_fp(
            FloatingPoint::new_f32(std::f32::consts::PI),
            FloatingPoint::new_f32(4.),
            &inv_map,
        );
        let mut x = 0.;
        let absolute_eps = 2_f64.powf(-126.);
        while FloatingPoint::new_f32(x) <= pi_over_4 {
            let correct_result = (x as f64).cos();
            let result = cos(FloatingPoint::new_f32(x));
            let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
            if abs_diff >= absolute_eps {
                let ulp_of_result = if (correct_result as f32) > result.get_f32_value() {
                    result
                        .get_f32_value()
                        .next_after(result.get_f32_value() + 1.)
                        - result.get_f32_value()
                } else {
                    result
                        .get_f32_value()
                        .next_after(result.get_f32_value() - 1.)
                        - result.get_f32_value()
                };
                let mut ulp_count: i32 = 0;
                while correct_result as f32
                    != result.get_f32_value() + ulp_count as f32 * ulp_of_result
                {
                    ulp_count += 1;
                    if ulp_count > 5 {
                        panic!(
                            "x: {}, correct_result: {}, result: {}",
                            x,
                            correct_result,
                            result.get_f32_value()
                        );
                    }
                }
            }
            x = x.next_after(x + 0.01);
        }
    }

    #[test]
    fn test_atan() {
        let inv_map = create_inv_map();
        let relative_eps = 2_f64.powf(-20.);
        let absolute_eps = 2_f64.powf(-126.);
        let s = 0;
        let min_e = 0;
        let max_e = 254;
        let min_m = 0;
        let max_m = 0x7fffff;
        for e in min_e..=max_e {
            print!(
                "\r{:.0}%",
                (e - min_e) as f32 / (max_e - min_e + 1) as f32 * 100.0
            );
            stdout().flush().unwrap();
            for m in min_m..=max_m {
                let op = (s << 31) + (e << 23) + m;
                let fp = FloatingPoint::new(op);
                let correct_result: f64 = (f32::from_bits(op) as f64).atan();
                let result = atan(fp, &inv_map);
                let abs_diff = (correct_result - result.get_f32_value() as f64).abs();
                assert!(
                    abs_diff < relative_eps * correct_result.abs() || abs_diff < absolute_eps,
                    "op: {}, correct_result: {}, result: {}",
                    f32::from_bits(op),
                    correct_result,
                    result.get_f32_value(),
                );
            }
        }
        println!();
    }
}
