use halo2_proofs::plonk::TableColumn;

// 这是一个look up table,用于判断是否在num_bits,比如说 NUM_BITS=8,则这个table可以判断[0,255]
#[derive(Debug, Clone)]
pub struct RangeCheckTable {
    value: TableColumn,
}
