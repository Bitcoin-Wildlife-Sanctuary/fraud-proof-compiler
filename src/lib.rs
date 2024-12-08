use bitcoin::opcodes::all::{OP_RETURN_199, OP_RETURN_200};
use bitcoin::{Opcode, ScriptBuf};

pub mod structured_script;

pub mod cleanup;

pub mod op_success_to_op_if_success;

pub mod reduce;

#[allow(non_snake_case)]
pub const _OP_SUCCESS: Opcode = OP_RETURN_199;

#[allow(non_snake_case)]
pub const _OP_IF_SUCCESS: Opcode = OP_RETURN_200;

#[allow(non_snake_case)]
pub fn OP_SUCCESS() -> ScriptBuf {
    ScriptBuf::from_bytes(vec![OP_RETURN_199.to_u8()])
}

#[allow(non_snake_case)]
pub fn OP_IF_SUCCESS() -> ScriptBuf {
    ScriptBuf::from_bytes(vec![OP_RETURN_200.to_u8()])
}
