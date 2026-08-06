package prg.titech.chain.find.java;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.visitor.VoidVisitorAdapter;
import prg.titech.chain.Call;
import prg.titech.chain.Chain;

import java.util.List;

public class JavaChainSearchVisitor extends VoidVisitorAdapter<JavaChainSearchState> {

    public static List<Chain> findChains(Node n) {
        var visitor = new JavaChainSearchVisitor();
        var state = new JavaChainSearchState();
        n.accept(visitor, state);
        return state.getResult();
    }

    @Override
    public void visit(MethodCallExpr method, JavaChainSearchState state) {
        method.getArguments().forEach(e -> {
            state.descent();
            e.accept(this, state);
            state.ascent();
        });
        method.getName().accept(this, state);
        state.incrementRemainingMethods();
        method.getScope().ifPresent(s -> s.accept(this, state));
        state.decrementRemainingMethods();
        method.getTypeArguments().ifPresent(l -> l.forEach(t -> t.accept(this, state)));
        method.getComment().ifPresent(c -> c.accept(this, state));

        state.getCurrent().call(Call.method(method.getNameAsString()).build());
        if (state.noMoreMethodsExpected()) {
            state.completeChain();
        }
    }
}
