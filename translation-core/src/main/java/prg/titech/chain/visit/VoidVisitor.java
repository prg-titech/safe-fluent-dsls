package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Name;
import prg.titech.chain.value.JavaExprValue;
import prg.titech.chain.value.StringValue;

public interface VoidVisitor<S> {
    void visit(Chain chain, S state);

    void visit(Call call, S state);

    void visit(Name name, S state);

    void visit(StringValue value, S state);

    void visit(JavaExprValue value, S state);
}
