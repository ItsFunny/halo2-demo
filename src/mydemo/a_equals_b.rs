use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

// a==b
/// a | b | sel

#[derive(Clone, Debug)]
pub struct AEqBConfig {
    pub a_value: Column<Advice>,
    pub b_value: Column<Advice>,
    pub selector: Selector,
}
pub struct AEqBChip<F: FieldExt> {
    config: AEqBConfig,
    _mh: PhantomData<F>,
}

impl<F: FieldExt> AEqBChip<F> {
    pub fn construct(config: AEqBConfig) -> Self {
        Self {
            config,
            _mh: Default::default(),
        }
    }
    pub fn configure(meta: &mut ConstraintSystem<F>) -> AEqBConfig {
        let a_value = meta.advice_column();
        let b_value = meta.advice_column();
        let selector = meta.selector();

        meta.enable_equality(a_value);
        meta.enable_equality(b_value);

        // create gate
        // a-b=0
        meta.create_gate("a equal b", |meta| {
            let a = meta.query_advice(a_value, Rotation::cur());
            let b = meta.query_advice(b_value, Rotation::cur());
            let s = meta.query_selector(selector);

            vec![s * (a - b)]
        });
        AEqBConfig {
            a_value,
            b_value,
            selector,
        }
    }

    // 开始渲染
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        a: Option<F>,
        b: Option<F>,
    ) -> Result<(), Error> {
        layout.assign_region(
            || "assign region",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                let a_value = region.assign_advice(
                    || "assign a",
                    self.config.a_value,
                    0,
                    || a.ok_or(Error::Synthesis),
                )?;
                let b_value = region.assign_advice(
                    || "assign b",
                    self.config.b_value,
                    0,
                    || b.ok_or(Error::Synthesis),
                )?;

                Ok(())
            },
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct AEqbCircuit<F: FieldExt> {
    a: Option<F>,
    b: Option<F>,
}

impl<F: FieldExt> Circuit<F> for AEqbCircuit<F> {
    type Config = AEqBConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Default::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        AEqBChip::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = AEqBChip::construct(config);
        chip.assign(layouter, self.a, self.b)
    }
}

#[cfg(test)]
mod tests {
    use crate::mydemo::a_equals_b::AEqbCircuit;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::pairing::bn256::Fr;

    #[test]
    pub fn test_a_eq_b() {
        let k = 5;
        let a = Fr::from(2u64);
        let b = Fr::from(2u64);
        let circuit = AEqbCircuit::<Fr> {
            a: Some(a),
            b: Some(b),
        };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }
    #[test]
    pub fn test_not_eq() {
        let k = 5;
        let a = Fr::from(2u64);
        let b = Fr::from(3u64);
        let circuit = AEqbCircuit::<Fr> {
            a: Some(a),
            b: Some(b),
        };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_ne!(prover.verify(), Ok(()))
    }
}
