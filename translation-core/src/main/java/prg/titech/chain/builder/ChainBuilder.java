package prg.titech.chain.builder;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;

import java.util.ArrayList;

public class ChainBuilder extends Chain {

    public ChainBuilder() {
        super(new ArrayList<>());
    }

    public ChainBuilder call(Call c) {
        calls.add(c);
        return this;
    }

    public Chain build() {
        return this;
    }
}
