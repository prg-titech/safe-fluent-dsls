package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.value.StringValue;

@SuppressWarnings("unused") public interface ChainVisitor {
    default void visit(Frame context, Chain chain) {}

    default void visit(Frame context, Call call) {}

    default void visit(Frame context, StringValue value) {}
}
