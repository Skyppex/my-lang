﻿
namespace MyLang;

public class TypeChecker
{
    private static readonly Dictionary<string, List<Type>> s_operandTypesForOperator = new()
    {
        ["+"] = new List<Type> { Type.i32, Type.f32, Type.@string },
        ["-"] = new List<Type> { Type.i32, Type.f32 },
        ["*"] = new List<Type> { Type.i32, Type.f32 },
        ["/"] = new List<Type> { Type.i32, Type.f32 },
        ["%"] = new List<Type> { Type.i32, Type.f32 },
        ["&"] = new List<Type> { Type.@bool },
        ["|"] = new List<Type> { Type.@bool },
        ["^"] = new List<Type> { Type.@bool },
        ["~"] = new List<Type> { Type.i32 },
        ["<<"] = new List<Type> { Type.i32 },
        [">>"] = new List<Type> { Type.i32 },
        ["=="] = new List<Type> { Type.i32, Type.f32, Type.@string, Type.@bool },
        ["!="] = new List<Type> { Type.i32, Type.f32, Type.@string, Type.@bool },
        ["<"] = new List<Type> { Type.i32, Type.f32 },
        [">"] = new List<Type> { Type.i32, Type.f32 },
        ["<="] = new List<Type> { Type.i32, Type.f32 },
        [">="] = new List<Type> { Type.i32, Type.f32 },
        ["&&"] = new List<Type> { Type.@bool },
        ["||"] = new List<Type> { Type.@bool },
        ["!"] = new List<Type> { Type.@bool },
    };

    public Type CheckType(IStatement statement, TypeEnvironment typeEnvironment)
    {
        switch (statement)
        {
            case Int32Literal:
                return Type.i32;
            
            case Float32Literal:
                return Type.f32;
            
            case StringLiteral:
                return Type.@string;
            
            case BooleanLiteral:
                return Type.@bool;

            case StructLiteral structLiteral:
            {
                if (!typeEnvironment.Lookup(structLiteral.Identifier, out Type? type))
                    throw new Exception($"Struct with name '{structLiteral.Identifier}' is not defined.");

                List<Type> fieldTypes = type!.Fields.Values.ToList();
                
                for (int i = 0; i < structLiteral.FieldInitializers.Count; i++)
                {
                    StructLiteral.FieldInitializer fieldInitializer = structLiteral.FieldInitializers[i];
                    
                    Type initializerType = CheckType(fieldInitializer.Initializer, typeEnvironment);

                    if (fieldTypes.Count < i)
                        throw new Exception($"Struct '{type.Name}' does not contain {i + 1} fields.");

                    if (fieldTypes[i] != initializerType)
                        throw new Exception($"Initializer for field '{type.Fields.Keys.ToList()[i]}' is not of type '{fieldTypes[i]}'.");
                }

                return type;
            }
            
            case UnionLiteral unionLiteral:
            {
                if (!typeEnvironment.Lookup(unionLiteral.Identifier, out Type? type))
                    throw new Exception($"Union with name '{unionLiteral.Identifier}' is not defined.");

                List<Type> fieldTypes = type!.Fields[unionLiteral.Member].Fields.Values.ToList();
                
                for (int i = 0; i < unionLiteral.FieldInitializers.Count; i++)
                {
                    UnionLiteral.FieldInitializer fieldInitializer = unionLiteral.FieldInitializers[i];
                    
                    Type initializerType = CheckType(fieldInitializer.Initializer, typeEnvironment);

                    if (fieldTypes.Count < i)
                        throw new Exception($"Union member '{unionLiteral.Identifier}{TokenSymbol.MEMBER_ACCESSOR}{unionLiteral.Member}' does not contain {i + 1} fields.");

                    if (fieldTypes[i] != initializerType)
                        throw new Exception($"Initializer for field '{type.Fields.Keys.ToList()[i]}' is not of type '{fieldTypes[i]}'.");
                }

                return type;
            }
            
            case UnaryExpression unaryExpression:
            {
                Type type = CheckType(unaryExpression.Operand, typeEnvironment);

                if (type != Type.i32 && type != Type.f32 && type != Type.@bool)
                    throw new InvalidOperationException($"Unary operator {unaryExpression.Operator} not supported for operand of type '{type}'");
                
                return type;
            }
            
            case TernaryExpression ternaryExpression:
            {
                Type conditionType = CheckType(ternaryExpression.Condition, typeEnvironment);
                Type trueType = CheckType(ternaryExpression.Then, typeEnvironment);
                Type falseType = CheckType(ternaryExpression.Else, typeEnvironment);

                if (conditionType != Type.@bool)
                    throw new InvalidOperationException($"Ternary operator condition must be of type '{Type.@bool}' but was '{conditionType}'");
                
                if (trueType != falseType)
                    throw new InvalidOperationException($"Ternary operator true and false expressions must be of the same type but were '{trueType}' and '{falseType}'");
                
                return trueType;
            }
            
            case DropExpression dropExpression:
            {
                return CheckType(dropExpression.Identifier, typeEnvironment);
            }
            
            case BinaryExpression binaryExpression:
            {
                Type leftType = CheckType(binaryExpression.Left, typeEnvironment);
                Type rightType = CheckType(binaryExpression.Right, typeEnvironment);

                if (leftType != rightType)
                    throw new InvalidOperationException($"Operator '{binaryExpression.Operator}' not supported for operands of type '{leftType}' and '{rightType}'");

                List<Type> allowedTypes = s_operandTypesForOperator[binaryExpression.Operator];

                if (!allowedTypes.Contains(leftType) || !allowedTypes.Contains(rightType))
                {
                    throw new InvalidOperationException(
                        $"Operator {binaryExpression.Operator} not supported for operands of type '{leftType}' and '{rightType}'");
                }

                switch (binaryExpression.Operator)
                {
                    case TokenSymbol.LOGICAL_AND or TokenSymbol.LOGICAL_OR:
                    {
                        if (leftType != Type.@bool || rightType != Type.@bool)
                        {
                            throw new InvalidOperationException(
                                $"Operator {binaryExpression.Operator} not supported for operands of type '{leftType}' and '{rightType}'");
                        }

                        return Type.@bool;
                    }
                    
                    case TokenSymbol.LOGICAL_EQUAL or TokenSymbol.LOGICAL_NOT_EQUAL or TokenSymbol.GREATER or TokenSymbol.LESS or TokenSymbol.LOGICAL_GREATER_EQUAL or TokenSymbol.LOGICAL_LESS_EQUAL:
                    {
                        if (!Type.Numbers.Contains(leftType) || !Type.Numbers.Contains(rightType))
                        {
                            throw new InvalidOperationException(
                                $"Operator {binaryExpression.Operator} not supported for operands of type '{leftType}' and '{rightType}'");
                        }

                        return Type.@bool;
                    }
                    
                    case TokenSymbol.ADD or TokenSymbol.SUBTRACT or TokenSymbol.MULTIPLY or TokenSymbol.DIVIDE
                     or TokenSymbol.MODULO:
                    {
                        switch (leftType, rightType)
                        {
                            case var (l, r) when (l, r) == (Type.i32, Type.i32):
                                return Type.i32;

                            case var (l, r) when (l, r) == (Type.f32, Type.f32):
                                return Type.f32;

                            case var (l, r) when (l, r) == (Type.@string, Type.@string):
                                return Type.@string;
                            
                            case var (l, r) when (l, r) == (Type.@bool, Type.@bool):
                                return Type.@bool;
                        }

                        break;
                    }
                }

                break;
            }
            
            case Identifier identifier:
            {
                if (typeEnvironment.Lookup(identifier.Symbol, out Type? type))
                    return type;
                
                throw new InvalidOperationException($"Identifier '{identifier.Symbol}' doesn't exist in current context.");
            }
            
            case VariableDeclarationExpression variableDeclarationExpression:
            {
                Type type = CheckType(variableDeclarationExpression.Initializer, typeEnvironment);

                if (!typeEnvironment.Lookup(variableDeclarationExpression.TypeName, out Type? expectedType))
                    throw new InvalidOperationException(
                        $"Type '{variableDeclarationExpression.TypeName}' doesn't exist in current context.");
                
                if (type != expectedType)
                {
                    throw new InvalidOperationException(
                        $"Variable {variableDeclarationExpression.Identifier.Symbol} has type '{expectedType}' but was assigned '{type}'");
                }
                
                return typeEnvironment.Define(variableDeclarationExpression.Identifier.Symbol, type);
            }
            
            case VariableDeclarationStatement variableDeclarationStatement:
            {
                if (!typeEnvironment.Lookup(variableDeclarationStatement.TypeName, out Type? type))
                    throw new InvalidOperationException(
                        $"Type '{variableDeclarationStatement.TypeName}' doesn't exist in current context.");
                
                return typeEnvironment.Define(variableDeclarationStatement.Identifier.Symbol, type!);
            }

            case AssignmentExpression assignmentExpression:
            {
                Type type = CheckType(assignmentExpression.Assignment, typeEnvironment);
                
                if (!typeEnvironment.Lookup(assignmentExpression.Identifier.Symbol, out Type? variableType))
                    throw new InvalidOperationException(
                        $"Identifier '{assignmentExpression.Identifier.Symbol}' doesn't exist in current context.");
                
                if (type != variableType)
                    throw new InvalidOperationException(
                        $"Variable {assignmentExpression.Identifier.Symbol} has type '{variableType}' but was assigned '{type}'");
                
                return type;
            }

            case BlockExpression blockExpression:
            {
                return CheckType(blockExpression.Expression, typeEnvironment);
            }

            case StructDeclarationStatement structDeclarationStatement:
            {
                if (typeEnvironment.IsDefined(structDeclarationStatement.TypeName))
                    throw new Exception($"Type '{structDeclarationStatement.TypeName}' already exists in current context.");

                structDeclarationStatement.Fields.ToList()
                    .ForEach(f =>
                    {
                        if (typeEnvironment.IsDefined(f.TypeName))
                            return;

                        throw new Exception($"Type '{f.TypeName}' doesn't exist in current context.");
                    });
                
                return Type.statement;
            }

            case UnionDeclarationStatement unionDeclarationStatement:
            {
                if (typeEnvironment.IsDefined(unionDeclarationStatement.TypeName))
                    throw new Exception($"Type '{unionDeclarationStatement.TypeName}' already exists in current context.");
                
                unionDeclarationStatement.Members.ToList()
                    .ForEach(m =>
                    {
                        m.Fields.ForEach(f =>
                        {
                            if (typeEnvironment.IsDefined(f.TypeName))
                                return;
                            
                            throw new Exception($"Type '{f.TypeName}' doesn't exist in current context.");
                        });
                    });
                
                return Type.statement;
            }
            
            case Program:
            case IfStatement:
                return Type.statement;
        }

        throw new InvalidOperationException($"Cannot check type of {statement.GetType()}");
    }
}