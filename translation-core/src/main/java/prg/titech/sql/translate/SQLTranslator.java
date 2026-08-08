package prg.titech.sql.translate;

import prg.titech.chain.translate.Translation;
import prg.titech.chain.translate.Translator;
import prg.titech.chain.visit.Visitable;

import java.util.*;

public class SQLTranslator extends Translator {
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

    public static Translation translate(Visitable v) {
        SQLTranslator self = new SQLTranslator();
        Translation translation = new Translation();
        v.accept(self, translation);
        return translation;
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

    @Override
    public Optional<String> getQuoteDelimiter(String method) {
        if (method.equals("value")) {
            return Optional.of("\"");
        } else {
            return Optional.empty();
        }
    }

}
