use super::field::Field;
use num_bigint::{BigUint, RandBigInt};
use std::rc::Rc;

#[derive(Debug)]
pub struct Polynomial {
    field: Rc<Field>,
    constant: BigUint,
    coefficients: Vec<BigUint>,
}

impl Polynomial {
    pub fn new(field: Rc<Field>, constant: BigUint, dim: usize) -> Self {
        assert!(
            &constant < field.prime(),
            "constant={:x?};prime={:x?}",
            &constant,
            field.prime()
        );
        let mut rng = rand::thread_rng();
        let coefficients: Vec<BigUint> = (0..dim)
            .map(|_| rng.gen_biguint_below(field.prime()))
            .collect();
        Self {
            field,
            constant,
            coefficients,
        }
    }

    pub fn call(&self, key: usize) -> BigUint {
        // let mut res = BigUint::zero();
        let mut x = BigUint::from(key);
        // for c in &self.coefficients {
        //     let t = (&x * c) % self.field.prime();
        //     res += t;
        //     x *= key;
        // }
        let res = self
            .coefficients
            .iter()
            .map(|c| {
                let t = &x * c;
                x *= key;
                t
            })
            .fold(self.constant.clone(), |acc, x| {
                (acc + &x) % self.field.prime()
            });
        //     .fold(BigUint::zero(), |acc, x| (acc + &x) % self.field.prime());
        // res += &self.constant;
        // res = res % self.field.prime();
        res
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use num_traits::Zero;

    #[test]
    fn test_polynomial() {
        let field = Rc::new(Field::new(32));
        let threshold = 2;
        let polynomial = Polynomial::new(field.clone(), BigUint::zero(), threshold - 1);
        assert_eq!(polynomial.coefficients.len(), 1);
        assert_eq!(polynomial.call(0), BigUint::zero());
        assert_eq!(polynomial.call(1), polynomial.coefficients[0]);
    }
}
