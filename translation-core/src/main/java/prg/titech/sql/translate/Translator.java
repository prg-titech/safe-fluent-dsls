package prg.titech.sql.translate;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.value.StringValue;
import prg.titech.chain.visit.ChainVisitor;

import java.util.*;

public class Translator implements ChainVisitor {
    private final static Map<String, String> keywordMapping;

    static {
        Map<String, String> $ = new HashMap<>();
        $.put("select", "SELECT");
        $.put("from", "FROM");
        $.put("where", "WHERE");
        $.put("and", "AND");
        $.put("or", "OR");
        $.put("eq", "=");
        $.put("ne", "!=");
        $.put("lt", "<");
        $.put("le", "<=");
        $.put("gt", ">");
        $.put("ge", ">=");
        keywordMapping = Collections.unmodifiableMap($);
    }

    private final List<String> tokens = new ArrayList<>();

    public static List<String> translate(Chain chain) {
        Translator self = new Translator();
        ChainTraverser traverser = new ChainTraverser(self, ChainTraverser.Strategy.ALL);
        chain.accept(traverser);
        return self.tokens;
    }

    @Override
    public void visit(Frame context, Call call) {
        if (context.isFirstPosition()) {
            addKeyword(call.getMethodName());
        }
        if (context.isMiddlePosition()) {
            tokens.add(",");
        }
    }

    @Override
    public void visit(Frame context, StringValue value) {
        tokens.add(value.toString());
    }

    private void addKeyword(String methodName) {
        if (keywordMapping.containsKey(methodName)) {
            tokens.add(keywordMapping.get(methodName));
        }
    }
}
