package prg.titech.chain.translate;

import prg.titech.chain.Call;
import prg.titech.chain.Name;
import prg.titech.chain.Value;
import prg.titech.chain.token.Token;
import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.VoidVisitorAdaptor;

import java.util.List;


public abstract class Translator extends VoidVisitorAdaptor<Translation> {
    protected String currentMethod;
    protected String layoutCharacter = " ";

    protected void addTokenIfNonEmpty(List<Token> sourceTokens, String token, Translation state) {
        if (!token.isEmpty()) {
            addToken(sourceTokens, token, state);
        }
    }

    protected void addToken(List<Token> sourceTokens, String token, Translation state) {
        state.addToken(sourceTokens, token);
        if (!layoutCharacter.isEmpty()) {
            state.addToken(null, layoutCharacter);
        }
    }

    public abstract boolean isMethodAllowed(String method);

    public abstract String getKeyword(String method);

    public abstract String getDelimiter(String method);

    public abstract String getQuoteDelimiter(String method);

    @Override
    public void visit(Call call, Translation state) {
        call.getMethodName().accept(this, state);
        boolean addDelimiter = false;
        for (Value value : call.getParameters()) {
            if (addDelimiter) {
                addTokenIfNonEmpty(null, getDelimiter(currentMethod), state);
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

        addTokenIfNonEmpty(name.getSourceTokens(), getKeyword(name.toString()), state);
    }

    @Override
    public void visit(StringValue value, Translation state) {
        String quoteChar = getQuoteDelimiter(currentMethod);
        if (quoteChar.isEmpty()) {
            addToken(value.getSourceTokens(), value.toString(), state);
        } else {
            String currentLayoutChar = layoutCharacter;
            layoutCharacter = "";
            addToken(null, quoteChar, state);
            addToken(value.getSourceTokens(), value.toString(), state);
            layoutCharacter = currentLayoutChar;
            addToken(null, quoteChar, state);
        }
    }
}
