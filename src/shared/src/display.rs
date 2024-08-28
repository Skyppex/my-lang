use crate::{
    parser::{
        AccessModifier, Assignment, AssociatedType, Binary, BinaryOperator, Call, ClosureParameter,
        EnumDeclaration, EnumMember, EnumMemberField, EnumMemberFieldInitializers, Expression,
        FieldInitializer, FlagsMember, For, FunctionDeclaration, If, Literal, Match, MatchArm,
        Member, ModuleDeclaration, Parameter, ProtocolDeclaration, Statement, StructDeclaration,
        StructField, TypeAliasDeclaration, Unary, UnaryOperator, UnionDeclaration, Use, UseItem,
        VariableDeclaration, While,
    },
    type_checker::{
        self,
        ast::{
            Block, TypedClosureParameter, TypedExpression, TypedMatchArm, TypedParameter,
            TypedStatement,
        },
        decision_tree::{Case, Constructor, Decision, FieldPattern, Pattern, Variable},
        FullName, Type,
    },
    types::{GenericConstraint, GenericType, TypeAnnotation, TypeIdentifier},
};

pub struct Indent {
    levels: Vec<bool>,
}

impl Indent {
    pub fn new() -> Indent {
        Indent { levels: vec![] }
    }

    fn increase(&mut self) {
        self.levels.push(false);
    }

    fn increase_leaf(&mut self) {
        self.levels.push(true);
    }

    fn decrease(&mut self) {
        self.levels.pop();
    }

    fn end_current(&mut self) {
        let len = self.levels.len();

        if len == 0 {
            return;
        }

        self.levels[len - 1] = true;
    }

    fn dash(&self) -> String {
        let mut result = String::new();

        for is_end in self.levels.iter().rev().skip(1).rev() {
            result.push_str(if *is_end { "  " } else { "┆ " });
        }

        result.push_str("├─");
        result
    }

    fn dash_end(&self) -> String {
        let mut result = String::new();
        for is_end in self.levels.iter().rev().skip(1).rev() {
            result.push_str(if *is_end { "  " } else { "┆ " });
        }

        self.levels.last().map(|is_end| {
            result.push_str(if *is_end { "╰─" } else { "├─" });
        });

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

                for (i, statement) in statements.iter().enumerate() {
                    result.push_str(&statement.indent_display(indent));
                    if i < statements.len() - 1 {
                        result.push_str("\n\n");
                    }
                }

                result
            }
            Statement::ModuleDeclaration(ModuleDeclaration {
                access_modifier,
                module_path,
            }) => {
                let mut result = String::new();
                result.push_str("<module statement>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}access_modifier: {}\n",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}module_path: {}",
                        indent.dash_end(),
                        module_path.join("::")
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Statement::Use(Use { use_item }) => {
                let mut result = String::new();
                result.push_str("<use statement>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}use_item: {}",
                        indent.dash(),
                        use_item.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Statement::StructDeclaration(StructDeclaration {
                access_modifier,
                type_identifier,
                where_clause,
                fields,
            }) => {
                let mut result = String::new();
                result.push_str("<struct declaration>\n");
                indent.increase();

                result.push_str(
                    format!(
                        "{}type_name: {}\n",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}access_modifier: {}",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );

                if let Some(where_clause) = where_clause {
                    result.push_str(
                        format!(
                            "\n{}where_clause: {}",
                            indent.dash(),
                            indent_display_vec(
                                where_clause,
                                "generic constraints",
                                "constraint",
                                indent
                            )
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}where_clause: None", indent.dash()).as_str());
                }

                for (i, field) in fields.iter().enumerate() {
                    if i < fields.len() - 1 {
                        result.push_str(
                            format!("\n{}{}", indent.dash(), field.indent_display(indent)).as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), field.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            Statement::EnumDeclaration(EnumDeclaration {
                access_modifier,
                type_identifier,
                shared_fields,
                members,
            }) => {
                let mut result = String::new();
                result.push_str("<enum declaration>\n");
                indent.increase();

                result.push_str(
                    format!(
                        "{}type_name: {}\n",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}access_modifier: {}",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, shared_field) in shared_fields.iter().enumerate() {
                    if i < shared_fields.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}{},",
                                indent.dash(),
                                shared_field.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                shared_field.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                for (i, member) in members.iter().enumerate() {
                    if i < members.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), member.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), member.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            Statement::UnionDeclaration(UnionDeclaration {
                access_modifier,
                type_identifier,
                literals,
            }) => {
                let mut result = String::new();
                result.push_str("<union declaration>");
                indent.increase();

                result.push_str(
                    format!(
                        "\n{}access_modifier: {}",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}{}",
                        indent.dash_end(),
                        indent_display_vec(literals, "literals", "literal", indent)
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            // Statement::FlagsDeclaration(FlagsDeclaration {
            //     access_modifier,
            //     type_name,
            //     members
            // }) => {
            //     let mut result = String::new();
            //     result.push_str(format!("<flags declaration> {}\n", type_name).as_str());
            //     indent.increase();
            //     result.push_str(format!("{}access_modifier: {}\n", indent.dash(), access_modifier.indent_display(indent)).as_str());

            //     for (i, member) in members.iter().enumerate() {
            //         let is_end = i == members.len() - 1;
            //         indent.current(is_end);
            //         result.push_str(format!("\n{}{}", indent.dash_end(), member.indent_display(indent)).as_str());
            //     }

            //     indent.decrease();
            //     result
            // },
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                access_modifier,
                type_identifier,
                type_annotations,
            }) => {
                let mut result = String::new();
                result.push_str("<type alias declaration>");
                indent.increase();

                result.push_str(
                    format!(
                        "\n{}access_modifier: {}",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}{}",
                        indent.dash_end(),
                        indent_display_vec(
                            type_annotations,
                            "type_annotations",
                            "type_annotation",
                            indent
                        )
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            Statement::ProtocolDeclaration(ProtocolDeclaration {
                access_modifier,
                type_identifier,
                associated_types,
                functions,
            }) => {
                let mut result = String::new();
                result.push_str("<protocol declaration>");
                indent.increase();

                result.push_str(
                    format!(
                        "\n{}access_modifier: {}",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}{}",
                        indent.dash(),
                        indent_display_vec(
                            associated_types,
                            "associated_types",
                            "associated_type",
                            indent
                        )
                    )
                    .as_str(),
                );

                indent.end_current();

                result.push_str(
                    format!(
                        "\n{}{}",
                        indent.dash_end(),
                        indent_display_vec(functions, "functions", "function", indent)
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            Statement::FunctionDeclaration(function_declaration) => {
                function_declaration.indent_display(indent)
            }
            Statement::Semi(statement) => {
                let mut result = String::new();
                result.push_str("<semi>\n");
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!(
                        "{}statement: {}",
                        indent.dash_end(),
                        statement.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Statement::Expression(e) => e.indent_display(indent),
        }
    }
}

impl IndentDisplay for AssociatedType {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<associated type>\n");
        indent.increase();
        result.push_str(format!("{}name: {}\n", indent.dash(), self.type_identifier).as_str());
        indent.end_current();
        result.push_str(
            format!(
                "{}type_annotation: {}",
                indent.dash_end(),
                self.default_type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for UseItem {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            UseItem::Item(item) => item.clone(),
            UseItem::Navigation(item, next) => {
                let mut result = String::new();
                result.push_str(format!("<navigation> {}\n", item).as_str());
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!("{}next: {}", indent.dash_end(), next.indent_display(indent)).as_str(),
                );
                indent.decrease();
                result
            }
            UseItem::List(items) => {
                let mut result = String::new();
                result.push_str("<list>");
                indent.increase();

                for (i, item) in items.iter().enumerate() {
                    if i < items.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), item.indent_display(indent)).as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), item.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for FunctionDeclaration {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let identifier = &self.type_identifier;
        let access_modifier = &self.access_modifier;
        let param = &self.param;
        let return_type_annotation = &self.return_type_annotation;
        let body = &self.body;

        let mut result = String::new();
        result.push_str("<function declaration>\n");
        indent.increase();

        result.push_str(format!("{}identifier: {}\n", indent.dash(), identifier).as_str());
        result.push_str(
            format!(
                "{}access_modifier: {}\n",
                indent.dash(),
                access_modifier.indent_display(indent)
            )
            .as_str(),
        );

        result.push_str(
            format!("{}param: {}\n", indent.dash(), param.indent_display(indent)).as_str(),
        );

        result.push_str(
            format!(
                "{}return_type: {}\n",
                indent.dash(),
                return_type_annotation.indent_display(indent)
            )
            .as_str(),
        );

        indent.end_current();
        result.push_str(
            format!("{}body: {}", indent.dash_end(), body.indent_display(indent)).as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for Expression {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            // Expression::None => String::new(),
            Expression::VariableDeclaration(VariableDeclaration {
                mutable,
                type_annotation,
                identifier,
                initializer,
            }) => {
                let mut result = String::new();
                result.push_str(format!("<variable declaration> {}\n", identifier).as_str());
                indent.increase();
                result.push_str(format!("{}mutable: {}\n", indent.dash(), mutable).as_str());
                result.push_str(
                    format!(
                        "{}type_annotation: {}\n",
                        indent.dash(),
                        type_annotation.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}initializer: {}",
                        indent.dash_end(),
                        initializer.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::If(If {
                condition,
                true_expression,
                false_expression,
            }) => {
                let mut result = String::new();
                result.push_str("<if>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}condition:{}",
                        indent.dash(),
                        condition.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "\n{}true expression: {}",
                        indent.dash(),
                        true_expression.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}false expression: {}",
                        indent.dash_end(),
                        false_expression.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::Match(Match { expression, arms }) => {
                let mut result = String::new();
                result.push_str("<match>");
                indent.increase();
                result.push_str(
                    format!(
                        "\n{}expression: {}",
                        indent.dash(),
                        expression.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, arm) in arms.iter().enumerate() {
                    if i < arms.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), arm.indent_display(indent)).as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), arm.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            Expression::Assignment(Assignment {
                member,
                initializer,
            }) => {
                let mut result = String::new();
                result.push_str("<assignment>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}member: {}\n",
                        indent.dash(),
                        member.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}initializer: {}",
                        indent.dash_end(),
                        initializer.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::Member(m) => m.indent_display(indent),
            Expression::Literal(l) => l.indent_display(indent),
            Expression::Closure(c) => {
                let mut result = String::new();
                result.push_str("<closure>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}param: {}\n",
                        indent.dash(),
                        c.param.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}return_type: {}\n",
                        indent.dash(),
                        c.return_type_annotation.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}body: {}",
                        indent.dash_end(),
                        c.body.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::Call(Call { callee, argument }) => {
                let mut result = String::new();
                result.push_str("<call>\n");
                indent.increase();
                result.push_str(
                    format!("{}callee: {}", indent.dash(), callee.indent_display(indent)).as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}argument: {}",
                        indent.dash_end(),
                        argument.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            Expression::Unary(Unary {
                operator,
                expression,
            }) => {
                let mut result = String::new();
                result.push_str("<unary>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}operator: {}\n",
                        indent.dash(),
                        operator.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}expression: {}",
                        indent.dash_end(),
                        expression.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::Binary(Binary {
                left,
                operator,
                right,
            }) => {
                let mut result = String::new();
                result.push_str("<binary>\n");
                indent.increase();
                result.push_str(
                    format!("{}left: {}\n", indent.dash(), left.indent_display(indent)).as_str(),
                );
                result.push_str(
                    format!(
                        "{}operator: {}\n",
                        indent.dash(),
                        operator.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}right: {}",
                        indent.dash_end(),
                        right.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::Block(statements) => {
                let mut result = String::new();
                result.push_str("<block>");
                indent.increase();

                for (i, statement) in statements.iter().enumerate() {
                    if i < statements.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), statement.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                statement.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            Expression::Print(value) => {
                let mut result = String::new();
                result.push_str(
                    format!(
                        "{}<print> {}",
                        indent.dash_end(),
                        value.indent_display(indent)
                    )
                    .as_str(),
                );
                result
            }
            Expression::Drop(identifier) => {
                let mut result = String::new();
                result.push_str(format!("<drop> {}", identifier).as_str());
                result
            }
            Expression::Loop(body) => {
                let mut result = String::new();
                result.push_str("<loop>");
                indent.increase_leaf();
                result.push_str(
                    format!(
                        "\n{}body: {}",
                        indent.dash_end(),
                        body.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::While(While {
                condition,
                body,
                else_body,
            }) => {
                let mut result = String::new();
                result.push_str("<while>");
                indent.increase();
                result.push_str(
                    format!(
                        "\n{}condition: {}",
                        indent.dash(),
                        condition.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}body: {}",
                        indent.dash_end(),
                        body.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();

                if let Some(else_body) = else_body {
                    result.push_str(
                        format!(
                            "\n{}else: {}",
                            indent.dash_end(),
                            else_body.indent_display(indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}else: None", indent.dash_end()).as_str());
                }

                indent.decrease();
                result
            }
            Expression::For(For {
                identifier,
                iterable,
                body,
                else_body,
            }) => {
                let mut result = String::new();
                result.push_str("<for>");
                indent.increase();
                result.push_str(format!("\n{}identifier: {}", indent.dash(), identifier).as_str());
                result.push_str(
                    format!(
                        "\n{}iterable: {}",
                        indent.dash(),
                        iterable.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}body: {}",
                        indent.dash_end(),
                        body.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();

                if let Some(else_body) = else_body {
                    result.push_str(
                        format!(
                            "\n{}else: {}",
                            indent.dash_end(),
                            else_body.indent_display(indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}else: None", indent.dash_end()).as_str());
                }

                indent.decrease();
                result
            }
            Expression::Break(e) => {
                let mut result = String::new();
                result.push_str("<break>\n");
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!(
                        "{}expression: {}",
                        indent.dash_end(),
                        e.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            Expression::Continue => {
                let mut result = String::new();
                result.push_str("<continue>");
                result
            }
            Expression::Return(e) => {
                let mut result = String::new();
                result.push_str("<return>\n");
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!(
                        "{}expression: {}",
                        indent.dash_end(),
                        e.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for Member {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Member::Identifier { symbol, generics } => {
                let mut result = String::new();
                result.push_str(format!("<identifier> {}", symbol).as_str());

                match generics {
                    Some(generics) => result.push_str(
                        format!(
                            "<{}>",
                            generics
                                .into_iter()
                                .map(|g| g.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                        .as_str(),
                    ),
                    None => {}
                }

                result
            }
            Member::MemberAccess {
                object,
                member,
                symbol,
                generics,
            } => {
                let mut result = String::new();
                result.push_str("<member access>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}object: {}\n",
                        indent.dash(),
                        object.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "{}member: {}\n",
                        indent.dash(),
                        member.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(format!("{}symbol: {}\n", indent.dash(), symbol).as_str());

                indent.end_current();

                if let Some(generics) = generics {
                    result.push_str(
                        format!(
                            "\n{}generics: {}",
                            indent.dash_end(),
                            indent_display_vec(generics, "generics", "generic", indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}generics: None", indent.dash_end()).as_str());
                }

                indent.decrease();
                result
            }
            Member::ParamPropagation {
                object,
                member,
                symbol,
                generics: _,
            } => {
                let mut result = String::new();
                result.push_str("<param propagation>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}object: {}\n",
                        indent.dash(),
                        object.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}member: {}\n",
                        indent.dash(),
                        member.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(format!("{}symbol: {}", indent.dash_end(), symbol).as_str());
                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for Literal {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Literal::Unit => "unit".to_string(),
            Literal::Int(v) => v.to_string(),
            Literal::UInt(v) => v.to_string(),
            Literal::Float(v) => v.to_string(),
            Literal::String(s) => s.to_string(),
            Literal::Char(c) => c.to_string(),
            Literal::Bool(b) => b.to_string(),
            Literal::Array(expressions) => {
                let mut result = String::new();
                result.push_str("<array>");
                indent.increase();

                for (i, expression) in expressions.iter().enumerate() {
                    if i < expressions.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), expression.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                expression.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            Literal::Struct {
                type_annotation: type_identifier,
                field_initializers,
            } => {
                let mut result = String::new();
                result.push_str("<struct literal>\n");
                indent.increase();
                result.push_str(
                    format!(
                        "{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, field) in field_initializers.iter().enumerate() {
                    if i < field_initializers.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), field.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), field.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            Literal::Enum {
                type_annotation: type_identifier,
                member,
                field_initializers,
            } => {
                let mut result = String::new();
                result.push_str("<enum literal>\n");
                indent.increase_leaf();
                result.push_str(
                    format!(
                        "{}type_name: {}\n",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(format!("{}member: {}\n", indent.dash_end(), member).as_str());
                indent.increase_leaf();
                result.push_str(
                    format!(
                        "{}{}",
                        indent.dash_end(),
                        field_initializers.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for StructField {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<struct field> {}\n", self.identifier).as_str());
        indent.increase();

        if let Some(access_modifier) = &self.access_modifier {
            result.push_str(
                format!(
                    "{}access_modifier: {}\n",
                    indent.dash(),
                    access_modifier.indent_display(indent)
                )
                .as_str(),
            );
        } else {
            result.push_str(format!("{}access_modifier: None\n", indent.dash()).as_str());
        }

        result.push_str(
            format!(
                "{}type_annotation: {}\n",
                indent.dash(),
                self.type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.end_current();
        result.push_str(format!("{}mutable: {}", indent.dash_end(), self.mutable).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for EnumMember {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<enum member> {}", self.identifier).as_str());
        indent.increase();

        for (i, field) in self.fields.iter().enumerate() {
            if i < self.fields.len() - 1 {
                result.push_str(
                    format!("\n{}{},", indent.dash(), field.indent_display(indent)).as_str(),
                );
            } else {
                indent.end_current();
                result.push_str(
                    format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str(),
                );
            }
        }

        indent.decrease();
        result
    }
}

impl IndentDisplay for EnumMemberField {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<enum member field>\n");
        indent.increase_leaf();
        result.push_str(format!("{}identifier: {}\n", indent.dash(), &self.identifier).as_str());
        result.push_str(
            format!(
                "{}type_annotation: {}",
                indent.dash_end(),
                self.type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for FlagsMember {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<flags member>\n");
        indent.increase_leaf();
        result.push_str(format!("{}identifier: {}\n", indent.dash(), &self.identifier).as_str());
        result.push_str(format!("{}value: {}", indent.dash_end(), self.value).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for AccessModifier {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            AccessModifier::Public => "public".to_string(),
            AccessModifier::Module => "module".to_string(),
            AccessModifier::Super => "super".to_string(),
        }
    }
}

impl IndentDisplay for FieldInitializer {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<field initializer>\n");
        indent.increase();

        if let Some(identifier) = &self.identifier {
            result
                .push_str(format!("{}field initializer: {}\n", indent.dash(), identifier).as_str());
        } else {
            result.push_str(format!("{}field initializer: None\n", indent.dash()).as_str());
        }

        indent.end_current();
        result.push_str(
            format!(
                "{}initializer: {}",
                indent.dash_end(),
                self.initializer.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for EnumMemberFieldInitializers {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            EnumMemberFieldInitializers::None => "".to_string(),
            EnumMemberFieldInitializers::Named(field_initializers) => {
                let mut result = String::new();
                result.push_str("<named field initializers>");
                indent.increase();

                for (i, (identifier, initializer)) in field_initializers.iter().enumerate() {
                    if i < field_initializers.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}{}: {},",
                                indent.dash(),
                                identifier,
                                initializer.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}: {}",
                                indent.dash_end(),
                                identifier,
                                initializer.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for Parameter {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<parameter>\n");
        indent.increase_leaf();
        result.push_str(
            format!(
                "{}name: {}\n",
                indent.dash(),
                self.identifier.indent_display(indent)
            )
            .as_str(),
        );
        result.push_str(
            format!(
                "{}type_annotation: {}",
                indent.dash_end(),
                self.type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for ClosureParameter {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<parameter>\n");
        indent.increase_leaf();
        result.push_str(
            format!(
                "{}name: {}\n",
                indent.dash(),
                self.identifier.indent_display(indent)
            )
            .as_str(),
        );
        result.push_str(
            format!(
                "{}type_annotation: {}",
                indent.dash_end(),
                self.type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for UnaryOperator {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            UnaryOperator::Identity => "+".to_string(),
            UnaryOperator::Negate => "-".to_string(),
            UnaryOperator::LogicalNot => "!".to_string(),
            UnaryOperator::BitwiseNot => "~".to_string(),
        }
    }
}

impl IndentDisplay for BinaryOperator {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            BinaryOperator::Add => "+".to_string(),
            BinaryOperator::Subtract => "-".to_string(),
            BinaryOperator::Multiply => "*".to_string(),
            BinaryOperator::Divide => "/".to_string(),
            BinaryOperator::Modulo => "%".to_string(),
            BinaryOperator::BitwiseAnd => "&".to_string(),
            BinaryOperator::BitwiseOr => "|".to_string(),
            BinaryOperator::BitwiseXor => "^".to_string(),
            BinaryOperator::BitwiseLeftShift => "<<".to_string(),
            BinaryOperator::BitwiseRightShift => ">>".to_string(),
            BinaryOperator::LogicalAnd => "&&".to_string(),
            BinaryOperator::LogicalOr => "||".to_string(),
            BinaryOperator::Equal => "==".to_string(),
            BinaryOperator::NotEqual => "!=".to_string(),
            BinaryOperator::LessThan => "<".to_string(),
            BinaryOperator::LessThanOrEqual => "<=".to_string(),
            BinaryOperator::GreaterThan => ">".to_string(),
            BinaryOperator::GreaterThanOrEqual => ">=".to_string(),
            BinaryOperator::Range => "..".to_string(),
            BinaryOperator::RangeInclusive => "..=".to_string(),
        }
    }
}

impl IndentDisplay for MatchArm {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<match arm>");
        indent.increase();
        result.push_str(
            format!(
                "\n{}pattern: {}",
                indent.dash(),
                self.pattern.indent_display(indent)
            )
            .as_str(),
        );
        indent.end_current();
        result.push_str(
            format!(
                "\n{}expression: {}",
                indent.dash_end(),
                self.expression.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for Pattern {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Unit => "unit".to_string(),
            Pattern::Bool(b) => b.to_string(),
            Pattern::Int(v) => v.to_string(),
            Pattern::UInt(v) => v.to_string(),
            Pattern::Float(v) => v.to_string(),
            Pattern::Char(c) => c.to_string(),
            Pattern::String(s) => s.to_string(),
            Pattern::Variable(v) => v.to_string(),
            Pattern::Constructor(Constructor::Struct {
                type_annotation,
                field_patterns,
            }) => {
                let mut result = String::new();

                result.push_str("<struct pattern>");
                indent.increase();
                result.push_str(
                    format!(
                        "\n{}type_annotation: {}",
                        indent.dash(),
                        type_annotation.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, field_pattern) in field_patterns.iter().enumerate() {
                    if i < field_patterns.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}{},",
                                indent.dash(),
                                field_pattern.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                field_pattern.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();

                result
            }
        }
    }
}

impl IndentDisplay for FieldPattern {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<field pattern>\n");
        indent.increase_leaf();
        result.push_str(
            format!(
                "{}identifier: {}\n",
                indent.dash(),
                self.identifier.indent_display(indent)
            )
            .as_str(),
        );
        indent.end_current();
        result.push_str(
            format!(
                "{}pattern: {}",
                indent.dash_end(),
                self.pattern.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for TypedStatement {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            TypedStatement::None => String::new(),
            TypedStatement::Program { statements } => {
                let mut result = String::new();

                for (i, statement) in statements.iter().enumerate() {
                    result.push_str(&statement.indent_display(indent));
                    if i < statements.len() - 1 {
                        result.push_str("\n\n");
                    }
                }

                result
            }
            TypedStatement::ModuleDeclaration {
                access_modifier,
                module_path,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<module declaration> {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}access_modifier: {}\n",
                        indent.dash(),
                        access_modifier.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!("{}module_path: {}", indent.dash(), module_path.join("::")).as_str(),
                );
                indent.decrease();
                result
            }
            TypedStatement::Use { use_item, type_ } => {
                let mut result = String::new();
                result.push_str(format!("<use> {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}use_item: {}",
                        indent.dash(),
                        use_item.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedStatement::StructDeclaration {
                type_identifier,
                where_clause,
                fields,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<struct declaration> {}\n", type_).as_str());
                indent.increase();

                result.push_str(
                    format!(
                        "{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                if let Some(where_clause) = where_clause {
                    result.push_str(
                        format!(
                            "\n{}where_clause: {}",
                            indent.dash(),
                            indent_display_vec(
                                where_clause,
                                "generic constraints",
                                "constraint",
                                indent
                            )
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}where_clause: None", indent.dash()).as_str());
                }

                for field in fields.iter() {
                    indent.end_current();
                    result.push_str(
                        format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str(),
                    );
                }

                indent.decrease();
                result
            }
            TypedStatement::EnumDeclaration {
                type_identifier,
                shared_fields,
                members,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<enum declaration> {}\n", type_).as_str());
                indent.increase();

                result.push_str(
                    format!(
                        "{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, shared_field) in shared_fields.iter().enumerate() {
                    if i < members.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}{},",
                                indent.dash(),
                                shared_field.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                shared_field.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                for (i, member) in members.iter().enumerate() {
                    if i < members.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), member.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), member.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            TypedStatement::UnionDeclaration {
                type_identifier,
                literals,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<union declaration> {}", type_).as_str());
                indent.increase();

                result.push_str(
                    format!(
                        "\n{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, literal) in literals.iter().enumerate() {
                    if i < literals.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), literal.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), literal.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            TypedStatement::ProtocolDeclaration {
                type_identifier,
                associated_types,
                functions,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<protocol declaration> {}\n", type_).as_str());
                indent.increase();

                result.push_str(
                    format!(
                        "{}type_name: {}\n",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "{}{}\n",
                        indent.dash(),
                        indent_display_vec(
                            associated_types,
                            "associated_types",
                            "associated_type",
                            indent
                        )
                    )
                    .as_str(),
                );

                indent.end_current();

                result.push_str(
                    format!(
                        "{}{}\n",
                        indent.dash_end(),
                        indent_display_vec(functions, "functions", "function", indent)
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            TypedStatement::TypeAliasDeclaration {
                type_identifier,
                type_annotations,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<type alias declaration> {}", type_).as_str());
                indent.increase();

                result.push_str(
                    format!(
                        "\n{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}{}",
                        indent.dash_end(),
                        indent_display_vec(
                            type_annotations,
                            "type_annotations",
                            "type_annotation",
                            indent
                        )
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            TypedStatement::FunctionDeclaration {
                identifier,
                param,
                return_type,
                body,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<function declaration> {}\n", type_).as_str());
                indent.increase();

                result.push_str(format!("{}identifier: {}\n", indent.dash(), identifier).as_str());

                result.push_str(
                    format!("{}{}\n", indent.dash(), param.indent_display(indent)).as_str(),
                );

                result
                    .push_str(format!("{}return_type: {}\n", indent.dash(), return_type).as_str());

                indent.end_current();
                result.push_str(
                    format!("{}body: {}", indent.dash_end(), body.indent_display(indent)).as_str(),
                );

                indent.decrease();
                result
            }
            TypedStatement::Semi(e) => {
                let mut result = String::new();
                result.push_str(format!("<semi>: {}\n", Type::Void).as_str());
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!(
                        "{}statement: {}",
                        indent.dash_end(),
                        e.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedStatement::Expression(e) => e.indent_display(indent),
        }
    }
}

impl IndentDisplay for TypedExpression {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            // TypedExpression::None => String::new(),
            TypedExpression::VariableDeclaration {
                mutable,
                identifier,
                initializer,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(
                    format!("<variable declaration> {}: {}\n", identifier, type_).as_str(),
                );
                indent.increase();
                result.push_str(format!("{}mutable: {}\n", indent.dash(), mutable).as_str());
                indent.end_current();

                if let Some(initializer) = initializer {
                    result.push_str(
                        format!(
                            "{}initializer: {}",
                            indent.dash_end(),
                            initializer.indent_display(indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("{}initializer: None", indent.dash_end()).as_str());
                }

                indent.decrease();
                result
            }
            TypedExpression::If {
                condition,
                true_expression,
                false_expression,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<if>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}condition:{}",
                        indent.dash(),
                        condition.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}true expression: {}",
                        indent.dash(),
                        true_expression.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();

                if let Some(false_expression) = false_expression {
                    result.push_str(
                        format!(
                            "\n{}false expression: {}",
                            indent.dash_end(),
                            false_expression.indent_display(indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(
                        format!("\n{}false expression: None", indent.dash_end()).as_str(),
                    );
                }

                indent.decrease();
                result
            }
            TypedExpression::Match {
                expression,
                arms,
                decision_tree,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<match>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}expression: {}\n",
                        indent.dash(),
                        expression.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "{}{}",
                        indent.dash(),
                        indent_display_vec(arms, "arms", "arm", indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}decision_tree: {}",
                        indent.dash_end(),
                        decision_tree.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            TypedExpression::Assignment {
                member,
                initializer,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<assignment>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}member: {}\n",
                        indent.dash(),
                        member.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}initializer: {}",
                        indent.dash_end(),
                        initializer.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedExpression::Member(m) => m.indent_display(indent),
            TypedExpression::Literal(l) => l.indent_display(indent),
            TypedExpression::Closure {
                param,
                return_type,
                body,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<closure>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!("{}param: {}\n", indent.dash(), param.indent_display(indent)).as_str(),
                );
                result.push_str(
                    format!(
                        "{}return_type: {}\n",
                        indent.dash(),
                        return_type.type_annotation().indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!("{}body: {}", indent.dash_end(), body.indent_display(indent)).as_str(),
                );
                indent.decrease();
                result
            }
            TypedExpression::Call {
                callee,
                argument,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<call>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!("{}callee: {}", indent.dash(), callee.indent_display(indent)).as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}argument: {}",
                        indent.dash_end(),
                        argument.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.decrease();
                result
            }
            TypedExpression::Index {
                callee,
                argument,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<index>: {}\n", type_).as_str());
                indent.increase_leaf();
                result.push_str(
                    format!(
                        "{}callee: {}\n",
                        indent.dash(),
                        callee.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}index: {}",
                        indent.dash_end(),
                        argument.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedExpression::Unary {
                operator,
                expression,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<unary>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}operator: {}\n",
                        indent.dash(),
                        operator.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}expression: {}",
                        indent.dash_end(),
                        expression.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedExpression::Binary {
                left,
                operator,
                right,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<binary>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!("{}left: {}\n", indent.dash(), left.indent_display(indent)).as_str(),
                );
                result.push_str(
                    format!(
                        "{}operator: {}\n",
                        indent.dash(),
                        operator.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(
                    format!(
                        "{}right: {}",
                        indent.dash_end(),
                        right.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedExpression::Block(Block { statements, type_ }) => {
                let mut result = String::new();
                result.push_str(format!("<block>: {}", type_).as_str());
                indent.increase();

                for (i, statement) in statements.iter().enumerate() {
                    if i < statements.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), statement.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                statement.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            TypedExpression::Print { value } => {
                let mut result = String::new();
                result.push_str(format!("<print> {}: {}", value, Type::Void).as_str());
                result
            }
            TypedExpression::Drop { identifier, type_ } => {
                let mut result = String::new();
                result.push_str(format!("<drop> {}: {}", identifier, type_).as_str());
                result
            }
            TypedExpression::Loop { body, type_ } => {
                let mut result = String::new();
                result.push_str(format!("<loop>: {}", type_).as_str());
                indent.increase();
                indent.end_current();

                result.push_str(
                    format!("{}body: {}", indent.dash_end(), body.indent_display(indent)).as_str(),
                );

                indent.decrease();
                result
            }
            TypedExpression::While {
                condition,
                body,
                else_body,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<while>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}condition: {}\n",
                        indent.dash(),
                        condition.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}body: {}",
                        indent.dash_end(),
                        body.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();

                if let Some(else_body) = else_body {
                    result.push_str(
                        format!(
                            "\n{}else: {}",
                            indent.dash_end(),
                            else_body.indent_display(indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}else: None", indent.dash_end()).as_str());
                }

                indent.decrease();
                result
            }
            TypedExpression::For {
                identifier,
                iterable,
                body,
                else_body,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<for>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}identifier: {}\n",
                        indent.dash(),
                        identifier.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}iterable: {}\n",
                        indent.dash(),
                        iterable.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}body: {}",
                        indent.dash_end(),
                        body.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();

                if let Some(else_body) = else_body {
                    result.push_str(
                        format!(
                            "\n{}else: {}",
                            indent.dash_end(),
                            else_body.indent_display(indent)
                        )
                        .as_str(),
                    );
                } else {
                    result.push_str(format!("\n{}else: None", indent.dash_end()).as_str());
                }

                indent.decrease();
                result
            }
            TypedExpression::Break(e) => {
                let mut result = String::new();
                result.push_str(format!("<break>: {}\n", Type::Void).as_str());
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!(
                        "{}expression: {}",
                        indent.dash_end(),
                        e.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
            TypedExpression::Continue => {
                let mut result = String::new();
                result.push_str(format!("<continue>: {}\n", Type::Void).as_str());
                result
            }
            TypedExpression::Return(e) => {
                let mut result = String::new();
                result.push_str(format!("<return>: {}\n", Type::Void).as_str());
                indent.increase();
                indent.end_current();
                result.push_str(
                    format!(
                        "{}expression: {}",
                        indent.dash_end(),
                        e.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for TypedParameter {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<parameter>\n");
        indent.increase_leaf();
        result.push_str(
            format!(
                "{}name: {}\n",
                indent.dash(),
                self.identifier.indent_display(indent)
            )
            .as_str(),
        );
        result.push_str(
            format!(
                "{}type_annotation: {}",
                indent.dash_end(),
                self.type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for TypedClosureParameter {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<parameter>\n");
        indent.increase_leaf();
        result.push_str(
            format!(
                "{}name: {}\n",
                indent.dash(),
                self.identifier.indent_display(indent)
            )
            .as_str(),
        );
        result.push_str(
            format!(
                "{}type_annotation: {}",
                indent.dash_end(),
                self.type_annotation.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for type_checker::ast::Member {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            type_checker::ast::Member::Identifier { symbol, type_ } => {
                let mut result = String::new();
                result.push_str(format!("<identifier> {}: {}", symbol, type_).as_str());
                result
            }
            type_checker::ast::Member::MemberAccess {
                object,
                member,
                symbol,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<member access>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}object: {}\n",
                        indent.dash(),
                        object.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "{}member: {}\n",
                        indent.dash(),
                        member.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(format!("{}symbol: {}", indent.dash_end(), symbol).as_str());
                indent.decrease();
                result
            } // type_checker::ast::Member::MemberFunctionAccess {
              //     object,
              //     member,
              //     symbol,
              //     type_,
              // } => {
              //     let mut result = String::new();
              //     result.push_str(format!("<member function access>: {}\n", type_).as_str());
              //     indent.increase();
              //     result.push_str(
              //         format!(
              //             "{}object: {}\n",
              //             indent.dash(),
              //             object.indent_display(indent)
              //         )
              //         .as_str(),
              //     );
              //     result.push_str(
              //         format!(
              //             "{}member: {}\n",
              //             indent.dash(),
              //             member.indent_display(indent)
              //         )
              //         .as_str(),
              //     );
              //     indent.end_current();
              //     result.push_str(format!("{}symbol: {}", indent.dash_end(), symbol).as_str());
              //     indent.decrease();
              //     result
              // }
        }
    }
}

impl IndentDisplay for type_checker::ast::Literal {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            type_checker::ast::Literal::Void => "void".to_string(),
            type_checker::ast::Literal::Unit => "unit".to_string(),
            type_checker::ast::Literal::Int(v) => v.to_string(),
            type_checker::ast::Literal::UInt(v) => v.to_string(),
            type_checker::ast::Literal::Float(v) => v.to_string(),
            type_checker::ast::Literal::String(s) => s.to_string(),
            type_checker::ast::Literal::Char(c) => c.to_string(),
            type_checker::ast::Literal::Bool(b) => b.to_string(),
            type_checker::ast::Literal::Array { values, type_ } => {
                let mut result = String::new();
                result.push_str(format!("<array>: {}", type_).as_str());
                indent.increase();

                for (i, expression) in values.iter().enumerate() {
                    if i < values.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), expression.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}",
                                indent.dash_end(),
                                expression.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            type_checker::ast::Literal::Struct {
                type_annotation: type_identifier,
                field_initializers,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<struct literal>: {}\n", type_).as_str());
                indent.increase();
                result.push_str(
                    format!(
                        "{}type_name: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );

                for (i, field) in field_initializers.iter().enumerate() {
                    if i < field_initializers.len() - 1 {
                        result.push_str(
                            format!("\n{}{},", indent.dash(), field.indent_display(indent))
                                .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!("\n{}{}", indent.dash_end(), field.indent_display(indent))
                                .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
            type_checker::ast::Literal::Enum {
                type_annotation: type_identifier,
                member,
                field_initializers,
                type_,
            } => {
                let mut result = String::new();
                result.push_str(format!("<enum literal>: {}\n", type_).as_str());
                indent.increase_leaf();
                result.push_str(
                    format!(
                        "{}type_name: {}\n",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );
                result.push_str(format!("{}member: {}\n", indent.dash_end(), member).as_str());
                indent.increase_leaf();
                result.push_str(
                    format!(
                        "{}{}",
                        indent.dash_end(),
                        field_initializers.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.decrease();
                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for type_checker::ast::StructField {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<struct field> {}: {}\n", self.identifier, self.type_).as_str());
        indent.increase_leaf();
        result.push_str(format!("{}mutable: {}", indent.dash_end(), self.mutable).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for type_checker::ast::EnumMember {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<enum member> {}", self.type_).as_str());
        indent.increase();

        for (i, field) in self.fields.iter().enumerate() {
            if i < self.fields.len() - 1 {
                result.push_str(
                    format!("\n{}{},", indent.dash(), field.indent_display(indent)).as_str(),
                );
            } else {
                indent.end_current();
                result.push_str(
                    format!("\n{}{}", indent.dash_end(), field.indent_display(indent)).as_str(),
                );
            }
        }

        indent.decrease();
        result
    }
}

impl IndentDisplay for type_checker::ast::EnumMemberField {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<enum member field>: {}\n", self.type_).as_str());
        indent.increase_leaf();
        result.push_str(format!("{}identifier: {}\n", indent.dash(), self.identifier).as_str());
        result.push_str(format!("{}type: {}", indent.dash_end(), self.type_).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for type_checker::ast::UnaryOperator {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            type_checker::ast::UnaryOperator::Identity => "+".to_string(),
            type_checker::ast::UnaryOperator::Negate => "-".to_string(),
            type_checker::ast::UnaryOperator::LogicalNot => "!".to_string(),
            type_checker::ast::UnaryOperator::BitwiseNot => "~".to_string(),
        }
    }
}

impl IndentDisplay for type_checker::ast::BinaryOperator {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            type_checker::ast::BinaryOperator::Add => "+".to_string(),
            type_checker::ast::BinaryOperator::Subtract => "-".to_string(),
            type_checker::ast::BinaryOperator::Multiply => "*".to_string(),
            type_checker::ast::BinaryOperator::Divide => "/".to_string(),
            type_checker::ast::BinaryOperator::Modulo => "%".to_string(),
            type_checker::ast::BinaryOperator::BitwiseAnd => "&".to_string(),
            type_checker::ast::BinaryOperator::BitwiseOr => "|".to_string(),
            type_checker::ast::BinaryOperator::BitwiseXor => "^".to_string(),
            type_checker::ast::BinaryOperator::BitwiseLeftShift => "<<".to_string(),
            type_checker::ast::BinaryOperator::BitwiseRightShift => ">>".to_string(),
            type_checker::ast::BinaryOperator::LogicalAnd => "&&".to_string(),
            type_checker::ast::BinaryOperator::LogicalOr => "||".to_string(),
            type_checker::ast::BinaryOperator::Equal => "==".to_string(),
            type_checker::ast::BinaryOperator::NotEqual => "!=".to_string(),
            type_checker::ast::BinaryOperator::LessThan => "<".to_string(),
            type_checker::ast::BinaryOperator::LessThanOrEqual => "<=".to_string(),
            type_checker::ast::BinaryOperator::GreaterThan => ">".to_string(),
            type_checker::ast::BinaryOperator::GreaterThanOrEqual => ">=".to_string(),
            type_checker::ast::BinaryOperator::Range => "..".to_string(),
            type_checker::ast::BinaryOperator::RangeInclusive => "..=".to_string(),
        }
    }
}

impl IndentDisplay for type_checker::ast::AccessModifier {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        match self {
            type_checker::ast::AccessModifier::Public => "public".to_string(),
            type_checker::ast::AccessModifier::Module => "module".to_string(),
            type_checker::ast::AccessModifier::Super => "super".to_string(),
        }
    }
}

impl IndentDisplay for type_checker::ast::FieldInitializer {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<field initializer>\n");
        indent.increase();

        if let Some(identifier) = &self.identifier {
            result
                .push_str(format!("{}field initializer: {}\n", indent.dash(), identifier).as_str());
        } else {
            result.push_str(format!("{}field initializer: None\n", indent.dash()).as_str());
        }

        indent.end_current();
        result.push_str(
            format!(
                "{}initializer: {}",
                indent.dash_end(),
                self.initializer.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for type_checker::ast::EnumMemberFieldInitializers {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            type_checker::ast::EnumMemberFieldInitializers::None => "".to_string(),
            type_checker::ast::EnumMemberFieldInitializers::Named(field_initializers) => {
                let mut result = String::new();
                result.push_str("<named field initializer>");
                indent.increase();

                for (i, (identifier, initializer)) in field_initializers.iter().enumerate() {
                    if i < field_initializers.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}{}: {},",
                                indent.dash(),
                                identifier,
                                initializer.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}{}: {}",
                                indent.dash_end(),
                                identifier,
                                initializer.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for TypeIdentifier {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str(format!("<type name>: {}\n", self.to_string()).as_str());

        match self {
            TypeIdentifier::Type(type_name) => {
                indent.increase_leaf();
                result.push_str(format!("{}type: {}", indent.dash_end(), type_name).as_str());
                indent.decrease();
                result
            }
            TypeIdentifier::GenericType(type_name, generics) => {
                indent.increase();
                result.push_str(format!("{}type: {}", indent.dash(), type_name).as_str());
                indent.end_current();
                result.push_str(format!("\n{}generics:", indent.dash_end()).as_str());
                indent.increase();

                for (i, generic) in generics.iter().enumerate() {
                    if i < generics.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}generic: {},",
                                indent.dash(),
                                generic.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}generic: {}",
                                indent.dash_end(),
                                generic.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                indent.decrease();
                result
            }
            TypeIdentifier::ConcreteType(type_name, concrete_types) => {
                indent.increase();
                result.push_str(format!("{}type: {}", indent.dash(), type_name).as_str());
                indent.end_current();
                result.push_str(format!("\n{}concrete_types:", indent.dash_end()).as_str());
                indent.increase();

                for (i, concrete_type) in concrete_types.iter().enumerate() {
                    if i < concrete_types.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}concrete_type: {},",
                                indent.dash(),
                                concrete_type.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}concrete_type: {}",
                                indent.dash_end(),
                                concrete_type.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }

                indent.decrease();
                indent.decrease();
                result
            }
            TypeIdentifier::MemberType(type_identifier, name) => {
                indent.increase();
                result.push_str(
                    format!(
                        "{}type: {}",
                        indent.dash(),
                        type_identifier.indent_display(indent)
                    )
                    .as_str(),
                );
                indent.end_current();
                result.push_str(format!("\n{}member: {}", indent.dash_end(), name).as_str());
                indent.decrease();
                result
            }
        }
    }
}

impl IndentDisplay for GenericType {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<generic type>\n");
        indent.increase_leaf();
        result.push_str(format!("{}type: {}", indent.dash_end(), self.type_name).as_str());
        indent.decrease();
        result
    }
}

impl IndentDisplay for GenericConstraint {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<generic constraint>\n");
        indent.increase_leaf();
        result.push_str(
            format!(
                "{}type: {}\n",
                indent.dash_end(),
                self.generic.indent_display(indent)
            )
            .as_str(),
        );
        result.push_str(format!("{}constraints:", indent.dash_end()).as_str());
        indent.increase();

        for (i, constraint) in self.constraints.iter().enumerate() {
            if i < self.constraints.len() - 1 {
                result.push_str(
                    format!(
                        "\n{}constraint: {},",
                        indent.dash(),
                        constraint.indent_display(indent)
                    )
                    .as_str(),
                );
            } else {
                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}constraint: {}",
                        indent.dash_end(),
                        constraint.indent_display(indent)
                    )
                    .as_str(),
                );
            }
        }

        indent.decrease();
        indent.decrease();
        result
    }
}

impl IndentDisplay for TypeAnnotation {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<type annotation>\n");
        indent.increase_leaf();

        match self {
            TypeAnnotation::Type(type_name) => {
                result.push_str(format!("{}type: {}", indent.dash_end(), type_name).as_str());
            }
            TypeAnnotation::ConcreteType(type_name, generics) => {
                result.push_str(format!("{}concrete_type: {}", indent.dash(), type_name).as_str());

                for (i, generic) in generics.iter().enumerate() {
                    if i < generics.len() - 1 {
                        result.push_str(
                            format!(
                                "\n{}conretes: {},",
                                indent.dash(),
                                generic.indent_display(indent)
                            )
                            .as_str(),
                        );
                    } else {
                        indent.end_current();
                        result.push_str(
                            format!(
                                "\n{}concretes: {}",
                                indent.dash_end(),
                                generic.indent_display(indent)
                            )
                            .as_str(),
                        );
                    }
                }
            }
            TypeAnnotation::Array(type_annotation) => {
                result.push_str(
                    format!(
                        "{}slice_type: {}",
                        indent.dash_end(),
                        type_annotation.indent_display(indent)
                    )
                    .as_str(),
                );
            }
            TypeAnnotation::Literal(literal) => {
                result.push_str(
                    format!(
                        "{}literal: {}",
                        indent.dash_end(),
                        literal.indent_display(indent)
                    )
                    .as_str(),
                );
            }
            TypeAnnotation::Function(param, return_type) => {
                result.push_str(
                    format!(
                        "{}param: {}",
                        indent.dash(),
                        param
                            .into_iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .as_str(),
                );
                result.push_str(
                    format!(
                        "\n{}return_type: {}",
                        indent.dash_end(),
                        return_type.indent_display(indent)
                    )
                    .as_str(),
                );
            }
        }

        indent.decrease();
        result
    }
}

impl IndentDisplay for TypedMatchArm {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<match arm>");
        indent.increase();
        result.push_str(
            format!(
                "\n{}pattern: {}",
                indent.dash(),
                self.pattern.indent_display(indent)
            )
            .as_str(),
        );
        indent.end_current();
        result.push_str(
            format!(
                "\n{}expression: {}",
                indent.dash_end(),
                self.expression.indent_display(indent)
            )
            .as_str(),
        );
        indent.decrease();
        result
    }
}

impl IndentDisplay for Decision {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<decision>");
        indent.increase();

        match self {
            Decision::Success { expression, type_ } => {
                result.push_str(format!(": {}", type_).as_str());
                result.push_str(format!("\n{}variant: Success", indent.dash()).as_str());
                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}expression: {}",
                        indent.dash_end(),
                        expression.indent_display(indent)
                    )
                    .as_str(),
                );
            }
            Decision::Failure { error_message } => {
                result.push_str(format!(": {}", Type::Unknown).as_str());
                result.push_str(format!("\n{}variant: Failure", indent.dash()).as_str());
                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}error_message: {}",
                        indent.dash_end(),
                        error_message.indent_display(indent)
                    )
                    .as_str(),
                );
            }
            Decision::Guard {
                condition,
                consequence,
                alternative,
                type_,
            } => {
                result.push_str(format!(": {}", type_).as_str());
                result.push_str(format!("\n{}variant: Guard", indent.dash()).as_str());
                result.push_str(
                    format!(
                        "\n{}condition: {}",
                        indent.dash(),
                        condition.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}consequence: {}",
                        indent.dash(),
                        consequence.indent_display(indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}alternative: {}",
                        indent.dash_end(),
                        alternative.indent_display(indent)
                    )
                    .as_str(),
                );
            }
            Decision::Switch {
                variable,
                cases,
                fallback,
                type_,
            } => {
                result.push_str(format!(": {}", type_).as_str());
                result.push_str(format!("\n{}variant: Switch", indent.dash()).as_str());
                result.push_str(
                    format!(
                        "\n{}variable: {}",
                        indent.dash(),
                        variable.indent_display(indent)
                    )
                    .as_str(),
                );

                result.push_str(
                    format!(
                        "\n{}{}",
                        indent.dash(),
                        indent_display_vec(cases, "cases", "case", indent)
                    )
                    .as_str(),
                );

                indent.end_current();
                result.push_str(
                    format!(
                        "\n{}fallback: {}",
                        indent.dash_end(),
                        fallback.indent_display(indent)
                    )
                    .as_str(),
                );
            }
        }

        indent.decrease();

        result
    }
}

impl IndentDisplay for Case {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<case>");
        indent.increase();
        result.push_str(
            format!(
                "\n{}pattern: {}",
                indent.dash(),
                self.pattern.indent_display(indent)
            )
            .as_str(),
        );

        result.push_str(
            format!(
                "\n{}{}",
                indent.dash(),
                indent_display_vec(&self.arguments, "arguments", "argument", indent)
            )
            .as_str(),
        );

        indent.end_current();
        result.push_str(
            format!(
                "\n{}decision: {}",
                indent.dash_end(),
                self.body.indent_display(indent)
            )
            .as_str(),
        );

        indent.decrease();

        result
    }
}

impl IndentDisplay for Variable {
    fn indent_display(&self, indent: &mut Indent) -> String {
        let mut result = String::new();
        result.push_str("<variable>");
        result.push_str(format!(": {}", self.type_.full_name()).as_str());

        indent.increase_leaf();
        result.push_str(
            format!(
                "\n{}name: {}",
                indent.dash(),
                self.identifier.indent_display(indent)
            )
            .as_str(),
        );
        result
            .push_str(format!("\n{}type_: {}", indent.dash_end(), self.type_.full_name()).as_str());

        indent.decrease();

        result
    }
}

impl<T: IndentDisplay> IndentDisplay for Option<T> {
    fn indent_display(&self, indent: &mut Indent) -> String {
        match self {
            Some(v) => v.indent_display(indent),
            None => "None".to_string(),
        }
    }
}

impl<T: IndentDisplay> IndentDisplay for Box<T> {
    fn indent_display(&self, indent: &mut Indent) -> String {
        self.as_ref().indent_display(indent)
    }
}

impl<T: IndentDisplay> IndentDisplay for &T {
    fn indent_display(&self, indent: &mut Indent) -> String {
        (*self).indent_display(indent)
    }
}

impl IndentDisplay for String {
    fn indent_display(&self, _indent: &mut Indent) -> String {
        self.clone()
    }
}

fn indent_display_vec<T: IndentDisplay>(
    vec: &Vec<T>,
    parent_type_name: &str,
    item_field_name: &str,
    indent: &mut Indent,
) -> String {
    let mut result = String::new();

    result.push_str(format!("<{}>", parent_type_name).as_str());
    indent.increase();

    for (i, item) in vec.iter().enumerate() {
        if i < vec.len() - 1 {
            result.push_str(
                format!(
                    "\n{}{}: {},",
                    indent.dash(),
                    item_field_name,
                    item.indent_display(indent)
                )
                .as_str(),
            );
        } else {
            indent.end_current();
            result.push_str(
                format!(
                    "\n{}{}: {}",
                    indent.dash_end(),
                    item_field_name,
                    item.indent_display(indent)
                )
                .as_str(),
            );
        }
    }

    indent.decrease();
    result
}
