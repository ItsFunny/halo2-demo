use crate::zk::wrapper::PrimeWrapper;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Layouter;
use halo2_proofs::pairing::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector};

pub trait Halo2BoxConfig<F: FieldExt> {}
pub trait SBox<F: FieldExt> {
    fn apply(&self, elements: &mut [F]);
}
pub trait CsSBox<F: FieldExt>: SBox<F> {
    type Config;
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config;

    fn assign_constraints(
        &self,
        cs: impl Layouter<F>,
        element: &PrimeWrapper<F>,
    ) -> Result<PrimeWrapper<F>, Error>;
    fn assign_constraints_on_lc_for_set(
        &self,
        mut cs: impl Layouter<F>,
        elements: Vec<Num<E>>,
    ) -> Result<Vec<Num<E>>, SynthesisError> {
        let mut results = Vec::with_capacity(elements.len());
        for (i, el) in elements.into_iter().enumerate() {
            let applied = self.apply_constraints_on_lc(cs.namespace(|| "assign "), el)?;
            results.push(applied)
        }

        Ok(results)
    }
}
#[cfg(test)]
mod tests {
    #[test]
    pub fn it_works() {}
}
