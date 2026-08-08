package prg.titech.chain.value;

import com.github.javaparser.ast.expr.Expression;
import jakarta.annotation.Nonnull;
import prg.titech.chain.Value;
import prg.titech.chain.token.TokenRange;
import prg.titech.chain.visit.GenericVisitor;
import prg.titech.chain.visit.VoidVisitor;

import java.util.Optional;

public record JavaExprValue(Expression inner) implements Value {
    @Override
    public <S> void accept(VoidVisitor<S> visitor, S state) {
        visitor.visit(this, state);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }

    @Override
    public @Nonnull String toString() {
        return inner.toString();
    }

    @Override
    public Optional<TokenRange> getTokenRange() {
        return inner.getTokenRange().map(TokenRange::from);
    }
}
