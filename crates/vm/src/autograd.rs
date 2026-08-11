use std::sync::{Arc, Mutex};
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum BackwardOp {
    Add,
    Sub,
    Mul,
    Div,
    Matmul,
    Transpose,
    Neg,
    Relu,
    Sigmoid,
    Exp,
    Log,
    Sin,
    Cos,
    Tan,
    Pow,
    MseLoss,
    NoOp, // Untuk leaf nodes
}

#[derive(Clone)]
pub struct TapeNode {
    pub op: BackwardOp,
    // Kita simpan indeks-indeks tensor input yang berada di Heap
    pub parents: Vec<usize>,
    // Indeks tensor dari node ini sendiri
    pub self_tensor_idx: usize,
}

#[derive(Clone, Default)]
pub struct Tape {
    pub nodes: Vec<TapeNode>,
}

impl Tape {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: TapeNode) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }
}
