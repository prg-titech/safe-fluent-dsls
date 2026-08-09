package prg.titech.chain.translate;

import com.google.common.collect.Streams;
import prg.titech.chain.token.Token;

import java.util.ArrayList;
import java.util.List;

public class Translation {
    private final List<String> targetTokens;
    private final List<List<Token>> sourceTokens;

    public Translation() {
        this.targetTokens = new ArrayList<>();
        this.sourceTokens = new ArrayList<>();
    }

    public void addToken(List<Token> source, String target) {
        this.targetTokens.add(target);
        this.sourceTokens.add(source);
    }

    @Override
    public String toString() {
        return String.join("", targetTokens);
    }

    public String toDebugString() {
        StringBuilder sb = new StringBuilder();
        Streams.forEachPair(sourceTokens.stream(), targetTokens.stream(), (s, t) -> {
            sb.append(s);
            sb.append(" -> \"");
            sb.append(t);
            sb.append("\"\n");
        });
        return sb.toString();
    }
}
