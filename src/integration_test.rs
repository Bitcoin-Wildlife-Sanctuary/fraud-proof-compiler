use crate::code_cleanup::find_op_success_cleanup;
use crate::final_emit::append_final_emit_script;
use crate::op_success_to_op_if_success::op_success_to_op_if_success;
use crate::reduce::{reduce, EmitOpIfSuccess};
use crate::structured_script::StructuredScript;
use crate::{OP_IF_SUCCESS, OP_SUCCESS};
use bitcoin::ScriptBuf;
use bitcoin_script::{define_pushable, script};
use bitcoin_scriptexec::execute_script_with_witness;
use bitcoin_scriptexec::ExecError::OpReturn;

define_pushable!();

#[test]
fn test_success() {
    let script = script! {
        1 2 3 4
        1
        OP_IF
            OP_DEPTH OP_1SUB OP_PICK
            10001 OP_EQUAL
            OP_IF
                OP_SUCCESS
            OP_ENDIF

            5 6 7 8
            OP_DEPTH OP_1SUB OP_PICK
            10002 OP_EQUAL
            OP_IF_SUCCESS

            0 OP_IF_SUCCESS

            0 0
            OP_IF
                OP_SUCCESS
            OP_ELSE
                OP_IF_SUCCESS
            OP_ENDIF

            9 10 11 12
        OP_ENDIF
        OP_RETURN
    };

    let mut structured_script: StructuredScript = script.into();
    find_op_success_cleanup(&mut structured_script);
    op_success_to_op_if_success(&mut structured_script);

    let emit = reduce(&mut structured_script);
    assert_eq!(emit, EmitOpIfSuccess::YES);

    append_final_emit_script(&mut structured_script);

    let script: ScriptBuf = structured_script.into();
    let res = execute_script_with_witness(script.clone(), vec![vec![0x11, 0x27]]);
    assert!(res.success);

    let res = execute_script_with_witness(script.clone(), vec![vec![0x12, 0x27]]);
    assert!(res.success);

    let res = execute_script_with_witness(script.clone(), vec![vec![0x13, 0x27]]);
    assert_eq!(res.success, false);
    assert_eq!(res.error, Some(OpReturn));

    let res = execute_script_with_witness(script, vec![]);
    assert_eq!(res.success, false);
    assert_eq!(res.error, Some(OpReturn));
}
