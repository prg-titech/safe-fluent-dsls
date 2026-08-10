package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Name;
import prg.titech.chain.value.JavaExprValue;
import prg.titech.chain.value.StringValue;

public interface GenericVisitor<R, S> {
    R visit(Chain chain, S state);

    R visit(Call call, S state);

    R visit(Name name, S state);

    R visit(StringValue value, S state);

    R visit(JavaExprValue value, S state);
}
