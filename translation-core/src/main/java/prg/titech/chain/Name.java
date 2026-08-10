package prg.titech.chain;

import prg.titech.chain.token.Token;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.VoidVisitor;

import java.util.List;

public class Name implements Leaf {
    private final String name;
    private final List<Token> sourceTokens;

    public Name(String name, List<Token> sourceTokens) {
        this.name = name;
        this.sourceTokens = sourceTokens;
    }

    public Name(String name) {
        this(name, List.of());
    }

    @Override
    public List<Token> getSourceTokens() {
        return sourceTokens;
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
