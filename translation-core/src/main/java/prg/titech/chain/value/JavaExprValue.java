package prg.titech.chain.value;

import com.github.javaparser.ast.expr.Expression;
import prg.titech.chain.Value;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.visit.ChainVisitor;

public class JavaExprValue implements Value {
    private final Expression inner;

    public JavaExprValue(Expression inner) {
        this.inner = inner;
    }

    @Override
    public void accept(ChainTraverser traverser) {
        traverser.traverse(this);
    }

    @Override
    public void accept(Frame context, ChainVisitor visitor) {
        visitor.visit(context, this);
    }

    @Override
    public String toString() {
        return inner.toString();
    }
}
