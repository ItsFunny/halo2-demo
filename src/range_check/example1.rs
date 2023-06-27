use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::Layouter;
use halo2_proofs::plonk::{
    Advice, Assigned, Column, Constraint, ConstraintSystem, Error, Expression, Selector,
};
use halo2_proofs::poly::Rotation;
/// value|q_range_check
use halo2_proofs::*;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct RangeCheckConfig<F: FieldExt, const RANGE: usize> {
    value: Column<Advice>,
    q_range_check: Selector,
    _m: PhantomData<F>,
}

impl<F: FieldExt, const RANGE: usize> RangeCheckConfig<F, RANGE> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        value: Column<Advice>,
    ) -> Result<RangeCheckConfig<F, RANGE>, Error> {
        let q_range_check = meta.selector();
        let config = Self {
            value,
            q_range_check,
            _m: Default::default(),
        };

        // 开始编写range check的电路
        // 需求是判断某个数是否在某个区间内
        // 既 value v and a range R , v<R
        // 转换为数学表达式就是:
        // v*(1-v)*(2-v)*...(R-1-V)=0
        meta.create_gate("range check gate", |mut meta| {
            let q_range_check = meta.query_selector(q_range_check);
            let value = meta.query_advice(value, Rotation::cur());
            let range_check = |range: usize, value: Expression<F>| {
                (0..RANGE).fold(value.clone(), |expr, i| {
                    expr * (Expression::Constant(F::from(i as u64)) - value.clone())
                })
            };

            vec![q_range_check * (range_check(RANGE, value.clone()))]
        });
        Ok(config)
    }
    pub(crate) fn assign(
        &self,
        mut layout: impl Layouter<F>,
        value: Option<F>,
    ) -> Result<(), Error> {
        layout.assign_region(
            || "assign",
            |mut region| {
                // 因为是一个新的region,所以offset 为0 即可
                let OFFSET = 0;
                self.q_range_check.enable(&mut region, OFFSET)?;

                // 赋值advice
                region.assign_advice(
                    || "assign",
                    self.value,
                    OFFSET,
                    || value.ok_or(Error::Synthesis),
                )?;

                Ok(())
            },
        )
    }
}
#[cfg(test)]
mod tests {
    use crate::range_check::example1::RangeCheckConfig;
    use halo2_proofs::arithmetic::FieldExt;
    use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner};
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::pairing::bn256::Fr;
    use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};

    #[derive(Default)]
    pub struct MyCircuit<F: FieldExt, const RANGE: usize> {
        value: Option<F>,
    }
    impl<F: FieldExt, const RANGE: usize> Circuit<F> for MyCircuit<F, RANGE> {
        type Config = RangeCheckConfig<F, RANGE>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            let value = meta.advice_column();
            RangeCheckConfig::configure(meta, value).unwrap()
        }

        fn synthesize(
            &self,
            config: Self::Config,
            layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            config.assign(layouter, self.value)
        }
    }
    #[test]
    pub fn test_range_check() {
        const RANGE: usize = 8;
        for i in 0..RANGE {
            let circuit = MyCircuit::<Fr, RANGE> {
                value: Some(Fr::from(i as u64)),
            };
            let prover = MockProver::run(4, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
