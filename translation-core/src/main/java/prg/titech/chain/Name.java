package prg.titech.chain;

import jakarta.annotation.Nullable;
import prg.titech.chain.token.TokenRange;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.VoidVisitor;

import java.util.Optional;

public class Name implements Node {
    private final String name;
    private final @Nullable TokenRange range;

    public Name(String name, @Nullable TokenRange range) {
        this.name = name;
        this.range = range;
    }

    public Name(String name) {
        this(name, null);
    }

    @Override
    public Optional<TokenRange> getTokenRange() {
        return Optional.ofNullable(range);
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
    public String toString() {
        return name;
    }
}
