package prg.titech.chain.projection;

import prg.titech.chain.token.Token;

public class ParseError implements Projection {
    private final Token sourceToken;
    private final Token targetToken;
    private final String message;

    public ParseError(Token sourceToken, Token targetToken, String message) {
        this.sourceToken = sourceToken;
        this.targetToken = targetToken;
        this.message = message;
    }

    @Override
    public Token getSourceToken() {
        return sourceToken;
    }

    @Override
    public Token getTargetToken() {
        return targetToken;
    }

    @Override
    public String getMessage() {
        return message;
    }
}
