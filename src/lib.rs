// #![no_std]

mod field;

#[cfg(feature = "benchmark")]
pub mod num_primes;
#[cfg(not(feature = "benchmark"))]
mod num_primes;

mod polynomial;

use field::Field;
use num_bigint::BigUint;
use num_traits::identities::{One, Zero};
use polynomial::Polynomial;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SSSError {
    #[error("No shares")]
    NoShares,
}

#[derive(Debug)]
pub struct Value {
    chunk_size: usize,
    int: BigUint,
}

#[derive(Debug)]
pub struct Share {
    key: usize,
    val: Vec<Value>,
    field: Rc<Field>,
}

impl Share {
    pub fn prime(&self) -> &BigUint {
        self.field.prime()
    }
}

/// # input parameters
///   secret: message that should not be known by others
///   chunk_size: byte length of a piece of data; secret = chunk_size * piece_number
///   n: number of shares
///   threshold: min number of shares to reconstruct the secret
/// # output parameters
pub fn distribute(secret: &[u8], chunk_size: usize, n: usize, threshold: usize) -> Vec<Share> {
    let field = Rc::new(Field::new(chunk_size * 8 + 1));

    let mut shares: Vec<Share> = (0..n)
        .map(|key| Share {
            key: key + 1,
            val: Vec::new(),
            field: field.clone(),
        })
        .collect();

    secret.chunks(chunk_size as usize).for_each(|secret| {
        let polynomial =
            Polynomial::new(field.clone(), BigUint::from_bytes_le(secret), threshold - 1);
        shares.iter_mut().for_each(|s| {
            s.val.push(Value {
                chunk_size: secret.len(),
                int: polynomial.call(s.key),
            });
        });
    });
    shares
}

pub fn reconstruct(shares: &[Share]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if shares.len() == 0 {
        return Err(SSSError::NoShares.into());
    }
    let prime = shares[0].field.prime();

    let betas: Vec<BigUint> = shares.iter().map(|s| beta(prime, s, shares)).collect();

    let mut res = Vec::new();
    // let mut acc;
    // for chunk in 0..shares[0].val.len() {
    shares[0]
        .val
        .iter()
        .enumerate()
        .for_each(|(chunk, share0_val)| {
            let acc = shares
                .iter()
                .enumerate()
                .map(|(i, s)| &s.val[chunk].int * &betas[i])
                .fold(BigUint::zero(), |acc, x| (acc + &x) % prime);
            // acc %= prime;
            let mut bytes = acc.to_bytes_le();
            bytes.resize(share0_val.chunk_size, 0);
            res.append(&mut bytes);
        });

    Ok(res)
}

fn beta(prime: &BigUint, current: &Share, shares: &[Share]) -> BigUint {
    let pm2: BigUint = prime - 2u32;
    let mut res = BigUint::one();
    for s in shares {
        if s.key == current.key {
            continue;
        }
        let xi = BigUint::from(s.key);
        let x = BigUint::from(current.key);
        let mut sub = prime - &x; // -x
        sub += &xi; //xi - x
        sub %= prime;
        sub = sub.modpow(&pm2, prime); // 1/(xi-x)
        sub *= &xi; //xi/(xi-x)
        res *= sub;
        res %= prime;
    }
    res
}

#[cfg(feature = "benchmark")]
#[inline]
pub fn beta_wrapper(prime: &BigUint, current: &Share, shares: &[Share]) -> BigUint {
    beta(prime, current, shares)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::Rng;

    #[test]
    #[should_panic]
    fn test_uintsub() {
        let a = BigUint::from(3u32);
        let b = BigUint::from(4u32);
        let _ = a - b;
    }

    #[test]
    fn test_sss() {
        const SECRET: &str = "abcdefghijklmnopqrstuvwxyz";
        const SIZE: usize = 10;
        const THRESHOLD: usize = 8;

        let mut shares = distribute(SECRET.as_bytes(), 8, SIZE, THRESHOLD);
        println!("origin is {:x?}", SECRET.as_bytes());

        let act = reconstruct(&shares[..THRESHOLD + 1]).unwrap();
        println!("acture1 is {:x?}", act.as_slice());
        let res = std::str::from_utf8(act.as_slice()).unwrap();
        assert_eq!(SECRET, res);

        let mut rng = rand::thread_rng();
        shares.shuffle(&mut rng);
        let act = reconstruct(&shares[..THRESHOLD + 1]).unwrap();
        println!("acture2 is {:x?}", act.as_slice());
        let res = std::str::from_utf8(act.as_slice()).unwrap();
        assert_eq!(SECRET, res);

        shares.shuffle(&mut rng);
        let act = reconstruct(&shares[..THRESHOLD + 1]).unwrap();
        println!("acture3 is {:x?}", act.as_slice());
        let res = std::str::from_utf8(act.as_slice()).unwrap();
        assert_eq!(SECRET, res);
    }

    #[test]
    fn test_multi_times() {
        let size = 10;
        let threshold = 10;

        let mut rng = rand::thread_rng();

        (0..100).for_each(|_| {
            let secret: Vec<u8> = (0..rng.gen_range(1, 1024)).map(|_| rng.gen()).collect();

            let mut shares = distribute(&secret, 16, size, size);
            shares.shuffle(&mut rng);
            let act = reconstruct(&shares[0..threshold]).unwrap();
            assert_eq!(secret.to_vec(), act);
        });
    }
}
