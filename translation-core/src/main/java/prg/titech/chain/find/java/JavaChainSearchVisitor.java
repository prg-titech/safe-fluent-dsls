package prg.titech.chain.find.java;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.visitor.VoidVisitorAdapter;
import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.value.JavaExprValue;

import java.util.List;

public class JavaChainSearchVisitor extends VoidVisitorAdapter<JavaChainSearchState> {

    private JavaChainSearchVisitor() {}

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

        CallBuilder currentCall = Call.method(method.getNameAsString());
        for (Expression e : method.getArguments()) {
            currentCall.arg(new JavaExprValue(e));
        }
        state.getCurrent().call(currentCall.build());
        if (state.noMoreMethodsExpected()) {
            state.completeChain();
        }
    }
}
