package prg.titech.chain.builder;

import jakarta.annotation.Nullable;
import prg.titech.chain.Call;
import prg.titech.chain.Name;
import prg.titech.chain.Value;
import prg.titech.chain.token.TokenRange;

import java.util.ArrayList;
import java.util.List;

public class CallBuilder {
    private final Name methodName;
    private final List<Value> parameters;
    private @Nullable TokenRange range;

    public CallBuilder(String methodName) {
        this.methodName = new Name(methodName);
        this.parameters = new ArrayList<>();
    }

    public CallBuilder arg(Value v) {
        parameters.add(v);
        return this;
    }

    public CallBuilder arg(String s) {
        parameters.add(Value.of(s));
        return this;
    }

    public CallBuilder range(TokenRange range) {
        this.range = range;
        return this;
    }

    public Call build() {
        return new Call(methodName, parameters, range);
    }
}
