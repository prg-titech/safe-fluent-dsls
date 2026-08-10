package prg.titech.chain.value;

import jakarta.annotation.Nullable;
import prg.titech.chain.Leaf;
import prg.titech.chain.Value;
import prg.titech.chain.token.Token;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.VoidVisitor;

import java.util.List;

public class StringValue implements Value, Leaf {
    private final String inner;
    private List<Token> sourceTokens;

    public StringValue(String inner, @Nullable Token sourceToken) {
        this(inner, sourceToken == null ? List.of() : List.of(sourceToken));
    }

    public StringValue(String inner, List<Token> sourceTokens) {
        this.inner = inner;
        this.sourceTokens = sourceTokens;
    }

    public StringValue(String inner) {
        this.inner = inner;
        this.sourceTokens = List.of();
    }

    @Override
    public String toString() {
        return inner;
    }

    public String toQuotedString(CharSequence quoteSymbol) {
        return quoteSymbol + inner.replace(quoteSymbol, "\\" + quoteSymbol) + quoteSymbol;
    }

    @Override
    public <S> void accept(VoidVisitor<S> visitor, S state) {
        visitor.visit(this, state);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }

    @Override
    public List<Token> getSourceTokens() {
        return sourceTokens;
    }

}
