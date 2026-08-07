package prg.titech.chain;

import prg.titech.chain.builder.ChainBuilder;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.PrettyPrintVisitor;

import java.util.List;

public class Chain implements Value {
    protected final List<Call> calls;

    public Chain(List<Call> calls) {
        this.calls = calls;
    }

    public static ChainBuilder builder() {
        return new ChainBuilder();
    }

    public List<Call> getCalls() {
        return calls;
    }

    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    public void accept(Frame context, ChainVisitor visitor) {
        visitor.visit(context, this);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }

    @Override
    public String toString() {
        return PrettyPrintVisitor.prettyPrint(this);
    }
}
