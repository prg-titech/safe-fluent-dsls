package prg.titech.chain;

import prg.titech.chain.builder.ChainBuilder;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.PrettyPrintVisitor;
import prg.titech.chain.visit.VoidVisitor;

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

    @Override
    public <S> void accept(VoidVisitor<S> visitor, S state) {
        visitor.visit(this, state);
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
