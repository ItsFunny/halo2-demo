extern crate core;

pub mod example1;
mod mydemo;
mod range_check;

use halo2_proofs::dev::MockProver;
use halo2_proofs::pairing::bn256::Fr;
use halo2_proofs::poly::Rotation;
use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*};
use std::marker::PhantomData;

// 1. 定义circuit config
#[derive(Debug, Clone)]
pub struct FiboConfig {
    pub advice: [Column<Advice>; 3],
    pub selector: Selector,
    pub instance: Column<Instance>,
}

// 2. 定义chip
pub struct FiboChip<F: FieldExt> {
    config: FiboConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone)]
pub struct ACell<F: FieldExt>(AssignedCell<F, F>);

impl<F: FieldExt> FiboChip<F> {
    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &ACell<F>,
        raw: usize,
    ) -> Result<(), Error> {
        // 相当于bellman 的enforce
        layouter.constrain_instance(cell.0.cell(), self.config.instance, raw)
    }
    pub(crate) fn assign_new_raw(
        &self,
        mut layouter: impl Layouter<F>,
        prev_b: &ACell<F>,
        prev_c: &ACell<F>,
    ) -> Result<ACell<F>, Error> {
        // 开始新的一个region,也可以认为是新的一行
        layouter.assign_region(
            || "new row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                // 复制约束
                // 将 [0,1],[0,2]的值复制给 [1,0],[1,1]
                prev_b
                    .0
                    .copy_advice(|| "copy b ", &mut region, self.config.advice[0], 0)?;
                prev_c
                    .0
                    .copy_advice(|| "copy c ", &mut region, self.config.advice[1], 0)?;

                // 根据fib的性质, 通过prev_b,prev_c的值计算新的c的值
                let c_val = prev_b
                    .0
                    .value()
                    .and_then(|b| prev_c.0.value().map(|c| *b + *c));
                // 然后给第3列 advice 赋值
                let c_cell = region
                    .assign_advice(
                        || "assign new c",
                        self.config.advice[2],
                        0,
                        || c_val.ok_or(Error::Synthesis),
                    )
                    .map(|v| ACell(v))?;
                Ok(c_cell)
            },
        )
    }

    pub(crate) fn assign_first_row(
        &self,
        mut layouter: impl Layouter<F>,
        a: Option<F>,
        b: Option<F>,
    ) -> Result<(ACell<F>, ACell<F>, ACell<F>), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                // 这段代码就相当于将 第一行的selector 置为true
                self.config.selector.enable(&mut region, 0)?;
                //  为什么这里的offset 要设置为0, 这是因为根据上面的表来看,a advice是在第一行的第一列,offset是个相对偏移量,所以是0,如果在第2行,则为1
                let a_cell = region
                    .assign_advice(
                        || "a",
                        self.config.advice[0],
                        0,
                        || a.ok_or(Error::Synthesis),
                    )
                    .map(|v| ACell(v))?;

                let b_cell = region
                    .assign_advice(
                        || "b",
                        self.config.advice[1],
                        0,
                        || a.ok_or(Error::Synthesis),
                    )
                    .map(|v| ACell(v))?;

                // 开始给c赋值
                let c_val = a.and_then(|a| b.map(|b| a + b));
                let c_cell = region
                    .assign_advice(
                        || "c",
                        self.config.advice[2],
                        0,
                        || c_val.ok_or(Error::Synthesis),
                    )
                    .map(|v| ACell(v))?;

                // 为什么要return 这些cell
                // 因为: 根据复制约束来定,第二行的[1,0],[1,1] 是由第一行的[0,1],[0,2] 而来的,所以需要返回,然后调用copy_advice赋值给下一行
                Ok((a_cell, b_cell, c_cell))
            },
        )
    }
}

// 3. define chip
impl<F: FieldExt> FiboChip<F> {
    fn construct(config: FiboConfig) -> Self {
        Self {
            config,
            _marker: Default::default(),
        }
    }
    // 只是定义custom_gate,相当于配置,数据啥的此时都是不可知的
    fn configure(meta: &mut ConstraintSystem<F>) -> FiboConfig {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let selector = meta.selector();
        let instance = meta.instance_column();

        // 因为根据fib 的特性
        // col_a | col_b | col_c|selector
        // 1        1       2
        // 1        2       3  // [1,0]是由[0,1]获得, [1,1] 是由[0,2] 获得,这种具有前后关系的情况下,就得进行约束,所以要enable, 这种也叫permutation argument
        // 加上这个约束之后,就可以有equality check,确保值是一样的(既由上一个延伸过来的)
        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_c);
        meta.enable_equality(instance);

        // 这样就完成了一个custom gate的便携
        meta.create_gate("add", |meta| {
            // col_a | col_b | col_c|selector
            // a        b       c       s
            //
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());

            vec![s * (a + b - c)]
        });
        FiboConfig {
            advice: [col_a, col_b, col_c],
            selector,
            instance,
        }
    }
}

#[derive(Default)]
pub struct MyCircuit<F: FieldExt> {
    pub a: Option<F>,
    pub b: Option<F>,
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    type Config = FiboConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Default::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        FiboChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FiboChip::construct(config);
        // 开始给table 布局,fib的特性,需要先给第一行赋值,所以写一个fist
        let (mut prev_a, mut prev_b, mut prev_c) =
            chip.assign_first_row(layouter.namespace(|| "assign fist row"), self.a, self.b)?;
        // 约束判断
        chip.expose_public(layouter.namespace(|| "expose a"), &prev_a, 0)?;
        chip.expose_public(layouter.namespace(|| "expose c"), &prev_a, 1)?;
        // 因为我们要证明的是fib(9)=v
        // 而在第一行已经实现了 fib(1) 和fib(2)
        // 所以接下来的继续assign 6次即可
        for _i in 3..9 {
            let new_c: ACell<F> =
                chip.assign_new_raw(layouter.namespace(|| "assign new row"), &prev_b, &prev_c)?;
            prev_b = prev_c;
            prev_c = new_c;
        }
        chip.expose_public(layouter.namespace(|| "expose "), &prev_c, 2)?;
        Ok(())
    }
}

fn main() {
    let k = 4;
    let a = Fr::from(1);
    let b = Fr::from(1);
    let circuit = MyCircuit {
        a: Some(a),
        b: Some(b),
    };
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    assert_eq!(prover.verify(), Ok(()))
}
