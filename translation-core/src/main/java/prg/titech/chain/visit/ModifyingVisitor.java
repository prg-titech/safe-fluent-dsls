package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Name;
import prg.titech.chain.Value;
import prg.titech.chain.token.TokenRange;
import prg.titech.chain.value.JavaExprValue;
import prg.titech.chain.value.StringValue;

import java.util.List;
import java.util.Objects;

public class ModifyingVisitor<S> implements GenericVisitor<Visitable, S> {
    @Override
    public Visitable visit(Chain chain, S state) {
        List<Call> calls = modifyList(chain.getCalls(), state);
        TokenRange range = chain.getTokenRange().orElse(null);
        return new Chain(calls, range);
    }

    @Override
    public Visitable visit(Call call, S state) {
        Name methodName = (Name) call.getMethodName().accept(this, state);
        List<Value> parameters = modifyList(call.getParameters(), state);
        TokenRange range = call.getTokenRange().orElse(null);
        return new Call(methodName, parameters, range);
    }

    @Override
    public Visitable visit(Name name, S state) {
        return name;
    }

    @Override
    public Visitable visit(StringValue value, S state) {
        return value;
    }

    @Override
    public Visitable visit(JavaExprValue value, S state) {
        return value;
    }

    private <V extends Visitable> List<V> modifyList(List<V> list, S state) {
        return list.stream().map(v -> (V) v.accept(this, state))
                .filter(Objects::nonNull)
                .toList();
    }
}
