package prg.titech.chain.value;

import com.github.javaparser.ast.expr.Expression;
import jakarta.annotation.Nonnull;
import prg.titech.chain.Value;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.GenericVisitor;

public record JavaExprValue(Expression inner) implements Value {

    @Override
    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    @Override
    public void accept(Frame context, ChainVisitor visitor) {
        visitor.visit(context, this);
    }

    @Override
    public <R, S> R accept(GenericVisitor<R, S> visitor, S state) {
        return visitor.visit(this, state);
    }

    @Override
    public @Nonnull String toString() {
        return inner.toString();
    }
}
