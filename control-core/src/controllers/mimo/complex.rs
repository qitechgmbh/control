//! Minimal complex arithmetic for frequency-domain synthesis.
//!
//! Only what the LMI backend needs: the plant is evaluated at sampled frequencies, so the matrices
//! are small, fixed-size and used a few thousand times per solve. `num-complex` is not a
//! dependency of this crate and this is a few dozen lines.

use super::matrix::Mat;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

impl C {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub const fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    pub const fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }

    pub const fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }

    pub const fn mul(self, o: Self) -> Self {
        Self::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }

    pub const fn scale(self, s: f64) -> Self {
        Self::new(self.re * s, self.im * s)
    }

    pub const fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    pub fn inv(self) -> Self {
        let d = self.re * self.re + self.im * self.im;
        if d == 0.0 {
            return Self::ZERO;
        }
        Self::new(self.re / d, -self.im / d)
    }

    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
}

pub type CMat<const N: usize> = [[C; N]; N];

pub const fn czeros<const N: usize>() -> CMat<N> {
    [[C::ZERO; N]; N]
}

pub fn cidentity<const N: usize>() -> CMat<N> {
    let mut m = czeros::<N>();
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = C::ONE;
    }
    m
}

/// Promote a real matrix.
pub fn from_real<const N: usize>(a: &Mat<N>) -> CMat<N> {
    let mut m = czeros::<N>();
    for i in 0..N {
        for j in 0..N {
            m[i][j] = C::real(a[i][j]);
        }
    }
    m
}

pub fn cmatmul<const N: usize>(a: &CMat<N>, b: &CMat<N>) -> CMat<N> {
    let mut out = czeros::<N>();
    for i in 0..N {
        for k in 0..N {
            let aik = a[i][k];
            if aik == C::ZERO {
                continue;
            }
            for j in 0..N {
                out[i][j] = out[i][j].add(aik.mul(b[k][j]));
            }
        }
    }
    out
}

pub fn cadd<const N: usize>(a: &CMat<N>, b: &CMat<N>) -> CMat<N> {
    let mut out = czeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[i][j] = a[i][j].add(b[i][j]);
        }
    }
    out
}

pub fn csub<const N: usize>(a: &CMat<N>, b: &CMat<N>) -> CMat<N> {
    let mut out = czeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[i][j] = a[i][j].sub(b[i][j]);
        }
    }
    out
}

pub fn cscale<const N: usize>(a: &CMat<N>, s: f64) -> CMat<N> {
    let mut out = *a;
    for row in &mut out {
        for v in row.iter_mut() {
            *v = v.scale(s);
        }
    }
    out
}

/// Conjugate transpose, written `A*` in the paper.
pub fn cadjoint<const N: usize>(a: &CMat<N>) -> CMat<N> {
    let mut out = czeros::<N>();
    for i in 0..N {
        for j in 0..N {
            out[j][i] = a[i][j].conj();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplication_matches_the_scalar_definition() {
        let a = C::new(3.0, -2.0);
        let b = C::new(-1.0, 4.0);
        let p = a.mul(b);
        assert!((p.re - (-3.0 + 8.0)).abs() < 1e-12, "re {}", p.re);
        assert!((p.im - (12.0 + 2.0)).abs() < 1e-12, "im {}", p.im);
    }

    #[test]
    fn inverse_round_trips() {
        let a = C::new(0.3, -1.7);
        let p = a.mul(a.inv());
        assert!((p.re - 1.0).abs() < 1e-12 && p.im.abs() < 1e-12, "{p:?}");
    }

    #[test]
    fn adjoint_of_a_product_reverses_it() {
        // (AB)* == B* A*, the identity the LMI construction leans on.
        let a: CMat<2> = [
            [C::new(1.0, 2.0), C::new(0.5, -1.0)],
            [C::new(-3.0, 0.25), C::new(2.0, 2.0)],
        ];
        let b: CMat<2> = [
            [C::new(0.0, 1.0), C::new(4.0, -2.0)],
            [C::new(1.5, 0.5), C::new(-1.0, 3.0)],
        ];
        let lhs = cadjoint(&cmatmul(&a, &b));
        let rhs = cmatmul(&cadjoint(&b), &cadjoint(&a));
        for i in 0..2 {
            for j in 0..2 {
                assert!((lhs[i][j].re - rhs[i][j].re).abs() < 1e-12);
                assert!((lhs[i][j].im - rhs[i][j].im).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn identity_is_multiplicative_unit() {
        let a: CMat<3> = [
            [C::new(1.0, 1.0), C::ZERO, C::new(2.0, -1.0)],
            [C::ZERO, C::new(-2.0, 0.5), C::ONE],
            [C::new(3.0, 0.0), C::ONE, C::ZERO],
        ];
        let i = cidentity::<3>();
        assert_eq!(cmatmul(&a, &i), a);
        assert_eq!(cmatmul(&i, &a), a);
    }
}
