package prg.titech.chain;

import jakarta.annotation.Nullable;
import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.token.TokenRange;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.GenericVisitor;

import java.util.List;
import java.util.Optional;

public class Call implements Node {
    private final Name methodName;
    protected final List<Value> parameters;
    private final @Nullable TokenRange range;

    public Call(Name methodName, List<Value> parameters, @Nullable TokenRange range) {
        this.methodName = methodName;
        this.parameters = parameters;
        this.range = range;
    }

    public Call(Name methodName, List<Value> parameters) {
        this(methodName, parameters, null);
    }

    public static CallBuilder method(String methodName, TokenRange range) {
        return new CallBuilder(methodName, range);
    }

    public static CallBuilder method(String methodName) {
        return new CallBuilder(methodName);
    }

    public Name getMethodName() {
        return methodName;
    }

    @SuppressWarnings("unused") public List<Value> getParameters() {
        return parameters;
    }

    @Override
    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    @Override
    public void accept(Frame context, ChainVisitor visitor) {
        visitor.visit(context, this);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }

    @Override
    public Optional<TokenRange> getTokenRange() {
        return Optional.ofNullable(range);
    }
}
