package prg.titech.chain.builder;

import prg.titech.chain.Call;
import prg.titech.chain.Value;

import java.util.ArrayList;

public class CallBuilder extends Call {

    public CallBuilder(String methodName) {
        super(methodName, new ArrayList<>());
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
        return this;
    }
}
