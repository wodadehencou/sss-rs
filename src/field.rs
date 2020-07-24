use super::num_primes;
use num_bigint::BigUint;

#[derive(Clone, Debug)]
pub struct Field {
    prime: BigUint,
}

impl Field {
    pub fn new(n: usize) -> Self {
        assert!(n > 16);
        Self {
            prime: num_primes::Generator::new_prime(n as u64),
        }
    }

    pub fn prime(&self) -> &BigUint {
        &self.prime
    }
}
