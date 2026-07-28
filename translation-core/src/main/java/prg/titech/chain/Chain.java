package prg.titech.chain;

import prg.titech.chain.builder.ChainBuilder;
import prg.titech.chain.visit.ChainVisitor;
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

    @Override
    public void traverse(ChainVisitor visitor) {
        visitor.visit((Value) this);
        visitor.visit(this);
        for (Call call : calls) {
            call.traverse(visitor);
        }
        visitor.endVisit(this);
        visitor.endVisit((Value) this);
    }

    @Override
    public String toString() {
        return PrettyPrintVisitor.prettyPrint(this);
    }
}
