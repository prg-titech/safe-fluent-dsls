package prg.titech.chain.find.java;

import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.MethodCallExpr;
import com.github.javaparser.ast.visitor.VoidVisitorAdapter;
import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.builder.CallBuilder;
import prg.titech.chain.token.Token;
import prg.titech.chain.token.TokenRange;
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

        CallBuilder currentCall = Call
                .method(method.getNameAsString(), method.getName().getTokenRange().map(TokenRange::from).orElse(null));

        // The token range of a call is NOT equal to method#getTokenRange(), because that would include the entire
        // expression before the "."!
        Token callBeginToken = method.getName().getTokenRange().map(r -> Token.from(r.getBegin())).orElse(null);
        Token callEndToken = method.getTokenRange().map(r -> Token.from(r.getEnd())).orElse(null);
        if (callBeginToken != null && callEndToken != null) {
            currentCall.range(new TokenRange(callBeginToken, callEndToken));
        }

        for (Expression e : method.getArguments()) {
            currentCall.arg(new JavaExprValue(e));
        }
        state.getCurrent().call(currentCall.build());
        if (state.noMoreMethodsExpected()) {
            state.completeChain();
        }
    }
}
