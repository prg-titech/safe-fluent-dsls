package prg.titech.chain;

import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.Visitable;

public interface Value extends Visitable {

    static Value of(String s) {
        return new StringValue(s);
    }

}
