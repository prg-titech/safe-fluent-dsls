package prg.titech.chain.builder;

import prg.titech.chain.Call;
import prg.titech.chain.Name;
import prg.titech.chain.Value;
import prg.titech.chain.token.Token;

import java.util.ArrayList;
import java.util.List;

public class CallBuilder {
    private final Name methodName;
    private final List<Value> parameters;

    public CallBuilder(String methodName, List<Token> sourceTokens) {
        this.methodName = new Name(methodName, sourceTokens);
        this.parameters = new ArrayList<>();
    }

    public CallBuilder(String methodName) {
        this(methodName, null);
    }

    public CallBuilder arg(Value v) {
        parameters.add(v);
        return this;
    }

    public CallBuilder arg(String s) {
        parameters.add(Value.of(s));
        return this;
    }

    public Call build() {
        return new Call(methodName, parameters);
    }
}
