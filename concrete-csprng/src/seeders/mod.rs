//! A module containing seeders objects.
//!
//! When initializing a generator, one needs to provide a [`Seed`], which is then used as key to the
//! AES blockcipher. As a consequence, the quality of the outputs of the generator is directly
//! conditioned by the quality of this seed. This module proposes different mechanisms to deliver
//! seeds that can accommodate varying scenarios.

/// A seed value, used to initialize a generator.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Seed(pub u128);

/// A trait representing a seeding strategy.
pub trait Seeder {
    /// Generates a new seed.
    fn seed(&mut self) -> Seed;

    /// Check whether the seeder can be used on the current machine. This function may check if some
    /// required CPU features are available or if some OS features are available for example.
    fn is_available() -> bool
    where
        Self: Sized;
}

mod implem;
pub use implem::*;

#[cfg(test)]
mod generic_tests {
    use crate::seeders::Seeder;

    /// Naively verifies that two fixed-size sequences generated by repeatedly calling the seeder
    /// are different.
    #[allow(unused)] // to please clippy when tests are not activated
    pub fn check_seeder_fixed_sequences_different<S: Seeder, F: Fn(u128) -> S>(
        construct_seeder: F,
    ) {
        const SEQUENCE_SIZE: usize = 500;
        const REPEATS: usize = 10_000;
        for i in 0..REPEATS {
            let mut seeder = construct_seeder(i as u128);
            let orig_seed = seeder.seed();
            for _ in 0..SEQUENCE_SIZE {
                assert_ne!(seeder.seed(), orig_seed);
            }
        }
    }
}
