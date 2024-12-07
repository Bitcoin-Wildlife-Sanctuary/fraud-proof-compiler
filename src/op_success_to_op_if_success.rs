use crate::structured_script::{OwnedInstruction, OwnedInstructions, StructuredScript};
use crate::{_OP_IF_SUCCESS, _OP_SUCCESS};
use bitcoin::opcodes::all::OP_NOT;
use bitcoin::opcodes::OP_TRUE;

pub fn op_success_to_op_if_success(structure: &mut StructuredScript) {
    match structure {
        StructuredScript::Script(v) => {
            let mut res = vec![];
            let len = v.0.len();

            for i in 0..len {
                if v.0[i] == OwnedInstruction::Op(_OP_SUCCESS) {
                    res.push(OwnedInstruction::Op(OP_TRUE));
                    res.push(OwnedInstruction::Op(_OP_IF_SUCCESS));
                } else {
                    res.push(v.0[i].clone());
                }
            }

            v.0 = res;
        }
        StructuredScript::MultiScript(vv) => {
            let mut len = vv.len();

            for mut i in 0..len {
                op_success_to_op_if_success(&mut vv[i]);
                if i > 0
                    && matches!(vv[i - 1], StructuredScript::Script(_))
                    && matches!(vv[i], StructuredScript::Script(_))
                {
                    let mut instructions = vv[i].clone();
                    match &mut vv[i - 1] {
                        StructuredScript::Script(v1) => {
                            if let StructuredScript::Script(v2) = &mut instructions {
                                v1.0.append(&mut v2.0);
                            }
                            vv.remove(i);
                            i -= 1;
                            len -= 1;
                        }
                        _ => unreachable!(),
                    }
                }
            }

            if vv.len() == 1 {
                let res = vv.pop().unwrap();
                *structure = res;
            }
        }
        StructuredScript::IfEndIf(v, _) => {
            op_success_to_op_if_success(v);

            if *v.as_ref()
                == StructuredScript::Script(OwnedInstructions(vec![
                    OwnedInstruction::Op(OP_TRUE),
                    OwnedInstruction::Op(_OP_IF_SUCCESS),
                ]))
            {
                *structure =
                    StructuredScript::Script(OwnedInstructions(vec![OwnedInstruction::Op(
                        _OP_IF_SUCCESS,
                    )]));
            }
        }
        StructuredScript::NotIfEndIf(v, _) => {
            op_success_to_op_if_success(v);

            if *v.as_ref()
                == StructuredScript::Script(OwnedInstructions(vec![
                    OwnedInstruction::Op(OP_TRUE),
                    OwnedInstruction::Op(_OP_IF_SUCCESS),
                ]))
            {
                *structure = StructuredScript::Script(OwnedInstructions(vec![
                    OwnedInstruction::Op(OP_NOT),
                    OwnedInstruction::Op(_OP_IF_SUCCESS),
                ]));
            }
        }
        StructuredScript::IfElseEndIf(v1, _, v2, _)
        | StructuredScript::NotIfElseEndIf(v1, _, v2, _) => {
            op_success_to_op_if_success(v1);
            op_success_to_op_if_success(v2);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::op_success_to_op_if_success::op_success_to_op_if_success;
    use crate::structured_script::StructuredScript;
    use crate::{OP_IF_SUCCESS, OP_SUCCESS};
    use bitcoin_script::{define_pushable, script};

    define_pushable!();

    #[test]
    fn test_conversion() {
        let script = script! {
            OP_NOP1
            OP_SUCCESS
            OP_NOP2
            OP_IF
                OP_NOP3
                OP_NOTIF
                    OP_NOP4
                OP_ELSE
                    OP_NOP5
                    OP_IF
                        OP_SUCCESS
                    OP_ENDIF
                OP_ENDIF
            OP_ENDIF
            OP_NOP6
            OP_SUCCESS
        };

        let mut structured_script = StructuredScript::<()>::from(script);
        op_success_to_op_if_success(&mut structured_script);

        let expected_script = script! {
            OP_NOP1
            1 OP_IF_SUCCESS
            OP_NOP2
            OP_IF
                OP_NOP3
                OP_NOTIF
                    OP_NOP4
                OP_ELSE
                    OP_NOP5
                    OP_IF_SUCCESS
                OP_ENDIF
            OP_ENDIF
            OP_NOP6
            1 OP_IF_SUCCESS
        };

        let expected = StructuredScript::from(expected_script);
        assert_eq!(expected, structured_script);
    }
}
