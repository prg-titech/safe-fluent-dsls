package prg.titech.chain;

import jakarta.annotation.Nullable;
import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.token.Token;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.Visitable;
import prg.titech.chain.visit.VoidVisitor;

import java.util.List;

public class Call implements Visitable {
    private final Name methodName;
    protected final List<Value> parameters;

    public Call(Name methodName, List<Value> parameters) {
        this.methodName = methodName;
        this.parameters = parameters;
    }

    public static CallBuilder method(String methodName, @Nullable Token sourceToken) {
        return new CallBuilder(methodName, sourceToken == null ? List.of() : List.of(sourceToken));
    }

    public static CallBuilder method(String methodName, List<Token> sourceTokens) {
        return new CallBuilder(methodName, sourceTokens);
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
    public <S> void accept(VoidVisitor<S> visitor, S state) {
        visitor.visit(this, state);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }
}
