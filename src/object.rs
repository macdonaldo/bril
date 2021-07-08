use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub name: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub args: Vec<Arg>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub return_type: Option<Type>,

    #[serde(default)]
    pub instrs: Vec<Code>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Arg {
    pub name: String,

    #[serde(rename = "type")]
    pub arg_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
    Ptr(Box<Type>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Code {
    Label { label: String },
    Instruction(Instruction),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Instruction {
    Constant {
        op: ConstOps,
        dest: String,

        #[serde(rename = "type")]
        dest_type: Type,
        value: Literal,
    },
    Value {
        op: ValueOps,
        dest: Option<String>, // Call may (optionally) produce a result

        #[serde(rename = "type")]
        dest_type: Option<Type>, // Call may (optionally) produce a result

        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        args: Vec<String>,

        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        funcs: Vec<String>,

        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        labels: Vec<String>,
    },
    Effect {
        op: EffectOps,

        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        args: Vec<String>,

        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        funcs: Vec<String>,

        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        labels: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Literal {
    Bool(bool),
    Int(i64),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ConstOps {
    Const,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ValueOps {
    // Arithmetic
    Add,
    Mul,
    Sub,
    Div,
    // Comparison
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
    // Logic
    Not,
    And,
    Or,
    // Control
    Call, // The call instruction can be a Value Operation or an Effect Operation
    // Misc.
    Id,
    // Memory
    Alloc,
    Load,
    PtrAdd,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EffectOps {
    // Control
    Jmp,
    Br,
    Ret,
    // Misc.
    Print,
    Nop,
    // Memory
    Free,
    Store,
}

#[derive(Debug, Clone)]
pub struct BasicBlock<'a> {
    pub label: String,
    pub instrs: Vec<&'a Code>,
}

impl Function {
    pub fn get_basic_blocks(&self) -> Vec<BasicBlock> {
        let mut basic_blocks = vec![];

        let label = String::from("entry");
        let mut bb = BasicBlock {
            label,
            instrs: vec![],
        };

        for i in 0..self.instrs.len() {
            let current = &self.instrs[i];

            if current.is_label() {
                bb.label = current.get_label();
            } else {
                bb.instrs.push(current);

                // this is the last basic block or the next one has a label
                if i + 1 == self.instrs.len() || self.instrs[i + 1].is_label() {
                    basic_blocks.push(bb.clone());
                    bb.instrs.clear();
                } else {
                    if current.is_terminator() {
                        basic_blocks.push(bb.clone());
                        bb.instrs.clear();

                        let label = format!("{}_bb{}", &self.name, basic_blocks.len());
                        bb.label = label;
                    }
                }
            }
        }

        basic_blocks
    }

    pub fn get_successors<'a>(&self, basic_blocks: &'a Vec<BasicBlock>) -> HashMap<&'a str, Vec<&'a str>> {
        let mut successors = HashMap::new();

        for i in 0..basic_blocks.len() {
            let bb = &basic_blocks[i];

            let last = &bb.instrs.len() - 1;
            let current_label: &str = bb.label.as_ref();

            if bb.instrs[last].is_terminator() {
                match bb.instrs[last] {
                    Code::Instruction(Instruction::Effect { labels, .. }) => {
                        let referenced_labels: Vec<&str> =
                            labels.iter().map(AsRef::as_ref).collect();
                        successors.insert(current_label, referenced_labels);
                    }
                    _ => (),
                };
            } else if i + 1 < basic_blocks.len() {
                let next_label = vec![basic_blocks[i + 1].label.as_ref()];
                successors.insert(current_label, next_label);
            }
        }
        successors
    }

    pub fn cfg_dot(&self, basic_block: &Vec<BasicBlock>, successors: &HashMap<&str, Vec<&str>>) {
        println!("digraph {} {{", self.name);

        for entry in basic_block {
            println!("\t{};", entry.label.replace(".", "_"));
        }

        for entry in basic_block {
            if let Some(list) = successors.get(entry.label.as_str()) {
                for succ in list {
                    println!("\t{} -> {};", entry.label.replace(".", "_"), succ.replace(".", "_"));
                }
            }
        }

        println!("}}");
    }
}

impl Code {
    pub fn is_label(&self) -> bool {
        match &self {
            Code::Label { .. } => true,
            _ => false,
        }
    }

    pub fn get_label(&self) -> String {
        match &self {
            Code::Label { label } => label.clone(),
            _ => String::new(),
        }
    }

    pub fn is_terminator(&self) -> bool {
        match &self {
            Code::Instruction(Instruction::Effect { op, .. }) => match op {
                EffectOps::Br | EffectOps::Jmp => true,
                _ => false,
            },
            _ => false,
        }
    }
}

pub fn print_basic_blocks(program: &Program) -> Result<()> {
    for function in program.functions.iter() {
        let basic_block = function.get_basic_blocks();
        println!("Function: {}", function.name);
        for bb in basic_block.iter() {
            println!("Basic Block: {}", bb.label);
            for instr in &bb.instrs {
                println!("{}", serde_json::to_string_pretty(instr)?);
            }
        }
    }
    Ok(())
}
