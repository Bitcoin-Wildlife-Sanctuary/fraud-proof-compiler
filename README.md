## Fraud proof compiler

This repository implements a toolkit that helps people write fraud proof logic in Bitcoin script, particularly focusing 
on providing two "pseudo" opcodes:

- `OP_SUCCESS`: mark the transaction as successful when being executed
- `OP_IF_SUCCESS`: equivalent to `OP_IF OP_SUCCESS OP_ENDIF`

Previously, in TapScript, we already have a number of OP_SUCCESSXX opcodes, as defined in 
[BIP-342](https://en.bitcoin.it/wiki/BIP_0342). 

> If any opcode numbered 80, 98, 126-129, 131-134, 137-138, 141-142, 149-153, 187-254 is encountered, validation succeeds (none of the rules below apply). This is true even if later bytes in the tapscript would fail to decode otherwise. These opcodes are renamed to OP_SUCCESS80, ..., OP_SUCCESS254, and collectively known as OP_SUCCESSx.

The problem is that these `OP_SUCCESSx` opcodes are way too powerful. The mere existence of them in the script will declare the script 
execution to be successful, even if the rest of the script is malformed (such as an `OP_IF` without the corresponding `OP_ENDIF`). 
Also, the `OP_SUCCESSx` opcodes are not intended to be used. Transactions that include them would be deemed nonstandard, 
and they are designed as placeholders while preventing people from using these opcodes before they are assigned with a 
different meaning.

### Seeking for a weaker OP_SUCCESSx

It has been previously proposed to have a weaker version of `OP_SUCCESS`, by Rusty Russell in a 2023 article, 
["Covenants: Examining ScriptPubkeys in Bitcoin Script"](https://rusty.ozlabs.org/2023/10/20/examining-scriptpubkey-in-script.html), 
the idea of which is an OP_SUCCESS that only declares the script execution to be successful if it is executed. This would,
of course, requires a consensus change, as we need to add a new opcode.

This repository is to provide a way that does not require adding new opcodes but is able to emulate such a weaker version of 
`OP_SUCCESSx`, as we discussed above.

### Rewriting

In summary, the compiler will perform the following in order to rewrite the code.

- Start by having all the codes in `OP_SUCCESS`. If there is any code in the same branch after `OP_SUCCESS`, the code would not be executed, and can be removed.
- Convert all `OP_SUCCESS` into the representation with `OP_IF_SUCCESS`.
- Iterate the following steps:
  * if we can find the deepest “if” or “else” branch with `OP_IF_SUCCESS` and the first `OP_IF_SUCCESS` in it is not at the end (first priority):
    - apply the rule to push `OP_IF_SUCCESS` to the end of this very branch, during which a deeper if-else statement might be created
  * else, if we can find the deepest if-else statement which has `OP_IF_SUCCESS` but all of them are at the end of the if branch or the else branch (second priority):
    - adjust both branches to have an `OP_IF_SUCCESS` at the end, remove the `OP_IF_SUCCESS` at the end, and add `OP_IF_SUCCESS` to the end of the if-else statement
    - If two `OP_IF_SUCCESS` come next to each other as a result here, the two `OP_IF_SUCCESS` would be replaced by `OP_IF 1 OP_ENDIF OP_IF_SUCCESS`
  * else, if there is a top-level `OP_IF_SUCCESS` but it is not the last instruction of the script (third priority):
    - apply the rule to push it to the end of the program, during which an if-else statement would be created
  * else, if there is a top-level OP_IF_SUCCESS and this is the last opcode of the script (fourth priority):
    - convert it into `OP_IF <Success Logic> OP_ENDIF`

The success logic is as follows:

```
OP_DEPTH 512 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (256 OP_2DROP)
OP_ENDIF
OP_DEPTH 256 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (128 OP_2DROP)
OP_ENDIF
OP_DEPTH 128 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (64 OP_2DROP)
OP_ENDIF
OP_DEPTH 64 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (32 OP_2DROP)
OP_ENDIF
OP_DEPTH 32 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (16 OP_2DROP)
OP_ENDIF
OP_DEPTH 16 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (8 OP_2DROP)
OP_ENDIF
OP_DEPTH 8 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (4 OP_2DROP)
OP_ENDIF
OP_DEPTH 4 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP ... OP_2DROP (2 OP_2DROP)
OP_ENDIF
OP_DEPTH 2 OP_GREATERTHANOREQUAL OP_IF
    OP_2DROP
OP_ENDIF
OP_DEPTH OP_IF
    OP_DROP
OP_ENDIF
OP_TRUE
```