package prg.titech.chain;

import prg.titech.chain.token.Token;
import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.Visitable;

import java.util.List;

public interface Value extends Visitable {

    static Value of(String s) {
        return new StringValue(s);
    }

    static Value of(String s, List<Token> sourceTokens) {
        return new StringValue(s, sourceTokens);
    }

}
