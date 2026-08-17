package prg.titech.chain.token;

import com.fasterxml.jackson.annotation.*;
import com.github.javaparser.JavaToken;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;

import java.util.Objects;
import java.util.Optional;
import java.util.stream.IntStream;

@JsonIncludeProperties({"image", "range"})
public class Token implements CharSequence {
    private final String image;

    private final Range range;

    protected @Nullable Token previousToken;

    protected @Nullable Token nextToken;

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

    @JsonCreator
    public Token(@JsonProperty("image") String image, @JsonProperty("range") Range range) {
        this.image = image;
        this.range = range;
        this.previousToken = null;
        this.nextToken = null;
    }

    public static Token from(JavaToken token) {
        return new LazyToken(
                token.getText(),
                token.getRange().map(Range::from).orElse(null),
                token.getPreviousToken().orElse(null),
                token.getNextToken().orElse(null)
        );
    }

    public String getImage() {
        return image;
    }

    public Range getRange() {
        return range;
    }

    public Optional<Token> getNextToken() {
        return Optional.ofNullable(nextToken);
    }

    public Token subToken(int start, int end) {
        if (start < 0) {
            start += length();
        }
        if (end < 0) {
            end += length();
        }
        return (Token) subSequence(start, end);
    }

    @Override
    public int length() {
        return image.length();
    }

    @Override
    public char charAt(int index) {
        return image.charAt(index);
    }

    @Override
    public boolean isEmpty() {
        return image.isEmpty();
    }

    @Override
    public @Nonnull CharSequence subSequence(int start, int end) {
        String newImage = image.substring(start, end);
        Position newStart = range.begin().right(start);
        Position newEnd = range.end().left(length() - end);
        return new Token(newImage, new Range(newStart, newEnd));
    }

    @Override
    public @Nonnull String toString() {
        return image;
    }

    public String toDebugString() {
        return String.format("<\"%s\", %s>", image.replace("\"", "\\\""), getRange());
    }

    @Override
    public @Nonnull IntStream chars() {
        return image.chars();
    }

    @Override
    public @Nonnull IntStream codePoints() {
        return image.codePoints();
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) {
            return true;
        }
        if (obj == null) {
            return false;
        }
        if (obj instanceof Token other) {
            return this.image.equals(other.image) && Objects.equals(this.range, other.range);
        } else {
            return false;
        }
    }

    @Override
    public int hashCode() {
        return Objects.hash(image, range);
    }


}
