//! Commutative algebra and polynomials.

use num_traits::{One, Pow, Zero};
use std::fmt::Display;
use std::iter::{Product, Sum};
use std::ops::{Add, AddAssign, Mul, Neg};

use derivative::Derivative;

use super::rig::*;

/// A commutative algebra over a commutative ring.
pub trait CommAlg: CommRing + Module<Ring = Self::R> {
    /// The base ring of the algebra.
    type R: CommRing;

    /** Convert an element of the base ring into an element of the algebra.

    A commutative algebra A over a commutative ring R can be defined as a ring
    homomorphism from R to A. This method computes that homomorphism.
     */
    fn from_scalar(r: Self::R) -> Self {
        Self::one() * r
    }
}

/** A polynomial in several variables.

In abstract terms, polynomials in a [commutative ring](super::rig::CommRing) R
are the free [commutative algebra](CommAlg) over R.
 */
#[derive(Clone, PartialEq, Eq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Polynomial<Var, Coef, Exp>(Combination<Monomial<Var, Exp>, Coef>);

impl<Var, Coef, Exp> Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Exp: Ord,
{
    /// Constructs the generating polynomial corresponding to a variable.
    pub fn generator(var: Var) -> Self
    where
        Coef: One,
        Exp: One,
    {
        Polynomial::from_monomial(Monomial::generator(var))
    }

    /// Constructs the polynomial corresponding to a monomial.
    pub fn from_monomial(m: Monomial<Var, Exp>) -> Self
    where
        Coef: One,
    {
        Polynomial(Combination::generator(m))
    }

    /// Iterates over the monomials in the polynomial.
    pub fn monomials(&self) -> impl ExactSizeIterator<Item = &Monomial<Var, Exp>> {
        self.0.variables()
    }

    /// Evaluates the polynomial by substituting for the variables.
    pub fn eval<A>(&self, values: &[A]) -> A
    where
        A: Clone + Mul<Coef, Output = A> + Pow<Exp, Output = A> + Sum + Product,
        Coef: Clone,
        Exp: Clone,
    {
        self.0.eval(self.monomials().map(|m| m.eval(values.iter().cloned())))
    }
}

impl<Var, Coef, Exp> FromIterator<(Coef, Monomial<Var, Exp>)> for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Add<Output = Coef>,
    Exp: Ord,
{
    fn from_iter<T: IntoIterator<Item = (Coef, Monomial<Var, Exp>)>>(iter: T) -> Self {
        Polynomial(iter.into_iter().collect())
    }
}

impl<Var, Coef, Exp> Display for Polynomial<Var, Coef, Exp>
where
    Var: Display,
    Coef: Display + PartialEq + One,
    Exp: Display + PartialEq + One,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// XXX: Lots of boilerplate to delegate the module structure of `Polynomial` to
// `Combination` because these traits cannot be derived automatically.

impl<Var, Coef, Exp> AddAssign<(Coef, Monomial<Var, Exp>)> for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Add<Output = Coef>,
    Exp: Ord,
{
    fn add_assign(&mut self, rhs: (Coef, Monomial<Var, Exp>)) {
        self.0 += rhs;
    }
}

impl<Var, Coef, Exp> Add for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Add<Output = Coef>,
    Exp: Ord,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Polynomial(self.0 + rhs.0)
    }
}

impl<Var, Coef, Exp> Zero for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Add<Output = Coef>,
    Exp: Ord,
{
    fn zero() -> Self {
        Polynomial(Combination::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<Var, Coef, Exp> AdditiveMonoid for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: AdditiveMonoid,
    Exp: Ord,
{
}

impl<Var, Coef, Exp> Mul<Coef> for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Clone + Default + Mul<Output = Coef>,
    Exp: Ord,
{
    type Output = Self;

    fn mul(self, a: Coef) -> Self::Output {
        Polynomial(self.0 * a)
    }
}

impl<Var, Coef, Exp> RigModule for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Clone + Default + CommRig,
    Exp: Ord,
{
    type Rig = Coef;
}

impl<Var, Coef, Exp> Neg for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Default + Neg<Output = Coef>,
    Exp: Ord,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Polynomial(self.0.neg())
    }
}

impl<Var, Coef, Exp> AbGroup for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Default + AbGroup,
    Exp: Ord,
{
}

impl<Var, Coef, Exp> Module for Polynomial<Var, Coef, Exp>
where
    Var: Ord,
    Coef: Clone + Default + CommRing,
    Exp: Ord,
{
    type Ring = Coef;
}

/// Multiply polynomials using the distributive law.
impl<Var, Coef, Exp> Mul for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Add<Output = Coef> + Mul<Output = Coef>,
    Exp: Clone + Ord + Add<Output = Exp>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Avoid unnecessary clones by tracking whether we're in the last
        // iteration of the outer and inner loops.
        let mut result = Polynomial::zero();
        let (outer, inner) = (self.0, rhs.0);
        let mut outer_iter = outer.into_iter();
        while let Some((a, m)) = outer_iter.next() {
            if outer_iter.len() == 0 {
                let mut inner_iter = inner.into_iter();
                while let Some((b, n)) = inner_iter.next() {
                    if inner_iter.len() == 0 {
                        result += (a * b, m * n);
                        break;
                    } else {
                        result += (a.clone() * b, m.clone() * n);
                    }
                }
                break;
            } else {
                for (b, n) in &inner {
                    result += (a.clone() * b.clone(), m.clone() * n.clone());
                }
            }
        }
        result
    }
}

impl<Var, Coef, Exp> One for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Add<Output = Coef> + One,
    Exp: Clone + Ord + Add<Output = Exp>,
{
    fn one() -> Self {
        Polynomial::from_monomial(Default::default())
    }
}

impl<Var, Coef, Exp> Monoid for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Rig,
    Exp: Clone + Ord + AdditiveMonoid,
{
}

impl<Var, Coef, Exp> Rig for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Rig,
    Exp: Clone + Ord + AdditiveMonoid,
{
}

impl<Var, Coef, Exp> Ring for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Default + Ring,
    Exp: Clone + Ord + AdditiveMonoid,
{
}

impl<Var, Coef, Exp> CommMonoid for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + CommRig,
    Exp: Clone + Ord + AdditiveMonoid,
{
}

impl<Var, Coef, Exp> CommRig for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + CommRig,
    Exp: Clone + Ord + AdditiveMonoid,
{
}

impl<Var, Coef, Exp> CommRing for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Default + CommRing,
    Exp: Clone + Ord + AdditiveMonoid,
{
}

impl<Var, Coef, Exp> CommAlg for Polynomial<Var, Coef, Exp>
where
    Var: Clone + Ord,
    Coef: Clone + Default + CommRing,
    Exp: Clone + Ord + AdditiveMonoid,
{
    type R = Coef;

    fn from_scalar(r: Self::R) -> Self {
        [(r, Monomial::one())].into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polynomials() {
        let x = || Polynomial::<_, i32, u32>::generator('x');
        let y = || Polynomial::<_, i32, u32>::generator('y');
        assert_eq!(x().to_string(), "x");

        let p = Polynomial::<char, i32, u32>::from_scalar(-5);
        assert_eq!(p.eval::<i32>(&[]), -5);

        let p = x() * y() * x() * 2 + y() * x() * y() * 3;
        assert_eq!(p.to_string(), "3 x y^2 + 2 x^2 y");
        assert_eq!(p.eval(&[1, 1]), 5);
        assert_eq!(p.eval(&[1, 2]), 16);
        assert_eq!(p.eval(&[2, 1]), 14);

        let p = (x() + y()) * (x() + y());
        assert_eq!(p.to_string(), "2 x y + x^2 + y^2");
    }
}
