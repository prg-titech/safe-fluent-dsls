package prg.titech.chain.projection;

import com.fasterxml.jackson.annotation.JsonProperty;
import prg.titech.chain.token.Token;

public record ParseError(
        @JsonProperty("source_token") Token sourceToken,
        @JsonProperty("target_token") Token targetToken,
        String message
) implements Projection {

    public boolean essentiallyEqual(ParseError other) {
        if (other == null) {
            return false;
        }
        return sourceToken.getImage().equals(other.sourceToken.getImage())
            && targetToken.getImage().equals(other.targetToken.getImage());
    }
}
