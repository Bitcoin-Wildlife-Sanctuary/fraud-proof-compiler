use bitcoin::opcodes::all::{OP_ELSE, OP_ENDIF, OP_IF, OP_NOTIF, OP_RETURN_199, OP_RETURN_200};
use bitcoin::script::Instruction;
use bitcoin::{Opcode, ScriptBuf};
use std::cmp::PartialEq;
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum OwnedInstruction {
    Op(Opcode),
    PushBytes(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub struct OwnedInstructions(pub Vec<OwnedInstruction>);

#[derive(Debug, PartialEq)]
pub enum StructuredScript {
    Script(OwnedInstructions),
    MultiScript(Vec<StructuredScript>),
    IfEndIf(Box<StructuredScript>),
    NotIfEndIf(Box<StructuredScript>),
    IfElseEndIf(Box<StructuredScript>, Box<StructuredScript>),
    NotIfElseEndIf(Box<StructuredScript>, Box<StructuredScript>),
}

#[allow(non_snake_case)]
pub fn OP_SUCCESS() -> ScriptBuf {
    ScriptBuf::from(vec![OP_RETURN_199.to_u8()])
}

#[allow(non_snake_case)]
pub fn OP_IF_SUCCESS() -> ScriptBuf {
    ScriptBuf::from(vec![OP_RETURN_200.to_u8()])
}

impl From<ScriptBuf> for OwnedInstructions {
    fn from(value: ScriptBuf) -> Self {
        let iter = value.instructions();
        let mut instructions = vec![];
        for inst in iter {
            let inst = inst.unwrap();
            match inst {
                Instruction::Op(opcode) => instructions.push(OwnedInstruction::Op(opcode)),
                Instruction::PushBytes(bytes) => {
                    instructions.push(OwnedInstruction::PushBytes(bytes.as_bytes().to_vec()))
                }
            }
        }
        OwnedInstructions(instructions)
    }
}

impl From<OwnedInstructions> for StructuredScript {
    fn from(value: OwnedInstructions) -> Self {
        let mut iter = value.0.iter().peekable();
        create_structured_script(&mut iter)
    }
}

impl From<ScriptBuf> for StructuredScript {
    fn from(value: ScriptBuf) -> Self {
        let owned_instructions: OwnedInstructions = value.into();
        owned_instructions.into()
    }
}

fn create_structured_script(
    iter: &mut Peekable<core::slice::Iter<OwnedInstruction>>,
) -> StructuredScript {
    let mut cur = vec![];
    let mut all = vec![];

    while iter.peek().is_some() {
        let next_instruction = iter.peek().unwrap();
        if **next_instruction == OwnedInstruction::Op(OP_IF) {
            iter.next().unwrap();
            if !cur.is_empty() {
                all.push(StructuredScript::Script(OwnedInstructions(cur.clone())));
                cur.clear();
            }

            let if_branch = create_structured_script(iter);

            // there must be a next instruction
            let next_instruction = iter.next().unwrap();
            if *next_instruction == OwnedInstruction::Op(OP_ELSE) {
                let else_branch = create_structured_script(iter);

                // there must be a next instruction
                let next_instruction = iter.next().unwrap();
                assert_eq!(*next_instruction, OwnedInstruction::Op(OP_ENDIF));

                all.push(StructuredScript::IfElseEndIf(
                    Box::new(if_branch),
                    Box::new(else_branch),
                ));
            } else if *next_instruction == OwnedInstruction::Op(OP_ENDIF) {
                all.push(StructuredScript::IfEndIf(Box::new(if_branch)));
            } else {
                panic!("An if branch does not seem to end correctly.");
            }
        } else if **next_instruction == OwnedInstruction::Op(OP_NOTIF) {
            iter.next().unwrap();
            if !cur.is_empty() {
                all.push(StructuredScript::Script(OwnedInstructions(cur.clone())));
                cur.clear();
            }

            let not_if_branch = create_structured_script(iter);

            // there must be a next instruction
            let next_instruction = iter.next().unwrap();
            if *next_instruction == OwnedInstruction::Op(OP_ELSE) {
                let else_branch = create_structured_script(iter);

                // there must be a next instruction
                let next_instruction = iter.next().unwrap();
                assert_eq!(*next_instruction, OwnedInstruction::Op(OP_ENDIF));

                all.push(StructuredScript::NotIfElseEndIf(
                    Box::new(not_if_branch),
                    Box::new(else_branch),
                ));
            } else if *next_instruction == OwnedInstruction::Op(OP_ENDIF) {
                all.push(StructuredScript::NotIfEndIf(Box::new(not_if_branch)));
            } else {
                panic!("An not-if branch does not seem to end correctly.");
            }
        } else if **next_instruction == OwnedInstruction::Op(OP_ELSE)
            || **next_instruction == OwnedInstruction::Op(OP_ENDIF)
        {
            if !cur.is_empty() {
                all.push(StructuredScript::Script(OwnedInstructions(cur.clone())));
                cur.clear();
            }

            return if all.len() == 1 {
                all.pop().unwrap()
            } else {
                StructuredScript::MultiScript(all)
            };
        } else {
            cur.push((*next_instruction).clone());
            iter.next().unwrap();
        }
    }

    if !cur.is_empty() {
        all.push(StructuredScript::Script(OwnedInstructions(cur.clone())));
        cur.clear();
    }

    if all.len() == 1 {
        all.pop().unwrap()
    } else {
        StructuredScript::MultiScript(all)
    }
}

#[cfg(test)]
mod test {
    use crate::structured_script::{OwnedInstruction, OwnedInstructions, StructuredScript};
    use bitcoin::opcodes::all::{OP_NOP1, OP_NOP4};
    use bitcoin::opcodes::{OP_NOP2, OP_NOP3};
    use bitcoin_script::{define_pushable, script};

    define_pushable!();

    #[test]
    fn test_create_structured_script() {
        let script = script! {
            OP_NOP1
            OP_IF
                OP_NOP2
                { 123456 }
                OP_IF
                    { 456789 }
                    OP_NOTIF
                        { 5678 }
                        OP_NOP3
                    OP_ENDIF
                OP_ELSE
                    OP_NOTIF
                        { 1011 }
                    OP_ELSE
                        { 1213 }
                    OP_ENDIF
                    { 1234 }
                OP_ENDIF
            OP_ENDIF
            OP_NOP4
        };

        let structued_script = StructuredScript::from(script);

        let expected = StructuredScript::MultiScript(vec![
            StructuredScript::Script(OwnedInstructions(vec![OwnedInstruction::Op(OP_NOP1)])),
            StructuredScript::IfEndIf(Box::new(StructuredScript::MultiScript(vec![
                StructuredScript::Script(OwnedInstructions(vec![
                    OwnedInstruction::Op(OP_NOP2),
                    OwnedInstruction::PushBytes(123456u32.to_le_bytes()[0..3].to_vec()),
                ])),
                StructuredScript::IfElseEndIf(
                    Box::new(StructuredScript::MultiScript(vec![
                        StructuredScript::Script(OwnedInstructions(vec![
                            OwnedInstruction::PushBytes(456789u32.to_le_bytes()[0..3].to_vec()),
                        ])),
                        StructuredScript::NotIfEndIf(Box::new(StructuredScript::Script(
                            OwnedInstructions(vec![
                                OwnedInstruction::PushBytes(5678u32.to_le_bytes()[0..2].to_vec()),
                                OwnedInstruction::Op(OP_NOP3),
                            ]),
                        ))),
                    ])),
                    Box::new(StructuredScript::MultiScript(vec![
                        StructuredScript::NotIfElseEndIf(
                            Box::new(StructuredScript::Script(OwnedInstructions(vec![
                                OwnedInstruction::PushBytes(1011u32.to_le_bytes()[0..2].to_vec()),
                            ]))),
                            Box::new(StructuredScript::Script(OwnedInstructions(vec![
                                OwnedInstruction::PushBytes(1213u32.to_le_bytes()[0..2].to_vec()),
                            ]))),
                        ),
                        StructuredScript::Script(OwnedInstructions(vec![
                            OwnedInstruction::PushBytes(1234u32.to_le_bytes()[0..2].to_vec()),
                        ])),
                    ])),
                ),
            ]))),
            StructuredScript::Script(OwnedInstructions(vec![OwnedInstruction::Op(OP_NOP4)])),
        ]);

        assert_eq!(expected, structued_script);
    }
}
