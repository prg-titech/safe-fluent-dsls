package prg.titech.chain.find.java;

import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.expr.StringLiteralExpr;
import prg.titech.chain.Chain;
import prg.titech.chain.token.TokenRange;
import prg.titech.chain.value.JavaExprValue;
import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.ModifyingVisitor;
import prg.titech.chain.visit.Visitable;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicBoolean;

public class JavaExprReplaceVisitor extends ModifyingVisitor<AtomicBoolean> {

    public static <V extends Visitable> Optional<V> replaceIn(V v) {
        JavaExprReplaceVisitor self = new JavaExprReplaceVisitor();
        AtomicBoolean isValid = new AtomicBoolean(true);
        V result = (V) v.accept(self, isValid);
        if (isValid.get()) {
            return Optional.of(result);
        } else {
            return Optional.empty();
        }
    }

    @Override
    public Visitable visit(JavaExprValue value, AtomicBoolean isValid) {
        Expression inner = value.inner();
        if (inner.isStringLiteralExpr()) {
            StringLiteralExpr realInner = inner.asStringLiteralExpr();
            return new StringValue(realInner.asString(), realInner.getTokenRange().map(TokenRange::from).orElse(null));
        } else if (inner.isMethodCallExpr()) {
            Chain subChain = JavaChainSearcher.expectChain(inner);
            Optional<Chain> replacedSubChain = JavaExprReplaceVisitor.replaceIn(subChain);
            if (replacedSubChain.isPresent()) {
                return replacedSubChain.get();
            }
        }
        isValid.set(false);
        return value;
    }
}
