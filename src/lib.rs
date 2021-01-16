#![feature(assoc_char_funcs)]
#![feature(hash_raw_entry)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod entities;

mod stack;
mod errors;
pub mod parser;
pub mod words;

use std::{collections::HashMap};

use stack::Stack;
use entities::{simple::literal::Literal, complex::definition::WordElement};
use errors::ForthError::{self, StackUnderflow, InvalidOperands};

use pest::Parser;
use parser::*;

type Result<T> = std::result::Result<T, ForthError>;

macro_rules! ternary {
    ($c:expr, $v:expr, $v1:expr) => {
        if $c {$v} else {$v1}
    };
}

type WordFn = fn(&mut ForthInterpreter) -> Result<()>;

trait ExecuteExt {
	fn execute(&mut self, interpreter: &mut ForthInterpreter) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Variable { 
	name: String,
	value: Option<Literal>,
}

pub struct ForthInterpreter {
	stack: Stack<Literal>,
	
	variables: Vec<Variable>,
	constants: HashMap<String, Literal>, // No need in Option cause constant is initialized always

	native_words: HashMap<String, WordFn>,
	user_words: HashMap<String, WordElement>,
}

impl ForthInterpreter {
	pub fn new() -> Self {		
		Self {
			stack: Stack::new(),
			variables: Vec::new(),
			constants: HashMap::new(),

			native_words: [
				("+".into(), ForthInterpreter::add as WordFn), ("-".into(), ForthInterpreter::sub),
				("*".into(), ForthInterpreter::mul), ("/".into(), ForthInterpreter::div),
				("dup".into(), ForthInterpreter::dup), ("drop".into(), ForthInterpreter::drop),
				("swap".into(), ForthInterpreter::swap), ("over".into(), ForthInterpreter::over),
				("rot".into(), ForthInterpreter::rot), (".".into(), ForthInterpreter::print_top),
				("emit".into(), ForthInterpreter::emit), ("cr".into(), ForthInterpreter::cr),
				("=".into(), ForthInterpreter::equal), ("<".into(), ForthInterpreter::less_than),
				(">".into(), ForthInterpreter::greater_than), ("invert".into(), ForthInterpreter::invert),
				("and".into(), ForthInterpreter::and), ("or".into(), ForthInterpreter::or),
				("!".into(), ForthInterpreter::store_variable), ("@".into(), ForthInterpreter::fetch_variable),
			].iter().cloned().collect(),
			user_words: HashMap::<String, WordElement>::new(),
		}
	}
	
	fn get_unary_operand(&mut self) -> Result<Literal> {
		Ok(self.stack.pop().ok_or(StackUnderflow)?)
	}

	fn get_binary_operands(&mut self) -> Result<(Literal, Literal)> {
		let b: Literal = self.stack.pop().ok_or(StackUnderflow)?;
		let a: Literal = self.stack.pop().ok_or(StackUnderflow)?;
		Ok((a, b))
	}

	pub fn get_last_literal(&self) -> Result<&Literal> {
		Ok(self.stack.last().ok_or(StackUnderflow)?)
	}

	pub fn get_stack_dump(&self) -> &Stack<Literal> {
		&self.stack
	}

	pub fn get_vars_dump(&self) -> &Vec<Variable> {
		&self.variables
	}

	pub fn get_consts_dump(&self) -> &HashMap<String, Literal> {
		&self.constants
	}

	pub fn get_native_words_dump(&self) -> &HashMap<String, WordFn> {
		&self.native_words
	}

	pub fn get_user_words_dump(&self) -> &HashMap<String, WordElement> {
		&self.user_words
	}

	pub fn bool(literal: &Literal) -> bool {
		match &literal {
			&Literal::Integer(i) => {
				!(*i != -1i64)
			},
			Literal::String(_) => {
				true
			},
			_ => unreachable!()
		}
	}

	fn add(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		if let Literal::Integer(a) = a {
			if let Literal::Integer(b) = b {
				self.push(Literal::Integer(a + b));
				return Ok(())
			}
		}
		Err(InvalidOperands)
	}

	fn sub(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		if let Literal::Integer(a) = a {
			if let Literal::Integer(b) = b {
				self.push(Literal::Integer(a - b));
				return Ok(())
			}
		}
		Err(InvalidOperands)
	}

    fn mul(&mut self) -> Result<()> {
        let (a, b) = self.get_binary_operands()?;
        if let Literal::Integer(a) = a {
            if let Literal::Integer(b) = b {
                self.push(Literal::Integer(a * b));
                return Ok(()) 
            }
        }
        Err(InvalidOperands)
    }

	fn div(&mut self) -> Result<()> {
        let (a, b) = self.get_binary_operands()?;
        if let Literal::Integer(a) = a {
            if let Literal::Integer(b) = b {
                self.push(Literal::Integer(a / b));
                return Ok(()) 
            }
    	}
        Err(InvalidOperands)
    }

	fn dup(&mut self) -> Result<()> {
		self.push(self.get_last_literal()?.clone());
		Ok(())
	}

	fn drop(&mut self) -> Result<()> {
		self.stack.pop().ok_or(StackUnderflow)?;
		Ok(())
	}

	fn swap(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		self.push(b);
		self.push(a);
		Ok(())
	}

	fn over(&mut self) -> Result<()> {
		let length = self.stack.length();
		if length >= 2 {
			self.push((*self.stack.get(length - 2)).clone());
			return Ok(())
		}
		Err(StackUnderflow)
	}

	fn rot(&mut self) -> Result<()> {
		let length = self.stack.length();
		if length >= 3 {
			let element = self.stack.remove(length - 3);
			self.stack.push(element);
			return Ok(())
		}
		Err(StackUnderflow)
	}

	fn print_top(&mut self) -> Result<()> {
		print!("{}", *self.stack.last().ok_or(StackUnderflow)?);
		Ok(())
	}

	fn emit(&mut self) -> Result<()> {
		let last = self.stack.last().ok_or(StackUnderflow)?;
		if let &Literal::Integer(i) = last {
			print!("{}", char::from_u32(i as u32).ok_or(InvalidOperands)?);
		}
		Ok(())
	}

	fn cr(&mut self) -> Result<()> {
		print!("\n");
		Ok(())
	}

	fn equal(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		self.stack.push(Literal::Integer(ternary!(a == b, -1, 0)));
		Ok(())
	}

	fn less_than(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		self.stack.push(Literal::Integer(ternary!(a < b, -1, 0)));
		Ok(())
	}

	fn greater_than(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		self.stack.push(Literal::Integer(ternary!(a > b, -1, 0)));
		Ok(())
	}

	fn and(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		self.stack.push(Literal::Integer(ternary!(a != 0.into() && b != 0.into(), -1, 0)));
		Ok(())
	}

	fn or(&mut self) -> Result<()> {
		let (a, b) = self.get_binary_operands()?;
		self.stack.push(Literal::Integer(ternary!(a != 0.into() || b != 0.into(), -1, 0)));
		Ok(())
	}
	
	fn invert(&mut self) -> Result<()> {
		let a = self.stack.pop().ok_or(StackUnderflow)?;
		self.stack.push(Literal::Integer(ternary!(a == 0.into(), -1, 0)));
		Ok(())
	}

	fn variable(&mut self) -> Result<()> {
		let var_name = self.stack.pop().ok_or(StackUnderflow)?;
		self.variables.push(Variable { name: var_name.to_string(), value: None});
		Ok(())
	}

	fn set_variable(&mut self, name: String, value: Literal) -> Result<()> {
		let variable = self.variables.iter_mut().find(|var| var.name == name);
		match variable {
			None => {
				self.variables.push(
					Variable {
						name: name.clone(),
						value: Some(value)
					}
				)
			}
			Some(e) => {
				e.value = Some(value);
			}
		}
		Ok(())
	}

	fn contains_variable(&self, name: &String) -> bool {
		if let None = self.variables.iter().find(|var| &var.name == name) {
			false
		} else {
			true
		}
	}

	fn get_variable_id(&self, name: &String) -> Option<usize> {
		self.variables.iter().position(|var| &var.name == name)
	}
	
	fn store_variable(&mut self) -> Result<()> {
		let (var_value, var_index) = self.get_binary_operands()?;
		if let Literal::Pointer(idx) = var_index {
			self.variables[idx].value = Some(var_value);
		}
		Ok(())
	}

	fn fetch_variable(&mut self) -> Result<()> {
		let var_index = self.get_unary_operand()?;
		if let Literal::Pointer(idx) = var_index {
			self.push(self.variables[idx].value.as_ref().unwrap_or(&0i64.into()).clone());
		}
		Ok(())
	}

	pub fn execute_line(&mut self, line: &str) -> Result<()> {
		let line_pair = ForthParser::parse(Rule::line, line).unwrap();
		let mut line = entities::Line::parse(line_pair.peek().unwrap());

		line.execute(self)?;

		Ok(())
	}

	fn push(&mut self, value: Literal) {
		self.stack.push(value);
	}
}

#[cfg(test)]
mod tests {
	use crate::ForthInterpreter;
	use crate::Literal;

	#[test]
	fn test_interpreter() {
		let mut forth: ForthInterpreter = ForthInterpreter::new();
		forth.push(Literal::Integer(5));
		forth.push(Literal::Integer(5));
	}

	#[test]
	fn test_parsing() {
		
		let mut forth: ForthInterpreter = ForthInterpreter::new();
		//forth.execute_line("a b +")
	}

	#[test]
	fn test_variable() {
		let mut interpreter = ForthInterpreter::new();
    
		interpreter.execute_line("variable user_var").unwrap();
		interpreter.execute_line("123 user_var !").unwrap();

		let value = interpreter.get_last_literal().unwrap();
		if let Literal::Integer(i) = value {
			println!("{:?}", unsafe { (*i as *const Option<Literal>).as_ref() });
		}

	}

}
