package prg.titech.sql.translate;

import prg.titech.chain.Chain;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.translate.TokenList;
import prg.titech.chain.translate.Translator;

import java.util.*;

public class SQLTranslator implements Translator {
    private final TokenList tokens = new TokenList();
    private final static Set<String> allowedMethods;
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

        Set<String> m = new HashSet<>(keywordMapping.keySet());
        m.add("build");
        m.add("columnId");
        m.add("value");
        allowedMethods = m;
    }

    public static String translate(Chain chain) {
        return translateTokens(chain).toString();
    }

    public static TokenList translateTokens(Chain chain) {
        SQLTranslator self = new SQLTranslator();
        ChainTraverser traverser = new ChainTraverser(self, ChainTraverser.Strategy.ALL);
        chain.accept(traverser);
        return self.tokens;
    }

    @Override
    public TokenList getTokens() {
        return tokens;
    }

    @Override
    public boolean isMethodAllowed(String method) {
        return allowedMethods.contains(method);
    }

    @Override
    public Optional<String> getKeyword(String method) {
        return Optional.ofNullable(keywordMapping.get(method));
    }

    @Override
    public Optional<String> getDelimiter(String method) {
        return Optional.of(",");
    }
}
