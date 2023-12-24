use super::{Statement, Expression, Member, Literal, StructField, UnionMember, AccessModifier, UnionMemberField, FieldInitializer, UnaryOperator, BinaryOperator, Parameter, ConditionBlock};

pub struct Indent {
    level: usize,
}

impl Indent {
    pub fn new() -> Indent {
        Indent {
            level: 0,
        }
    }

    fn increase(&mut self) {
        self.level += 1;
    }

    fn decrease(&mut self) {
        self.level -= 1;
    }

    fn dash(&self) -> String {
        let mut result = String::new();
        for _ in 0..self.level - 1 {
            result.push_str("┆ ");
        }
        result.push_str("├─");
        result
    }

    fn dash_end(&self) -> String {
        let mut result = String::new();
        for _ in 0..self.level - 1 {
            result.push_str("┆");
        }
        result.push_str("╰─");
        result
    }
}

pub trait IndentDisplay {
    fn indent_display(&self, indent: &mut Indent) -> String;
}

impl IndentDisplay for Statement {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Statement::Program { statements } => {
                let mut result = String::new();
                for statement in statements {
                    result.push_str(&statement.indent_display(indent));
                }
                result
            },
            Statement::StructDeclaration {
                access_modifier,
                type_name,
                fields 
            } => {
                let mut result = String::new();
                result.push_str(format!("<struct declaration> {}\n", type_name).as_str());
                indent.increase();
                if let Some(access_modifier) = access_modifier {
                    result.push_str(format!("{}access_modifier: {}", indent.dash(), access_modifier.indent_display(indent)).as_str());
                } else {
                    result.push_str(format!("{}access_modifier: None", indent.dash()).as_str());
                }
                for field in fields {
                    result.push_str(format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str());
                }
                indent.decrease();
                result
            },
            Statement::UnionDeclaration {
                access_modifier,
                type_name,
                fields: members
            } => {
                let mut result = String::new();
                result.push_str(format!("<union declaration> {}\n", type_name).as_str());
                indent.increase();
                if let Some(access_modifier) = access_modifier {
                    result.push_str(format!("{}access_modifier: {}\n", indent.dash(), access_modifier.indent_display(indent)).as_str());
                } else {
                    result.push_str(format!("{}access_modifier: None", indent.dash()).as_str());
                }
                for member in members {
                    result.push_str(format!("\n{}{}", indent.dash_end(), member.indent_display(indent)).as_str());
                }
                indent.decrease();
                result
            },
            Statement::FunctionDeclaration {
                access_modifier,
                identifier,
                parameters,
                return_type,
                body
            } => {
                let mut result = String::new();
                result.push_str(format!("<function declaration> {}\n", identifier).as_str());
                indent.increase();
                if let Some(access_modifier) = access_modifier {
                    result.push_str(format!("{}access_modifier: {}", indent.dash(), access_modifier.indent_display(indent)).as_str());
                } else {
                    result.push_str(format!("{}access_modifier: None", indent.dash()).as_str());
                }
                for parameter in parameters {
                    result.push_str(format!("\n{}{}\n", indent.dash(), parameter.indent_display(indent)).as_str());
                }
                if let Some(return_type) = return_type {
                    result.push_str(format!("{}return_type: {}\n", indent.dash(), return_type).as_str());
                } else {
                    result.push_str(format!("{}return_type: None\n", indent.dash()).as_str());
                }
                result.push_str(format!("{}body: {}", indent.dash_end(), body.indent_display(indent)).as_str());
                indent.decrease();
                result
            },
            Statement::Expression(e) => e.indent_display(indent),
        }
    }
}

impl IndentDisplay for Expression {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Expression::None => {
                String::new()
            },
            Expression::VariableDeclaration {
                mutable,
                type_name,
                identifier,
                initializer
            } => {
                let mut result = String::new();
                result.push_str(format!("<variable declaration>{}\n", identifier).as_str());
                indent.increase();
                result.push_str(format!("{}mutable: {}\n", indent.dash(), mutable).as_str());
                result.push_str(format!("{}type_name: {}\n", indent.dash(), type_name).as_str());
                if let Some(initializer) = initializer {
                    result.push_str(format!("{}initializer: {}", indent.dash_end(), initializer.indent_display(indent)).as_str());
                } else {
                    result.push_str(format!("{}initializer: None", indent.dash_end()).as_str());
                }
                indent.decrease();
                result
            },
            Expression::If {
                r#if,
                else_ifs,
                r#else
            } => {
                let mut result = String::new();
                result.push_str("<if>\n");
                indent.increase();
                result.push_str(format!("{}condition:{}", indent.dash(), r#if.condition.indent_display(indent)).as_str());
                if let Some(else_ifs) = else_ifs {
                    for else_if in else_ifs {
                        result.push_str(format!("\n{}{}\n", indent.dash(), else_if.indent_display(indent)).as_str());
                        result.push_str(format!("\n{}condition: {}\n", indent.dash(), else_if.condition.indent_display(indent)).as_str());
                    }
                } else {
                    result.push_str(format!("\n{}else_ifs: None\n", indent.dash()).as_str());
                }
                if let Some(r#else) = r#else {
                    result.push_str(format!("{}else block: {}", indent.dash_end(), r#else.indent_display(indent)).as_str());
                } else {
                    result.push_str(format!("{}else block: None", indent.dash_end()).as_str());
                }
                indent.decrease();
                result
            },
            Expression::Assignment {
                member,
                initializer
            } => {
                let mut result = String::new();
                result.push_str("<assignment>\n");
                indent.increase();
                result.push_str(format!("{}member: {}\n", indent.dash(), member.indent_display(indent)).as_str());
                result.push_str(format!("{}initializer: {}", indent.dash_end(), initializer.indent_display(indent)).as_str());
                indent.decrease();
                result
            },
            Expression::Member(m) => m.indent_display(indent),
            Expression::Literal(l) => l.indent_display(indent),
            Expression::Call {
                caller,
                arguments
            } => {
                let mut result = String::new();
                result.push_str("<call>\n");
                indent.increase();
                result.push_str(format!("{}caller: {}", indent.dash(), caller.indent_display(indent)).as_str());
                let mut i = 0;
                for argument in arguments {
                    if i < arguments.len() - 1 {
                        result.push_str(format!("\n{}argument: {},", indent.dash(), argument.indent_display(indent)).as_str());
                    } else {
                        result.push_str(format!("\n{}argument: {}", indent.dash_end(), argument.indent_display(indent)).as_str());
                    }
                    i += 1;
                }
                indent.decrease();
                result
            },
            Expression::Unary {
                operator,
                expression
            } => {
                let mut result = String::new();
                result.push_str("<unary>\n");
                indent.increase();
                result.push_str(format!("{}operator: {}\n", indent.dash(), operator.indent_display(indent)).as_str());
                result.push_str(format!("{}expression: {}", indent.dash_end(), expression.indent_display(indent)).as_str());
                indent.decrease();
                result
            },
            Expression::Binary {
                left,
                operator,
                right
            } => {
                let mut result = String::new();
                result.push_str("<binary>\n");
                indent.increase();
                result.push_str(format!("{}left: {}\n", indent.dash(), left.indent_display(indent)).as_str());
                result.push_str(format!("{}operator: {}\n", indent.dash(), operator.indent_display(indent)).as_str());
                result.push_str(format!("{}right: {}", indent.dash_end(), right.indent_display(indent)).as_str());
                indent.decrease();
                result
            },
            Expression::Ternary {
                condition,
                true_expression,
                false_expression
            } => {
                let mut result = String::new();
                result.push_str("<ternary>\n");
                indent.increase();
                result.push_str(format!("{}condition: {}\n", indent.dash(), condition.indent_display(indent)).as_str());
                result.push_str(format!("{}true_expression: {}\n", indent.dash(), true_expression.indent_display(indent)).as_str());
                result.push_str(format!("{}false_expression: {}", indent.dash_end(), false_expression.indent_display(indent)).as_str());
                indent.decrease();
                result
            },
            Expression::Block {
                statements
            } => {
                let mut result = String::new();
                result.push_str("<block>");
                indent.increase();
                let mut i = 0;
                for statement in statements {
                    if i < statements.len() - 1 {
                        result.push_str(format!("\n{}{},", indent.dash(), statement.indent_display(indent)).as_str());
                    } else {
                        result.push_str(format!("\n{}{}", indent.dash_end(), statement.indent_display(indent)).as_str());
                    }
                    i += 1;
                }
                indent.decrease();
                result
            },
            Expression::Drop {
                identifier
            } => {
                let mut result = String::new();
                result.push_str(format!("<drop> {}", identifier).as_str());
                result
            },
        }
    }
}

impl IndentDisplay for Member {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Member::Identifier {
                symbol
            } => {
                let mut result = String::new();
                result.push_str(format!("<identifier> {}", symbol).as_str());
                result
            },
            Member::MemberAccess {
                object,
                member,
                symbol
            } => {
                let mut result = String::new();
                result.push_str("<member access>");
                indent.increase();
                result.push_str(format!("{}object: {}\n", indent.dash(), object.indent_display(indent)).as_str());
                result.push_str(format!("{}member: {}\n", indent.dash(), member.indent_display(indent)).as_str());
                result.push_str(format!("{}symbol: {}", indent.dash_end(), symbol).as_str());
                indent.decrease();
                result
            },
        }
    }
}

impl IndentDisplay for Literal {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Literal::Int8(v) => v.to_string(),
            Literal::Int16(v) => v.to_string(),
            Literal::Int32(v) => v.to_string(),
            Literal::Int64(v) => v.to_string(),
            Literal::Int128(v) => v.to_string(),
            Literal::UInt8(v) => v.to_string(),
            Literal::UInt16(v) => v.to_string(),
            Literal::UInt32(v) => v.to_string(),
            Literal::UInt64(v) => v.to_string(),
            Literal::UInt128(v) => v.to_string(),
            Literal::Float32(v) => v.to_string(),
            Literal::Float64(v) => v.to_string(),
            Literal::String(s) => s.to_string(),
            Literal::Char(c) => c.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Struct {
                type_name,
                field_initializers
            } => {
                let mut result = String::new();
                result.push_str("<struct literal>\n");
                indent.increase();
                result.push_str(format!("{}type_name: {}", indent.dash(), type_name).as_str());
                if let Some(field_initializers) = field_initializers {
                    let mut i = 0;
                    for field in field_initializers {
                        if i < field_initializers.len() - 1 {
                            result.push_str(format!("\n{}{},", indent.dash(), field.indent_display(indent)).as_str());
                        } else {
                            result.push_str(format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str());
                        }
                        i += 1;
                    }
                } else {
                    result.push_str(format!("\n{}field_initializers: None", indent.dash_end()).as_str());
                }
                indent.decrease();
                result
            },
            Literal::Union {
                type_name,
                member,
                field_initializers
            } => {
                let mut result = String::new();
                result.push_str("<union literal>\n");
                indent.increase();
                result.push_str(format!("{}type_name: {}\n", indent.dash(), type_name).as_str());
                result.push_str(format!("{}member: {}", indent.dash(), member).as_str());
                if let Some(field_initializers) = field_initializers {
                    let mut i = 0;
                    for field in field_initializers {
                        if i < field_initializers.len() - 1 {
                            result.push_str(format!("\n{}{},", indent.dash(), field.indent_display(indent)).as_str());
                        } else {
                            result.push_str(format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str());
                        }
                        i += 1;
                    }
                } else {
                    result.push_str(format!("\n{}field_initializers: None", indent.dash_end()).as_str());
                }
                indent.decrease();
                result
            },
        }
    }
}

impl IndentDisplay for StructField {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<struct field> {}\n", self.identifier).as_str());
        indent.increase();
        if let Some(access_modifier) = &self.access_modifier {
            result.push_str(format!("{}access_modifier: {}\n", indent.dash(), access_modifier.indent_display(indent)).as_str());
        } else {
            result.push_str(format!("{}access_modifier: None\n", indent.dash()).as_str());
        }
        result.push_str(format!("{}type_name: {}\n", indent.dash(), self.type_name).as_str());
        result.push_str(format!("{}mutable: {}", indent.dash_end(), self.mutable).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for UnionMember {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<union member> {}", self.identifier).as_str());
        indent.increase();
        let mut i = 0;
        for field in &self.fields {
            if i == 0 {
                result.push_str(format!("\n{}{}", indent.dash(), field.indent_display(indent)).as_str());
            } else {
                if i < self.fields.len() - 1 {
                    result.push_str(format!("\n{}{},", indent.dash(), field.indent_display(indent)).as_str());
                } else {
                    result.push_str(format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str());
                }
            }

            i += 1;
        }
        indent.decrease();
        result
    }
}

impl IndentDisplay for UnionMemberField {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<union member field>\n");
        indent.increase();
        if let Some(identifier) = &self.identifier {
            result.push_str(format!("{}identifier: {}\n", indent.dash(), identifier).as_str());
        } else {
            result.push_str(format!("{}identifier: None\n", indent.dash()).as_str());
        }
        result.push_str(format!("{}type_name: {}", indent.dash_end(), self.type_name).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for AccessModifier {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            AccessModifier::Public => "public".to_string(),
            AccessModifier::Internal => "internal".to_string(),
        }
    }
}

impl IndentDisplay for FieldInitializer {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<field initializer>\n");
        if let Some(identifier) = &self.identifier {
            result.push_str(format!("{}field initializer: {}\n", indent.dash(), identifier).as_str());
        } else {
            result.push_str(format!("{}field initializer: None\n", indent.dash()).as_str());
        }
        result.push_str(format!("{}initializer: {}", indent.dash_end(), self.initializer.indent_display(indent)).as_str());
        result
    }
}

impl IndentDisplay for UnaryOperator {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            UnaryOperator::Negation => "-".to_string(),
            UnaryOperator::LogicalNot => "!".to_string(),
            UnaryOperator::BitwiseNot => "~".to_string(),
        }
    }
}

impl IndentDisplay for BinaryOperator {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            BinaryOperator::Addition => "+".to_string(),
            BinaryOperator::Subtraction => "-".to_string(),
            BinaryOperator::Multiplication => "*".to_string(),
            BinaryOperator::Division => "/".to_string(),
            BinaryOperator::Modulo => "%".to_string(),
            BinaryOperator::BitwiseAnd => "&".to_string(),
            BinaryOperator::BitwiseOr => "|".to_string(),
            BinaryOperator::BitwiseXor => "^".to_string(),
            BinaryOperator::BitwiseLeftShift => "<<".to_string(),
            BinaryOperator::BitwiseRightShift => ">>".to_string(),
            BinaryOperator::BooleanLogicalAnd => "&&".to_string(),
            BinaryOperator::BooleanLogicalOr => "||".to_string(),
            BinaryOperator::Equal => "==".to_string(),
            BinaryOperator::NotEqual => "!=".to_string(),
            BinaryOperator::LessThan => "<".to_string(),
            BinaryOperator::LessThanOrEqual => "<=".to_string(),
            BinaryOperator::GreaterThan => ">".to_string(),
            BinaryOperator::GreaterThanOrEqual => ">=".to_string(),
        }
    }
}

impl IndentDisplay for Parameter {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<parameter>\n");
        result.push_str(format!("{}parameter: {}\n", indent.dash(), self.identifier).as_str());
        result.push_str(format!("{}type_name: {}", indent.dash_end(), self.type_name).as_str());
        result
    }
}

impl IndentDisplay for ConditionBlock {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<condition block>\n");
        indent.increase();
        result.push_str(format!("{}condition: {}\n", indent.dash(), self.condition.indent_display(indent)).as_str());
        result.push_str(format!("{}body: {}", indent.dash_end(), self.block.indent_display(indent)).as_str());
        indent.decrease();
        result
    }
}
