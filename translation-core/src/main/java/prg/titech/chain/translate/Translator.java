package prg.titech.chain.translate;

import prg.titech.chain.Call;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.ChainVisitor;

import java.util.Optional;

public interface Translator extends ChainVisitor {
    TokenList getTokens();

    default void addToken(String token) {
        getTokens().add(token);
    }

    boolean isMethodAllowed(String method);

    Optional<String> getKeyword(String method);

    Optional<String> getDelimiter(String method);

    @Override
    default void visit(Frame context, Call call) {
        if (context.isFirstPosition()) {
            if (!isMethodAllowed(call.getMethodName())) {
                throw new IllegalArgumentException("Unknown method: " + call.getMethodName());
            }

            getKeyword(call.getMethodName()).ifPresent(this::addToken);
        } else if (context.isMiddlePosition()) {
            getDelimiter(call.getMethodName()).ifPresent(this::addToken);
        }
    }

    @Override
    default void visit(Frame context, StringValue value) {
        addToken(value.toString());
    }
}
