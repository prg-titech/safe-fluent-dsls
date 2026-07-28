package prg.titech.chain;

import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.Traversable;

public interface Value extends Traversable<ChainVisitor> {

    static Value of(String s) {
        return new StringValue(s);
    }
}
