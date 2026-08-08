package prg.titech.chain.token;

import jakarta.annotation.Nonnull;

import java.util.Iterator;
import java.util.Objects;
import java.util.Optional;

public record TokenRange(Token begin, Token end) implements Iterable<Token> {

    public static TokenRange from(com.github.javaparser.TokenRange range) {
        return new TokenRange(Token.from(range.getBegin()), Token.from(range.getEnd()));
    }

    @Override
    public @Nonnull Iterator<Token> iterator() {
        return new Iterator<>() {
            Token currentToken = begin;
            boolean hasReachedEnd = false;

            @Override
            public boolean hasNext() {
                return !hasReachedEnd;
            }

            @Override
            public Token next() {
                Token toReturn = currentToken;
                if (currentToken.equals(end)) {
                    hasReachedEnd = true;
                } else {
                    currentToken = currentToken.getNextToken().orElseThrow();
                }
                return toReturn;
            }
        };
    }

    public Optional<Range> toRange() {
        return begin.getRange().flatMap(
                begin -> end.getRange().map(
                        end -> new Range(begin.begin(), end.end())
                )
        );
    }

    @Override
    public @Nonnull String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append("[");
        Iterator<Token> it = iterator();
        sb.append(it.next().toString());    // SAFETY: Every token range is guaranteed to be nonempty
        while (it.hasNext()) {
            Token current = it.next();
            sb.append(", ");
            sb.append(current.toString());
        }
        sb.append("]");
        return sb.toString();
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) {
            return true;
        }
        if (obj == null) {
            return false;
        }
        if (obj instanceof TokenRange(Token oBegin, Token oEnd)) {
            return begin.equals(oBegin) && end.equals(oEnd);
        } else {
            return false;
        }
    }

    @Override
    public int hashCode() {
        return Objects.hash(begin, end);
    }
}
