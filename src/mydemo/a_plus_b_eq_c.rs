// a+b=c

use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner};
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
// a | b | c | out | s
pub struct APlusBEqCConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub c: Column<Advice>,
    pub out: Column<Instance>,
    pub s: Selector,
}

pub struct APlusBEqCChip<F: FieldExt> {
    config: APlusBEqCConfig,
    _p: PhantomData<F>,
}
impl<F: FieldExt> APlusBEqCChip<F> {
    pub fn construct(config: APlusBEqCConfig) -> Self {
        Self {
            config,
            _p: Default::default(),
        }
    }
    pub fn configure(meta: &mut ConstraintSystem<F>) -> APlusBEqCConfig {
        let a_advice = meta.advice_column();
        let b_advice = meta.advice_column();
        let c_advice = meta.advice_column();
        let instance = meta.instance_column();
        let s = meta.selector();

        meta.enable_equality(instance);
        meta.enable_equality(c_advice);

        meta.create_gate("a+b=c", |meta| {
            let a_value = meta.query_advice(a_advice, Rotation::cur());
            let b_value = meta.query_advice(b_advice, Rotation::cur());
            let c_value = meta.query_advice(c_advice, Rotation::cur());
            let s = meta.query_selector(s);
            vec![s * (a_value + b_value - c_value)]
        });
        APlusBEqCConfig {
            a: a_advice,
            b: b_advice,
            c: c_advice,
            out: instance,
            s,
        }
    }

    pub fn enforce(
        &self,
        mut layout: impl Layouter<F>,
        v: AssignedCell<F, F>,
    ) -> Result<(), Error> {
        layout.constrain_instance(v.cell(), self.config.out, 0)
    }

    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        a: Option<F>,
        b: Option<F>,
    ) -> Result<AssignedCell<F, F>, Error> {
        layout.assign_region(
            || "assign",
            |mut region| {
                self.config.s.enable(&mut region, 0)?;

                let a_value = region.assign_advice(
                    || "assign a",
                    self.config.a,
                    0,
                    || a.ok_or(Error::Synthesis),
                )?;
                let b_value = region.assign_advice(
                    || "assign b",
                    self.config.b,
                    0,
                    || b.ok_or(Error::Synthesis),
                )?;
                let c_value: Option<F> = a.and_then(|a| b.and_then(|b| Some(b + a)));
                let c_value = region.assign_advice(
                    || "assign c",
                    self.config.c,
                    0,
                    || c_value.ok_or(Error::Synthesis),
                )?;

                Ok(c_value)
            },
        )
    }
}
#[derive(Default, Clone, Debug)]
pub struct APlusBEqCCircuit<F: FieldExt> {
    a: Option<F>,
    b: Option<F>,
}

impl<F: FieldExt> Circuit<F> for APlusBEqCCircuit<F> {
    type Config = APlusBEqCConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Default::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        APlusBEqCChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = APlusBEqCChip::construct(config);
        let after = chip.assign(layouter.namespace(|| "assign"), self.a, self.b)?;
        let after = chip.assign(layouter.namespace(|| "assign"), self.a, self.b)?;
        let after = chip.assign(layouter.namespace(|| "assign"), self.a, self.b)?;
        chip.enforce(layouter.namespace(|| "enfoce"), after)
    }
}

#[cfg(test)]
mod tests {
    use crate::mydemo::a_plus_b_eq_c::APlusBEqCCircuit;
    use halo2_proofs::dev::{CircuitCost, CircuitGates, MockProver};
    use halo2_proofs::pairing::bn256::Fr;
    use halo2_proofs::plonk::Error;
    use std::marker::PhantomData;

    #[test]
    pub fn test_success() {
        let k = 5;
        let a = Fr::from(1u64);
        let b = Fr::from(2u64);
        let out = a.clone() + b.clone();
        let circuit = APlusBEqCCircuit::<Fr> {
            a: Some(a.clone()),
            b: Some(b.clone()),
        };
        let public_inputs = vec![vec![out.clone()]];
        let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }
    #[test]
    pub fn test_wrong() {
        let k = 5;
        let a = Fr::from(1u64);
        let b = Fr::from(2u64);
        let out = a.clone() + b.clone() + Fr::from(4u64);
        let circuit = APlusBEqCCircuit::<Fr> {
            a: Some(a.clone()),
            b: Some(b.clone()),
        };
        let public_inputs = vec![vec![out.clone()]];
        let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
        assert_ne!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_fibonacci1() {
        use halo2_proofs::dev::circuit_dot_graph;
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("example-circuit-layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Example Circuit Layout", ("sans-serif", 60))
            .unwrap();

        let k = 5;
        let a = Fr::from(1u64);
        let b = Fr::from(2u64);
        let out = a.clone() + b.clone();
        let circuit = APlusBEqCCircuit::<Fr> {
            a: Some(a.clone()),
            b: Some(b.clone()),
        };
        halo2_proofs::dev::CircuitLayout::default()
            .render(5, &circuit, &root)
            .unwrap();
    }
}
