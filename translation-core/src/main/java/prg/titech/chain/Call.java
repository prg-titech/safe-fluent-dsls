package prg.titech.chain;

import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.Visitable;

import java.util.List;

public class Call implements Visitable {
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
    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    @Override
    public void accept(Frame context, ChainVisitor visitor) {
        visitor.visit(context, this);
    }
}
