use crate::structured_script::{OwnedInstruction, OwnedInstructions, StructuredScript};
use crate::{_OP_IF_RETURN_TRUE, _OP_RETURN_TRUE};
use bitcoin::opcodes::all::OP_NOT;
use bitcoin::opcodes::OP_TRUE;

pub fn op_return_true_to_op_if_return_true(structure: &mut StructuredScript) {
    match structure {
        StructuredScript::Script(v) => {
            let mut res = vec![];
            let len = v.0.len();

            for i in 0..len {
                if v.0[i] == OwnedInstruction::Op(_OP_RETURN_TRUE) {
                    res.push(OwnedInstruction::Op(OP_TRUE));
                    res.push(OwnedInstruction::Op(_OP_IF_RETURN_TRUE));
                } else {
                    res.push(v.0[i].clone());
                }
            }

            v.0 = res;
        }
        StructuredScript::MultiScript(vv) => {
            let mut len = vv.len();

            let mut i = 0;
            while i < len {
                op_return_true_to_op_if_return_true(&mut vv[i]);
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
                i += 1;
            }

            if vv.len() == 1 {
                let res = vv.pop().unwrap();
                *structure = res;
            }
        }
        StructuredScript::IfEndIf(v) => {
            op_return_true_to_op_if_return_true(v);

            if *v.as_ref()
                == StructuredScript::Script(OwnedInstructions(vec![
                OwnedInstruction::Op(OP_TRUE),
                OwnedInstruction::Op(_OP_IF_RETURN_TRUE),
                ]))
            {
                *structure =
                    StructuredScript::Script(OwnedInstructions(vec![OwnedInstruction::Op(
                        _OP_IF_RETURN_TRUE,
                    )]));
            }
        }
        StructuredScript::NotIfEndIf(v) => {
            op_return_true_to_op_if_return_true(v);

            if *v.as_ref()
                == StructuredScript::Script(OwnedInstructions(vec![
                OwnedInstruction::Op(OP_TRUE),
                OwnedInstruction::Op(_OP_IF_RETURN_TRUE),
                ]))
            {
                *structure = StructuredScript::Script(OwnedInstructions(vec![
                    OwnedInstruction::Op(OP_NOT),
                    OwnedInstruction::Op(_OP_IF_RETURN_TRUE),
                ]));
            }
        }
        StructuredScript::IfElseEndIf(v1, v2) | StructuredScript::NotIfElseEndIf(v1, v2) => {
            op_return_true_to_op_if_return_true(v1);
            op_return_true_to_op_if_return_true(v2);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::op_return_true_to_op_if_return_true::op_return_true_to_op_if_return_true;
    use crate::structured_script::StructuredScript;
    use crate::{OP_IF_RETURN_TRUE, OP_RETURN_TRUE};
    use bitcoin_script::{define_pushable, script};

    define_pushable!();

    #[test]
    fn test_conversion() {
        let script = script! {
            OP_NOP1
            OP_RETURN_TRUE
            OP_NOP2
            OP_IF
                OP_NOP3
                OP_NOTIF
                    OP_NOP4
                OP_ELSE
                    OP_NOP5
                    OP_IF
                        OP_RETURN_TRUE
                    OP_ENDIF
                OP_ENDIF
            OP_ENDIF
            OP_NOP6
            OP_RETURN_TRUE
        };

        let mut structured_script = StructuredScript::from(script);
        op_return_true_to_op_if_return_true(&mut structured_script);

        let expected_script = script! {
            OP_NOP1
            1 OP_IF_RETURN_TRUE
            OP_NOP2
            OP_IF
                OP_NOP3
                OP_NOTIF
                    OP_NOP4
                OP_ELSE
                    OP_NOP5
                    OP_IF_RETURN_TRUE
                OP_ENDIF
            OP_ENDIF
            OP_NOP6
            1 OP_IF_RETURN_TRUE
        };

        let expected = StructuredScript::from(expected_script);
        assert_eq!(expected, structured_script);
    }
}
