package prg.titech.chain.token;

import com.github.javaparser.JavaToken;
import jakarta.annotation.Nullable;

import java.util.Optional;

/**
 * This class is used when translating a com.github.javaparser.JavaToken to our Token type.
 * <p>
 * As each token acts as a node in a doubly linked list, a real conversion would necessitate converting the entire
 * list. However, most of the time we only need a subset of this list, and we also want to convert each token
 * individually, and combine them later. Therefore, it would be hugely inefficient to demand a conversion of the entire
 * list whenever we want to convert a single token.
 * This class then acts as an intermediary. We convert the previous / next token only when we actually fetch it
 * using {#link getPreviousToken()} or {#link getNextToken()}. The result of this conversion is another lazytoken.
 * <p>
 * To the outside we hide that this class exists, by making Token a value-based class, where equality, the hash Value
 * and all other operations depend only on the value of the fields, not on its identity.
 */
public class LazyToken extends Token {
    private @Nullable JavaToken unconvertedPreviousToken;

    private @Nullable JavaToken unconvertedNextToken;

    public LazyToken(String image, Range range, @Nullable JavaToken unconvertedPreviousToken, @Nullable JavaToken unconvertedNextToken) {
        super(image, range, null, null);
        this.unconvertedPreviousToken = unconvertedPreviousToken;
        this.unconvertedNextToken = unconvertedNextToken;
    }

    @Override
    public Optional<Token> getNextToken() {
        nextToken = useFirstOrConvertSecond(nextToken, unconvertedNextToken);
        unconvertedNextToken = null;    // nextToken is now initialized, we don't need this anymore
        return Optional.ofNullable(nextToken);
    }

    private @Nullable Token useFirstOrConvertSecond(@Nullable Token first, @Nullable JavaToken second) {
        if (first != null) {
            return first;
        } else if (second != null) {
            return Token.from(second);
        } else {
            return null;
        }
    }
}
