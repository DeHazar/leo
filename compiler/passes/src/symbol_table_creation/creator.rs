// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use leo_ast::*;
use leo_errors::emitter::Handler;
use leo_span::Symbol;

use crate::{SymbolTable, VariableSymbol, VariableType};

/// A compiler pass during which the `SymbolTable` is created.
/// Note that this pass only creates the initial entries for functions, structs, and records.
/// The table is populated further during the type checking pass.
pub struct SymbolTableCreator<'a> {
    /// The `SymbolTable` constructed by this compiler pass.
    pub(crate) symbol_table: SymbolTable,
    /// The error handler.
    handler: &'a Handler,
    /// The current program name.
    program_name: Option<Symbol>,
}

impl<'a> SymbolTableCreator<'a> {
    pub fn new(handler: &'a Handler) -> Self {
        Self { symbol_table: Default::default(), handler, program_name: None }
    }
}

impl<'a> ExpressionVisitor<'a> for SymbolTableCreator<'a> {
    type AdditionalInput = ();
    type Output = ();
}

impl<'a> StatementVisitor<'a> for SymbolTableCreator<'a> {}

impl<'a> ProgramVisitor<'a> for SymbolTableCreator<'a> {
    fn visit_program_scope(&mut self, input: &'a ProgramScope) {
        // Set current program name
        self.program_name = Some(input.program_id.name.name);

        // Visit the program scope
        input.structs.iter().for_each(|(_, c)| (self.visit_struct(c)));
        input.mappings.iter().for_each(|(_, c)| (self.visit_mapping(c)));
        input.functions.iter().for_each(|(_, c)| (self.visit_function(c)));
        input.consts.iter().for_each(|(_, c)| (self.visit_const(c)));
    }

    fn visit_import(&mut self, input: &'a Program) {
        self.visit_program(input)
    }

    fn visit_struct(&mut self, input: &'a Composite) {
        if let Err(err) = self.symbol_table.insert_struct(self.program_name.unwrap(), input.name(), input) {
            self.handler.emit_err(err);
        }
    }

    fn visit_mapping(&mut self, input: &'a Mapping) {
        // Add the variable associated with the mapping to the symbol table.
        if let Err(err) = self.symbol_table.insert_variable(input.identifier.name, VariableSymbol {
            type_: Type::Mapping(MappingType {
                key: Box::new(input.key_type.clone()),
                value: Box::new(input.value_type.clone()),
            }),
            span: input.span,
            declaration: VariableType::Mut,
        }) {
            self.handler.emit_err(err);
        }
    }

    fn visit_function(&mut self, input: &'a Function) {
        if let Err(err) = self.symbol_table.insert_fn(self.program_name.unwrap(), input.name(), input) {
            self.handler.emit_err(err);
        }
    }

    fn visit_stub(&mut self, input: &'a Stub) {
        self.program_name = Some(input.stub_id.name.name);
        input.functions.iter().for_each(|(_, c)| (self.visit_function_stub(c)));
        input.structs.iter().for_each(|(_, c)| (self.visit_struct_stub(c)));
    }

    fn visit_function_stub(&mut self, input: &'a FunctionStub) {
        if let Err(err) =
            self.symbol_table.insert_fn(self.program_name.unwrap(), input.name(), &Function::from(input.clone()))
        {
            self.handler.emit_err(err);
        }
    }

    fn visit_struct_stub(&mut self, input: &'a Composite) {
        if let Err(err) = self.symbol_table.insert_struct(self.program_name.unwrap(), input.name(), input) {
            self.handler.emit_err(err);
        }
    }
}
