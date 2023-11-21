﻿namespace MyLang.Runtime;

public interface IRuntimeValue { }

public abstract class NumberValue : IRuntimeValue
{
    public abstract object Boxed { get; }
}

public sealed class Uninitialized : IRuntimeValue
{
    public static readonly Uninitialized Instance = new();
    private Uninitialized() { }
}

public sealed class NoValue : IRuntimeValue
{
    public static readonly NoValue Instance = new();
    private NoValue() { }
}

public class Int32Value : NumberValue
{
    public Int32Value(int value) => Value = value;
    public int Value { get; }
    public override object Boxed => Value;
    public override string ToString() => Value.ToString();
}

public class Float32Value : NumberValue
{
    public Float32Value(float value) => Value = value;
    public float Value { get; }
    public override object Boxed => Value;
    public override string ToString() => Value.ToString();
}

public class StringValue : IRuntimeValue
{
    public StringValue(string value) => Value = value;
    public string Value { get; init; }
    public override string ToString() => Value;
}

public class BooleanValue : IRuntimeValue
{
    public BooleanValue(bool value) => Value = value;
    public bool Value { get; init; }
    public override string ToString() => Value.ToString();
}

public class EmptyProgramValue : IRuntimeValue { }