package prg.titech.chain.builder;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;

import java.util.ArrayList;
import java.util.List;

public class ChainBuilder {
    private final List<Call> calls = new ArrayList<>();

    public ChainBuilder call(Call c) {
        calls.add(c);
        return this;
    }

    public Chain build() {
        return new Chain(calls);
    }

    public List<Call> getCalls() {
        return calls;
    }
}
