use crate::structured_script::StructuredScript;
use bitcoin::ScriptBuf;
use bitcoin_script::{define_pushable, script};

pub fn final_emit_code() -> ScriptBuf {
    define_pushable!();

    script! {
        OP_IF
            OP_DEPTH 512 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..256 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 256 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..128 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 128 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..64 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 64 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..32 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 32 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..16 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 16 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..8 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 8 OP_GREATERTHANOREQUAL OP_IF
                for _ in 0..4 {
                    OP_2DROP
                }
            OP_ENDIF

            OP_DEPTH 4 OP_GREATERTHANOREQUAL OP_IF
                OP_2DROP OP_2DROP
            OP_ENDIF

            OP_DEPTH 2 OP_GREATERTHANOREQUAL OP_IF
                OP_2DROP
            OP_ENDIF

            OP_DEPTH OP_IF
                OP_DROP
            OP_ENDIF

            OP_TRUE
        OP_ENDIF
    }
}

pub fn append_final_emit_script(structure: &mut StructuredScript) {
    let final_emit_code: StructuredScript = final_emit_code().into();

    match structure {
        StructuredScript::MultiScript(vv) => {
            vv.push(final_emit_code);
        }
        _ => {
            *structure = StructuredScript::MultiScript(vec![structure.clone(), final_emit_code]);
        }
    }
}
