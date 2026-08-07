package prg.titech.chain.value;

import prg.titech.chain.Value;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.GenericVisitor;

public class StringValue implements Value {
    private final String inner;

    public StringValue(String inner) {
        this.inner = inner;
    }

    @Override
    public String toString() {
        return inner;
    }

    @Override
    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    @Override
    public void accept(Frame context, ChainVisitor chainVisitor) {
        chainVisitor.visit(context, this);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }
}
