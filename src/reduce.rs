use crate::structured_script::{OwnedInstruction, OwnedInstructions, StructuredScript};
use crate::_OP_IF_SUCCESS;
use bitcoin::opcodes::all::OP_PUSHNUM_1;
use bitcoin::opcodes::OP_0;
use bitcoin::Opcode;
use std::cmp::PartialEq;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitOpIfSuccess {
    YES,
    NO,
}

pub fn reduce(structure: &mut StructuredScript) -> EmitOpIfSuccess {
    match structure {
        StructuredScript::Script(v) => {
            // Find the first OP_IF_SUCCESS.
            // If it exists:
            // - slice the script with it
            // - create the second chunk of the script and reduce it
            // - return EmitOpIfSuccess::YES indicating

            let len = v.0.len();

            for i in 0..len {
                if v.0[i] == OwnedInstruction::Op(_OP_IF_SUCCESS) {
                    if i != len - 1 {
                        let existing_code = OwnedInstructions(v.0[0..i].to_vec());
                        let mut rest_code = v.0[i + 1..len].to_vec();
                        rest_code.push(OwnedInstruction::Op(OP_0));
                        let rest_code = OwnedInstructions(rest_code);

                        let mut new_if_else_statement = StructuredScript::IfElseEndIf(
                            Box::new(StructuredScript::Script(OwnedInstructions(vec![
                                OwnedInstruction::Op(OP_PUSHNUM_1),
                            ]))),
                            Box::new(StructuredScript::Script(rest_code)),
                        );

                        let emit_result = reduce(&mut new_if_else_statement);
                        if emit_result == EmitOpIfSuccess::YES {
                            *structure = StructuredScript::MultiScript(vec![
                                StructuredScript::Script(existing_code),
                                new_if_else_statement,
                                StructuredScript::IfEndIf(Box::new(StructuredScript::Script(
                                    OwnedInstructions(vec![OwnedInstruction::Op(OP_PUSHNUM_1)]),
                                ))),
                            ]);
                        } else {
                            *structure = StructuredScript::MultiScript(vec![
                                StructuredScript::Script(existing_code),
                                new_if_else_statement,
                            ]);
                        }
                    } else {
                        // remove the last OP_IF_SUCCESS and emit it to the upper layer
                        v.0.truncate(i);
                    }

                    return EmitOpIfSuccess::YES;
                }
            }

            EmitOpIfSuccess::NO
        }
        StructuredScript::MultiScript(vv) => {
            let len = vv.len();

            for i in 0..len {
                let emit_result = reduce(&mut vv[i]);
                if emit_result == EmitOpIfSuccess::YES {
                    if i == len - 1 {
                        // do nothing, emit it to the upper layer
                    } else {
                        // create a new If-Else statement
                        let mut rest_code = if i + 1 == len - 1 {
                            vv[i + 1].clone()
                        } else {
                            StructuredScript::MultiScript(vv[i + 1..len].to_vec())
                        };
                        append_opcode(&mut rest_code, OP_0);

                        vv.truncate(i + 1);
                        vv.push(StructuredScript::IfElseEndIf(
                            Box::new(StructuredScript::Script(OwnedInstructions(vec![
                                OwnedInstruction::Op(OP_PUSHNUM_1),
                            ]))),
                            Box::new(rest_code),
                        ));

                        let more_emit = reduce(&mut vv[i + 1]);
                        if more_emit == EmitOpIfSuccess::YES {
                            vv.push(StructuredScript::IfEndIf(Box::new(
                                StructuredScript::Script(OwnedInstructions(vec![
                                    OwnedInstruction::Op(OP_PUSHNUM_1),
                                ])),
                            )));
                        }
                    }

                    return EmitOpIfSuccess::YES;
                }
            }

            EmitOpIfSuccess::NO
        }
        StructuredScript::IfEndIf(v) => {
            let emit_result = reduce(v);

            if emit_result == EmitOpIfSuccess::YES {
                let new_if_else_statement = StructuredScript::IfElseEndIf(
                    v.clone(),
                    Box::new(StructuredScript::Script(OwnedInstructions(vec![
                        OwnedInstruction::Op(OP_0),
                    ]))),
                );
                *structure = new_if_else_statement;
            }

            emit_result
        }
        StructuredScript::NotIfEndIf(v) => {
            let emit_result = reduce(v);

            if emit_result == EmitOpIfSuccess::YES {
                let new_if_else_statement = StructuredScript::NotIfElseEndIf(
                    v.clone(),
                    Box::new(StructuredScript::Script(OwnedInstructions(vec![
                        OwnedInstruction::Op(OP_0),
                    ]))),
                );
                *structure = new_if_else_statement;
            }

            emit_result
        }
        StructuredScript::IfElseEndIf(v1, v2) | StructuredScript::NotIfElseEndIf(v1, v2) => {
            let emit_result_1 = reduce(v1);
            let emit_result_2 = reduce(v2);

            if emit_result_1 == emit_result_2 {
                emit_result_1
            } else {
                if emit_result_1 == EmitOpIfSuccess::YES {
                    append_opcode(v2, OP_0);
                } else {
                    append_opcode(v1, OP_0);
                }
                EmitOpIfSuccess::YES
            }
        }
    }
}

fn append_opcode(structure: &mut StructuredScript, opcode: Opcode) {
    match structure {
        StructuredScript::Script(v) => {
            v.0.push(OwnedInstruction::Op(opcode));
        }
        StructuredScript::MultiScript(vv) => {
            let len = vv.len();

            if let StructuredScript::Script(v) = &mut vv[len - 1] {
                v.0.push(OwnedInstruction::Op(opcode));
            } else if let StructuredScript::MultiScript(_) = &mut vv[len - 1] {
                append_opcode(&mut vv[len - 1], opcode);
            } else {
                vv.push(StructuredScript::Script(OwnedInstructions(vec![
                    OwnedInstruction::Op(opcode),
                ])));
            }
        }
        _ => {
            *structure = StructuredScript::MultiScript(vec![
                structure.clone(),
                StructuredScript::Script(OwnedInstructions(vec![OwnedInstruction::Op(opcode)])),
            ])
        }
    }
}

#[cfg(test)]
mod test {
    use crate::reduce::{reduce, EmitOpIfSuccess};
    use crate::structured_script::StructuredScript;
    use crate::OP_IF_SUCCESS;
    use bitcoin_script::{define_pushable, script};

    define_pushable!();

    #[test]
    fn test_reduce() {
        let test_script = script! {
            OP_NOP1
            OP_IF
                 OP_NOP2
                 OP_IF_SUCCESS
                 OP_NOP3
                 OP_IF_SUCCESS
                 OP_NOP4
            OP_ENDIF
            OP_NOP5
        };
        let mut script: StructuredScript = test_script.into();

        let res = reduce(&mut script);
        assert_eq!(res, EmitOpIfSuccess::YES);

        let expected_script = script! {
            OP_NOP1
            OP_IF
                 OP_NOP2
                 OP_IF
                      1
                      0
                 OP_ELSE
                    OP_NOP3
                    OP_IF
                       1
                    OP_ELSE
                       OP_NOP4
                       0
                       0
                    OP_ENDIF
                 OP_ENDIF
                 OP_IF 1 OP_ENDIF
            OP_ELSE
                 0
            OP_ENDIF
            OP_IF
                  1
            OP_ELSE
                  OP_NOP5
                  0
            OP_ENDIF
        };
        let expected: StructuredScript = expected_script.into();

        assert_eq!(expected, script);
    }
}
