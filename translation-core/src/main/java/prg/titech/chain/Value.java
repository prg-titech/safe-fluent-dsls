package prg.titech.chain;

import prg.titech.chain.token.TokenRange;
import prg.titech.chain.value.StringValue;

public interface Value extends Node {

    static Value of(String s) {
        return new StringValue(s);
    }

    static Value of(String s, TokenRange range) {
        return new StringValue(s, range);
    }

}
