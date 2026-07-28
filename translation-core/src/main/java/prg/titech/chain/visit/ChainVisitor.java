package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Value;
import prg.titech.chain.value.StringValue;

@SuppressWarnings("unused") public interface ChainVisitor {
    default void visit(Chain chain) {}

    default void endVisit(Chain chain) {}

    default void visit(Call call) {}

    default void endVisit(Call call) {}

    default void visit(Value value) {}

    default void endVisit(Value value) {}

    default void visit(StringValue value) {}

    default void endVisit(StringValue value) {}
}
