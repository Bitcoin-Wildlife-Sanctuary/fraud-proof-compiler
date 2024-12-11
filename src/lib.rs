use bitcoin::opcodes::all::{OP_RETURN_199, OP_RETURN_200};
use bitcoin::{Opcode, ScriptBuf};

pub mod structured_script;

pub mod code_cleanup;

pub mod op_return_true_to_op_if_return_true;

pub mod reduce;

pub mod final_emit;

#[cfg(test)]
mod integration_test;

#[allow(non_snake_case)]
pub const _OP_RETURN_TRUE: Opcode = OP_RETURN_199;

#[allow(non_snake_case)]
pub const _OP_IF_RETURN_TRUE: Opcode = OP_RETURN_200;

#[allow(non_snake_case)]
pub fn OP_RETURN_TRUE() -> ScriptBuf {
    ScriptBuf::from_bytes(vec![OP_RETURN_199.to_u8()])
}

#[allow(non_snake_case)]
pub fn OP_IF_RETURN_TRUE() -> ScriptBuf {
    ScriptBuf::from_bytes(vec![OP_RETURN_200.to_u8()])
}
