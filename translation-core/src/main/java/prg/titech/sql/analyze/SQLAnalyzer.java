package prg.titech.sql.analyze;

import net.sf.jsqlparser.JSQLParserException;
import net.sf.jsqlparser.parser.CCJSqlParserUtil;
import net.sf.jsqlparser.parser.ParseException;
import prg.titech.chain.translate.TokenList;

import java.util.Optional;

public class SQLAnalyzer {

    public static Optional<ParseException> analyze(TokenList tokens) {
        String rawSQL = tokens.toString();
        try {
            CCJSqlParserUtil.parse(rawSQL);
            return Optional.empty();
        } catch (JSQLParserException e) {
            if (e.getCause().getCause() instanceof ParseException pe) {
                return Optional.of(pe);
            } else {
                return Optional.empty();
            }
        }
    }
}
