package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Name;
import prg.titech.chain.Value;
import prg.titech.chain.value.JavaExprValue;
import prg.titech.chain.value.StringValue;

public class VoidVisitorAdaptor<S> implements VoidVisitor<S> {
    @Override
    public void visit(Chain chain, S state) {
        for (Call call : chain.getCalls()) {
            call.accept(this, state);
        }
    }

    @Override
    public void visit(Call call, S state) {
        call.getMethodName().accept(this, state);
        for (Value v : call.getParameters()) {
            v.accept(this, state);
        }
    }

    @Override
    public void visit(Name name, S state) {}

    @Override
    public void visit(StringValue value, S state) {}

    @Override
    public void visit(JavaExprValue value, S state) {}
}
