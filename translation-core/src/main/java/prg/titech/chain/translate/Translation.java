package prg.titech.chain.translate;

import jakarta.annotation.Nullable;
import prg.titech.chain.token.Position;
import prg.titech.chain.token.Range;
import prg.titech.chain.token.Token;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class Translation {
    private final List<Token> targetTokens;
    private final Map<Token, List<Token>> sourceTokens;
    private Position currentCursorPosition = Position.HOME;

    public Translation() {
        this.targetTokens = new ArrayList<>();
        this.sourceTokens = new HashMap<>();
    }

    public void addToken(@Nullable List<Token> source, String target) {
        Range targetRange = new Range(currentCursorPosition, currentCursorPosition.right(target.length() - 1));
        currentCursorPosition = targetRange.end().right(1);
        Token targetToken = new Token(target, targetRange);
        this.targetTokens.add(targetToken);
        if (source != null) {
            this.sourceTokens.put(targetToken, source);
        }
    }

    public List<Token> findSourceOfOrAfter(Token target) {
        if (sourceTokens.containsKey(target)) {
            return sourceTokens.get(target);
        }
        List<Token> result = new ArrayList<>();
        for (Token t : targetTokens) {
            if (t.getRange().isBefore(target.getRange())) {
                continue;
            }

            if (sourceTokens.containsKey(t)) {
                result.addAll(sourceTokens.get(t));
                if (t.getRange().end().compareTo(target.getRange().end()) >= 0) {
                    break;
                }
            }
        }
        return result;
    }

    @Override
    public String toString() {
        return String.join("", targetTokens);
    }

    public String toDebugString() {
        StringBuilder sb = new StringBuilder();
        sb.append(targetTokens);
        sb.append("\n");
        sourceTokens.forEach((key, value) -> {
            sb.append(key.toDebugString());
            sb.append(" -> ");
            sb.append(value.stream().map(Token::toDebugString).toList());
            sb.append("\n");
        });
        return sb.toString();
    }
}
