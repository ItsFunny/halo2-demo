use halo2_proofs::arithmetic::{Field, FieldExt};
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

// 判断一个数是否在某个区间内
// value | range-left | range-right | s
// 1  [0 ,10] => (1 -0)*(1-10)*(-10*())
#[derive(Clone, Debug)]
pub struct CircuitConfig {
    value: Column<Advice>,
    s: Selector,
}

pub struct CircuitChip<F: FieldExt> {
    config: CircuitConfig,
    _ph: PhantomData<F>,
}
impl<F: FieldExt> CircuitChip<F> {
    fn configure(meta: &mut ConstraintSystem<F>, range_right: u64) -> CircuitConfig {
        let value = meta.advice_column();
        let s = meta.selector();
        // 数学公式:v*(1-v)*(2-v)*...(R-1-V)=0
        meta.create_gate("value in range [a,b]", |meta| {
            let value = meta.query_advice(value, Rotation::cur());
            let s = meta.query_selector(s);
            let range_check = |right| {
                (1..right).fold(value.clone(), |a, b| {
                    a * (Expression::Constant(F::from(b)) - value.clone())
                })
            };
            vec![s * range_check(range_right)]
        });
        CircuitConfig { value, s }
    }

    fn construct(config: CircuitConfig) -> Self {
        Self {
            config,
            _ph: Default::default(),
        }
    }

    fn assign(&self, mut layout: impl Layouter<F>, value: Option<F>) -> Result<(), Error> {
        layout.assign_region(
            || "assign",
            |mut region| {
                self.config.s.enable(&mut region, 0)?;
                region.assign_advice(
                    || "assign value",
                    self.config.value,
                    0,
                    || value.ok_or(Error::Synthesis),
                )?;
                Ok(())
            },
        )
    }
}

#[derive(Default)]
pub struct MyCircuit<F: FieldExt, const RANGE: u64> {
    value: Option<F>,
}

impl<F: FieldExt, const RANGE: u64> Circuit<F> for MyCircuit<F, RANGE> {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Default::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        CircuitChip::configure(meta, RANGE)
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = CircuitChip::construct(config);
        chip.assign(layouter, self.value)
    }
}
#[cfg(test)]
mod tests {
    use crate::mydemo::range_check::MyCircuit;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::pairing::bn256::Fr;

    #[test]
    pub fn test_success() {
        let k = 5;
        let a = Fr::from(2u64);
        let circuit = MyCircuit::<Fr, 6> { value: Some(a) };
        let public_inputs = vec![];
        let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }
    #[test]
    pub fn test_wrong() {
        let k = 5;
        let a = Fr::from(10u64);
        let circuit = MyCircuit::<Fr, 6> { value: Some(a) };
        let public_inputs = vec![];
        let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
        assert_ne!(prover.verify(), Ok(()))
    }
}
