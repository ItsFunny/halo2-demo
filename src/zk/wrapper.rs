use halo2_proofs::pairing::group::ff::PrimeField;

pub struct PrimeWrapper<F: PrimeField>(F);

impl<F: PrimeField> PrimeWrapper<F> {}
