package prg.titech.chain.translate;

import prg.titech.chain.Call;
import prg.titech.chain.Name;
import prg.titech.chain.Value;
import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.VoidVisitorAdaptor;

import java.util.Optional;

public abstract class Translator extends VoidVisitorAdaptor<Translation> {
    protected String currentMethod;

    protected void addTokenIfPresent(Optional<String> token, Translation state) {
        token.ifPresent(t -> state.addToken(null, t));
    }

    public abstract boolean isMethodAllowed(String method);

    public abstract Optional<String> getKeyword(String method);

    public abstract Optional<String> getDelimiter(String method);

    public abstract Optional<String> getQuoteDelimiter(String method);

    @Override
    public void visit(Call call, Translation state) {
        call.getMethodName().accept(this, state);
        boolean addDelimiter = false;
        for (Value value : call.getParameters()) {
            if (addDelimiter) {
                addTokenIfPresent(getDelimiter(currentMethod), state);
            }
            value.accept(this, state);

            addDelimiter = true;
        }
    }

    @Override
    public void visit(Name name, Translation state) {
        if (!isMethodAllowed(name.toString())) {
            throw new IllegalArgumentException("Unknown method: " + name);
        }
        currentMethod = name.toString();

        addTokenIfPresent(getKeyword(name.toString()), state);
    }

    @Override
    public void visit(StringValue value, Translation state) {
        addTokenIfPresent(getQuoteDelimiter(currentMethod), state);
        state.addToken(null, value.toString());
        addTokenIfPresent(getQuoteDelimiter(currentMethod), state);
    }
}
