package prg.titech.chain.builder;

import jakarta.annotation.Nullable;
import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.token.TokenRange;

import java.util.ArrayList;
import java.util.List;

public class ChainBuilder {
    private final List<Call> calls = new ArrayList<>();
    private @Nullable TokenRange range;

    public ChainBuilder call(Call c) {
        calls.add(c);
        return this;
    }

    public ChainBuilder range(TokenRange range) {
        this.range = range;
        return this;
    }

    public Chain build() {
        return new Chain(calls, range);
    }

    public List<Call> getCalls() {
        return calls;
    }
}
