package prg.titech.chain.token;

import jakarta.annotation.Nullable;

import java.util.Objects;

public class Token {
    private final String image;

    private @Nullable Range range;

    private @Nullable Token previousToken;

    private @Nullable Token nextToken;

    public Token(String image, Range range, Token previousToken, Token nextToken) {
        this.image = Objects.requireNonNull(image);
        this.range = range;
        this.previousToken = previousToken;
        if (previousToken != null) {
            previousToken.nextToken = this;
        }
        this.nextToken = nextToken;
        if (nextToken != null) {
            nextToken.previousToken = this;
        }
    }

    public Token(String image, Range range, Token previousToken) {
        this(image, range, previousToken, null);
    }

    public Token

}
