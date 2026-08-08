package prg.titech.chain.value;

import jakarta.annotation.Nullable;
import prg.titech.chain.Value;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.token.TokenRange;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.VoidVisitor;

import java.util.Optional;

public class StringValue implements Value {
    private final String inner;
    private @Nullable TokenRange range;

    public StringValue(String inner, @Nullable TokenRange range) {
        this.inner = inner;
        this.range = range;
    }

    public StringValue(String inner) {
        this(inner, null);
    }

    @Override
    public String toString() {
        return inner;
    }

    public String toQuotedString(CharSequence quoteSymbol) {
        return quoteSymbol + inner.replace(quoteSymbol, "\\" + quoteSymbol) + quoteSymbol;
    }

    @Override
    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    @Override
    public void accept(Frame context, ChainVisitor chainVisitor) {
        chainVisitor.visit(context, this);
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
    public Optional<TokenRange> getTokenRange() {
        return Optional.ofNullable(range);
    }

    public void setTokenRange(@Nullable TokenRange range) {
        this.range = range;
    }
}
