package prg.titech.chain;

import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.Traversable;

import java.util.List;

public class Call implements Traversable<ChainVisitor> {
    private final String methodName;
    protected final List<Value> parameters;

    public Call(String methodName, List<Value> parameters) {
        this.methodName = methodName;
        this.parameters = parameters;
    }

    public static CallBuilder method(String methodName) {
        return new CallBuilder(methodName);
    }

    public String getMethodName() {
        return methodName;
    }

    @SuppressWarnings("unused") public List<Value> getParameters() {
        return parameters;
    }

    @Override
    public void traverse(ChainVisitor visitor) {
        visitor.visit(this);
        for (Value parameter : parameters) {
            parameter.traverse(visitor);
        }
        visitor.endVisit(this);
    }
}
