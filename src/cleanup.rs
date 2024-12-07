use crate::structured_script::{OwnedInstruction, StructuredScript};
use crate::{_OP_IF_SUCCESS, _OP_SUCCESS};
use bitcoin::opcodes::all::{
    OP_PUSHNUM_1, OP_PUSHNUM_10, OP_PUSHNUM_11, OP_PUSHNUM_12, OP_PUSHNUM_13, OP_PUSHNUM_14,
    OP_PUSHNUM_15, OP_PUSHNUM_16, OP_PUSHNUM_2, OP_PUSHNUM_3, OP_PUSHNUM_4, OP_PUSHNUM_5,
    OP_PUSHNUM_6, OP_PUSHNUM_7, OP_PUSHNUM_8, OP_PUSHNUM_9, OP_PUSHNUM_NEG1,
};

fn find_op_success_cleanup(structure: &mut StructuredScript) -> bool {
    match structure {
        StructuredScript::Script(v) => {
            let len = v.0.len();

            for i in 0..len {
                if v.0[i] == OwnedInstruction::Op(_OP_SUCCESS) {
                    v.0.truncate(i + 1);
                    return true;
                }

                if let OwnedInstruction::Op(op) = &v.0[i] {
                    if i < len - 1
                        && v.0[i + 1] == OwnedInstruction::Op(_OP_IF_SUCCESS)
                        && [
                            OP_PUSHNUM_1,
                            OP_PUSHNUM_2,
                            OP_PUSHNUM_3,
                            OP_PUSHNUM_4,
                            OP_PUSHNUM_NEG1,
                            OP_PUSHNUM_5,
                            OP_PUSHNUM_6,
                            OP_PUSHNUM_7,
                            OP_PUSHNUM_8,
                            OP_PUSHNUM_9,
                            OP_PUSHNUM_10,
                            OP_PUSHNUM_11,
                            OP_PUSHNUM_12,
                            OP_PUSHNUM_13,
                            OP_PUSHNUM_14,
                            OP_PUSHNUM_15,
                            OP_PUSHNUM_16,
                        ]
                        .contains(op)
                    {
                        v.0.truncate(i);
                        v.0.push(OwnedInstruction::Op(_OP_SUCCESS));
                        return true;
                    }
                }

                if let OwnedInstruction::PushBytes(p) = &v.0[i] {
                    if !p.is_empty()
                        && i < len - 1
                        && v.0[i + 1] == OwnedInstruction::Op(_OP_IF_SUCCESS)
                    {
                        v.0.truncate(i);
                        v.0.push(OwnedInstruction::Op(_OP_SUCCESS));
                        return true;
                    }
                }
            }
            false
        }
        StructuredScript::MultiScript(vv) => {
            let len = vv.len();

            for i in 0..len {
                let res = find_op_success_cleanup(&mut vv[i]);
                if res {
                    vv.truncate(i + 1);

                    if vv.len() == 1 {
                        let content = vv[0].clone();
                        *structure = content;
                    }

                    return true;
                }
            }
            false
        }
        StructuredScript::IfEndIf(v, _) | StructuredScript::NotIfEndIf(v, _) => {
            find_op_success_cleanup(v);
            false
        }
        StructuredScript::IfElseEndIf(v1, _, v2, _)
        | StructuredScript::NotIfElseEndIf(v1, _, v2, _) => {
            find_op_success_cleanup(v1);
            find_op_success_cleanup(v2);
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cleanup::find_op_success_cleanup;
    use crate::structured_script::StructuredScript;
    use crate::{OP_IF_SUCCESS, OP_SUCCESS};
    use bitcoin_script::{define_pushable, script};

    define_pushable!();

    #[test]
    fn test_cleanup_nothing() {
        let script = script! {
            OP_NOP1
            OP_IF
                OP_NOP2
                OP_SUCCESS
                OP_NOP3
                OP_NOTIF
                    OP_NOP4
                OP_ENDIF
            OP_ENDIF
            OP_NOP5
            12 OP_IF_SUCCESS
            OP_NOP6
        };

        let mut structured_script = StructuredScript::<()>::from(script);
        find_op_success_cleanup(&mut structured_script);

        let expected_script = script! {
            OP_NOP1
            OP_IF
                OP_NOP2
                OP_SUCCESS
            OP_ENDIF
            OP_NOP5
            OP_SUCCESS
        };

        let expected = StructuredScript::from(expected_script);
        assert_eq!(expected, structured_script);
    }
}
