package prg.titech.chain.value;

import prg.titech.chain.Value;
import prg.titech.chain.visit.ChainVisitor;

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
    public void traverse(ChainVisitor chainVisitor) {
        chainVisitor.visit((Value) this);
        chainVisitor.visit(this);
        chainVisitor.endVisit(this);
        chainVisitor.endVisit((Value) this);
    }
}
