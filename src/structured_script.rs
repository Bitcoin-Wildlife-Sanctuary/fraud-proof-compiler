use bitcoin::opcodes::all::{OP_ELSE, OP_ENDIF, OP_IF, OP_NOTIF, OP_PUSHBYTES_0};
use bitcoin::opcodes::Ordinary::{OP_PUSHDATA1, OP_PUSHDATA2};
use bitcoin::script::Instruction;
use bitcoin::{Opcode, ScriptBuf};
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum OwnedInstruction {
    Op(Opcode),
    PushBytes(Vec<u8>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct OwnedInstructions(pub Vec<OwnedInstruction>);

#[derive(Debug, PartialEq, Clone)]
pub enum StructuredScript {
    Script(OwnedInstructions),
    MultiScript(Vec<StructuredScript>),
    IfEndIf(Box<StructuredScript>),
    NotIfEndIf(Box<StructuredScript>),
    IfElseEndIf(Box<StructuredScript>, Box<StructuredScript>),
    NotIfElseEndIf(Box<StructuredScript>, Box<StructuredScript>),
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
                    if bytes.is_empty() {
                        instructions.push(OwnedInstruction::Op(OP_PUSHBYTES_0));
                    } else {
                        instructions.push(OwnedInstruction::PushBytes(bytes.as_bytes().to_vec()));
                    }
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

impl From<StructuredScript> for ScriptBuf {
    fn from(value: StructuredScript) -> Self {
        let mut script_buf = vec![];
        write_script_buf(&mut script_buf, &value);
        ScriptBuf::from(script_buf)
    }
}

fn write_script_buf(buf: &mut Vec<u8>, structure: &StructuredScript) {
    match structure {
        StructuredScript::Script(v) => {
            for inst in v.0.iter() {
                match inst {
                    OwnedInstruction::Op(op) => buf.push(op.to_u8()),
                    OwnedInstruction::PushBytes(v) => {
                        let len = v.len();
                        if len == 0 {
                            buf.push(OP_PUSHBYTES_0.to_u8());
                        } else if len <= 75 {
                            buf.push(len as u8);
                            buf.extend_from_slice(v);
                        } else {
                            if len <= 255 {
                                buf.push(OP_PUSHDATA1.to_u8());
                                buf.push(len as u8);
                                buf.extend_from_slice(v);
                            } else if len <= 65535 {
                                buf.push(OP_PUSHDATA2.to_u8());
                                buf.push((len & 0xff) as u8);
                                buf.push((len >> 8) as u8);
                                buf.extend_from_slice(v);
                            } else {
                                // one cannot push more than 520 bytes to the stack
                                unreachable!()
                            }
                        }
                    }
                }
            }
        }
        StructuredScript::MultiScript(vv) => {
            for v in vv.iter() {
                write_script_buf(buf, v);
            }
        }
        StructuredScript::IfEndIf(v) => {
            buf.push(OP_IF.to_u8());
            write_script_buf(buf, v);
            buf.push(OP_ENDIF.to_u8());
        }
        StructuredScript::NotIfEndIf(v) => {
            buf.push(OP_NOTIF.to_u8());
            write_script_buf(buf, v);
            buf.push(OP_ENDIF.to_u8());
        }
        StructuredScript::IfElseEndIf(v1, v2) => {
            buf.push(OP_IF.to_u8());
            write_script_buf(buf, v1);
            buf.push(OP_ELSE.to_u8());
            write_script_buf(buf, v2);
            buf.push(OP_ENDIF.to_u8());
        }
        StructuredScript::NotIfElseEndIf(v1, v2) => {
            buf.push(OP_NOTIF.to_u8());
            write_script_buf(buf, v1);
            buf.push(OP_ELSE.to_u8());
            write_script_buf(buf, v2);
            buf.push(OP_ENDIF.to_u8());
        }
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
