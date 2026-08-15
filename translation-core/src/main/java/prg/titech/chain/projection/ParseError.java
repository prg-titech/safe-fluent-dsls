package prg.titech.chain.projection;

import prg.titech.chain.token.Token;

public record ParseError(Token sourceToken, Token targetToken, String message) implements Projection {

    public boolean essentiallyEqual(ParseError other) {
        if (other == null) {
            return false;
        }
        return sourceToken.getImage().equals(other.sourceToken.getImage())
            && targetToken.getImage().equals(other.targetToken.getImage());
    }
}
