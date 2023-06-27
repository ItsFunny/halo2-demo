// a+b=c

use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Instance, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

pub struct APlusBEqCConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub c: Column<Instance>,
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
    pub fn configure(&self, meta: &mut ConstraintSystem<F>) -> APlusBEqCConfig {
        let a_advice = meta.advice_column();
        let b_advice = meta.advice_column();
        let instance = meta.instance_column();
        let s = meta.selector();

        meta.create_gate("a+b=c", |meta| {
            let a_value = meta.query_advice(a_advice, Rotation::cur());
            let b_value = meta.query_advice(b_advice, Rotation::cur());
            let instance = meta.query_instance(instance, Rotation::cur());
            let s = meta.query_selector(s);
            vec![s * (a_value + b_value - instance)]
        });
        APlusBEqCConfig {
            a: a_advice,
            b: b_advice,
            c: instance,
            s,
        }
    }
}
